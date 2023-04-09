use std::io::{BufRead, BufReader, StdoutLock, Write};
use std::thread;
use std::time::Duration;

use color_eyre::eyre::{Result, WrapErr};
use crossbeam_channel::{bounded, select, tick, unbounded};

use maelstrom::*;

fn main() -> Result<()> {
    color_eyre::install()?;

    let (done, quit) = bounded::<()>(0);

    // Spawn timer thread to send ticks
    let ticker = tick(Duration::from_millis(500));

    // Spawn thread to receive messages
    let (send_msg, rcv_msg) = unbounded::<Message>();
    thread::spawn(move || {
        let stdin = std::io::stdin();
        let reader = BufReader::new(stdin.lock());

        for line in reader.lines() {
            // Read input from a stdin line
            let line = line.wrap_err("failed to get stdin line").unwrap();
            let message: Message = serde_json::from_str(&line)
                .wrap_err("failed deserializing Message")
                .unwrap();
            send_msg
                .send(message)
                .wrap_err("failed sending message")
                .unwrap();
        }

        done.send(()).unwrap();
    });

    // Main thread
    let mut stdout = std::io::stdout().lock();
    let mut node = Node::new();

    loop {
        select! {
            recv(ticker) -> _ => {
                let messages = node.pending_broadcasts();
                write_messages_stdout(&mut stdout, &messages);
            }
            recv(rcv_msg) -> message => {
                let message = message.wrap_err("failed to receive message")?;
                let responses = node.process_message(&message);
                write_messages_stdout(&mut stdout, &responses);
            }
            recv(quit) -> _ => break,
        }
    }

    Ok(())
}

fn write_messages_stdout(stdout: &mut StdoutLock, messages: &[Message]) {
    for message in messages {
        let serialized = serde_json::to_string(&message).unwrap();
        writeln!(stdout, "{}", serialized).unwrap();
    }
}
