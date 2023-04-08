use std::collections::HashMap;

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

    Echo(Echo),
    EchoOk(EchoOk),

    Generate(Generate),
    GenerateOk(GenerateOk),

    Broadcast(Broadcast),
    BroadcastOk(BroadcastOk),

    Read(Read),
    ReadOk(ReadOk),

    Topology(Topology),
    TopologyOk(TopologyOk),
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Echo {
    pub msg_id: usize,
    pub echo: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EchoOk {
    pub msg_id: usize,
    pub in_reply_to: usize,
    pub echo: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Generate {
    pub msg_id: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GenerateOk {
    pub msg_id: usize,
    pub in_reply_to: usize,
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Broadcast {
    pub msg_id: usize,
    pub message: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BroadcastOk {
    pub msg_id: usize,
    pub in_reply_to: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Read {
    pub msg_id: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReadOk {
    pub msg_id: usize,
    pub in_reply_to: usize,
    pub messages: Vec<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Topology {
    pub msg_id: usize,
    pub topology: HashMap<String, Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TopologyOk {
    pub msg_id: usize,
    pub in_reply_to: usize,
}

#[derive(Debug)]
pub struct Node {
    pub node_id: String,
    pub next_msg_id: usize,
    pub seen_messages: Vec<i64>,
}

impl Node {
    pub fn new() -> Self {
        Self {
            node_id: "UNKNOWN_NODE".to_string(),
            next_msg_id: 0,
            seen_messages: Vec::new(),
        }
    }

    pub fn process_message(&mut self, message: &Message) -> Option<Message> {
        // msg_id is monotonically increasing
        self.next_msg_id += 1;

        match &message.body {
            Payload::Init(Init {
                msg_id,
                node_id,
                node_ids: _,
            }) => {
                self.node_id = node_id.clone();
                Some(Message {
                    src: self.node_id.clone(),
                    dest: message.src.clone(),
                    body: Payload::InitOk(InitOk {
                        in_reply_to: *msg_id,
                    }),
                })
            }
            Payload::InitOk(_) => panic!("didn't expect init_ok message"),

            Payload::Echo(Echo { msg_id, echo }) => Some(Message {
                src: self.node_id.clone(),
                dest: message.src.clone(),
                body: Payload::EchoOk(EchoOk {
                    msg_id: self.next_msg_id,
                    in_reply_to: *msg_id,
                    echo: echo.clone(),
                }),
            }),
            Payload::EchoOk(_) => None,

            Payload::Generate(Generate { msg_id }) => {
                // The pair (node_id, next_msg_id) is unique
                let id = format!("{}-{}", self.node_id, self.next_msg_id);

                Some(Message {
                    src: self.node_id.clone(),
                    dest: message.src.clone(),
                    body: Payload::GenerateOk(GenerateOk {
                        msg_id: self.next_msg_id,
                        in_reply_to: *msg_id,
                        id,
                    }),
                })
            }
            Payload::GenerateOk(_) => None,

            Payload::Broadcast(Broadcast {
                msg_id,
                message: msg,
            }) => {
                self.seen_messages.push(*msg);

                Some(Message {
                    src: self.node_id.clone(),
                    dest: message.src.clone(),
                    body: Payload::BroadcastOk(BroadcastOk {
                        msg_id: self.next_msg_id,
                        in_reply_to: *msg_id,
                    }),
                })
            }
            Payload::BroadcastOk(_) => None,

            Payload::Read(Read { msg_id }) => Some(Message {
                src: self.node_id.clone(),
                dest: message.src.clone(),
                body: Payload::ReadOk(ReadOk {
                    msg_id: self.next_msg_id,
                    in_reply_to: *msg_id,
                    messages: self.seen_messages.clone(),
                }),
            }),
            Payload::ReadOk(_) => None,

            Payload::Topology(Topology {
                msg_id,
                topology: _,
            }) => {
                // TODO: Store topology?

                Some(Message {
                    src: self.node_id.clone(),
                    dest: message.src.clone(),
                    body: Payload::TopologyOk(TopologyOk {
                        msg_id: self.next_msg_id,
                        in_reply_to: *msg_id,
                    }),
                })
            }
            Payload::TopologyOk(_) => None,
        }
    }
}
