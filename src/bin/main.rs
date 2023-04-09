use std::io::{BufRead, BufReader, Write};

use color_eyre::eyre::{Result, WrapErr};

use maelstrom::*;

fn main() -> Result<()> {
    color_eyre::install()?;

    let stdin = std::io::stdin();
    let reader = BufReader::new(stdin.lock());

    let stdout = std::io::stdout();
    let mut writer = stdout.lock();

    let mut node = Node::new();

    for line in reader.lines() {
        // Read input from a stdin line
        let line = line.wrap_err("failed to get stdin line")?;
        let message: Message = serde_json::from_str(&line).wrap_err("failed deserializing Message")?;

        // Process message and write to stdout line
        let responses = node.process_message(&message);
        for response in responses {
            let serialized = serde_json::to_string(&response).unwrap();
            writeln!(writer, "{}", serialized).unwrap();
        }
    }

    Ok(())
}
