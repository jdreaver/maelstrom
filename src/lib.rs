use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    /// A string identifying the node this message came from
    pub src: String,

    ///  A string identifying the node this message is to
    pub dest: String,

    /// An object: the payload of the message
    pub body: Payload,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum Payload {
    Init(Init),
    InitOk(InitOk),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Init {
    pub msg_id: usize,
    pub node_id: String,
    pub node_ids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InitOk {
    pub in_reply_to: usize,
}

#[derive(Debug)]
pub struct Node {
    pub node_id: String,
    pub next_msg_id: usize,
}

impl Node {
    pub fn new() -> Self {
        Self {
            node_id: "UNKNOWN_NODE".to_string(),
            next_msg_id: 0,
        }
    }

    pub fn process_message(&mut self, message: &Message) -> Message {
        // msg_id is monotonically increasing
        self.next_msg_id += 1;

        match &message.body {
            Payload::Init(Init {
                msg_id,
                node_id,
                node_ids: _,
            }) => {
                self.node_id = node_id.clone();
                Message {
                    src: self.node_id.clone(),
                    dest: message.src.clone(),
                    body: Payload::InitOk(InitOk {
                        in_reply_to: *msg_id,
                    }),
                }
            }
            Payload::InitOk(_) => panic!("didn't expect init_ok message"),
        }
    }
}