use color_eyre::eyre::{Result, WrapErr};

use maelstrom::*;

fn main() -> Result<()> {
    color_eyre::install()?;

    let msg = Message {
        src: "c1".to_string(),
        dest: "n1".to_string(),
        body: Payload::Init(Init {
            msg_id: 1,
            node_id: "n1".to_string(),
            node_ids: vec!["n1".to_string(), "n2".to_string()],
        }),
    };

    let msg_str = serde_json::to_string_pretty(&msg).wrap_err("failed to serialize")?;

    println!("{}", msg_str);

    Ok(())
}
