use std::collections::{HashMap, HashSet, VecDeque};

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
    pub messages: HashSet<i64>,
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
    node_id: String,
    next_msg_id: usize,
    seen_messages: HashSet<i64>,

    /// Map from node ID to sibling node IDs
    topology: HashMap<String, Vec<String>>,

    /// Map from Node ID to pending messages for that node
    pending_messages: HashMap<String, NodePendingMessages>,
}

#[derive(Debug)]
struct NodePendingMessages {
    /// Messages that haven't been sent to the node
    next_messages: VecDeque<i64>,

    /// The message ID that was used to send the i64 at the front of the queue.
    pending_message_id: Option<usize>,
}

impl NodePendingMessages {
    fn new() -> Self {
        Self {
            next_messages: VecDeque::new(),
            pending_message_id: None,
        }
    }
}

impl Node {
    pub fn new() -> Self {
        Self {
            node_id: "UNKNOWN_NODE".to_string(),
            next_msg_id: 0,
            seen_messages: HashSet::new(),
            topology: HashMap::new(),
            pending_messages: HashMap::new(),
        }
    }

    fn get_msg_id(&mut self) -> usize {
        // msg_id is monotonically increasing
        self.next_msg_id += 1;
        self.next_msg_id
    }

    pub fn process_message(&mut self, message: &Message) -> Vec<Message> {
        match &message.body {
            Payload::Init(Init {
                msg_id,
                node_id,
                node_ids: _,
            }) => {
                self.node_id = node_id.clone();
                vec![Message {
                    src: self.node_id.clone(),
                    dest: message.src.clone(),
                    body: Payload::InitOk(InitOk {
                        in_reply_to: *msg_id,
                    }),
                }]
            }
            Payload::InitOk(_) => panic!("didn't expect init_ok message"),

            Payload::Echo(Echo { msg_id, echo }) => vec![Message {
                src: self.node_id.clone(),
                dest: message.src.clone(),
                body: Payload::EchoOk(EchoOk {
                    msg_id: self.get_msg_id(),
                    in_reply_to: *msg_id,
                    echo: echo.clone(),
                }),
            }],
            Payload::EchoOk(_) => vec![],

            Payload::Generate(Generate { msg_id: caller_id }) => {
                // The pair (node_id, next_msg_id) is unique
                let msg_id = self.get_msg_id();
                let id = format!("{}-{}", self.node_id, msg_id);

                vec![Message {
                    src: self.node_id.clone(),
                    dest: message.src.clone(),
                    body: Payload::GenerateOk(GenerateOk {
                        msg_id,
                        in_reply_to: *caller_id,
                        id,
                    }),
                }]
            }
            Payload::GenerateOk(_) => vec![],

            Payload::Broadcast(Broadcast {
                msg_id,
                message: msg,
            }) => {
                let is_new = self.seen_messages.insert(*msg);

                // Broadcast message to all peers if it is new
                if is_new {
                    let peers = self.topology.get(&self.node_id);
                    if let Some(peers) = peers {
                        let peers = peers.clone();
                        for peer in peers {
                            self.pending_messages
                                .entry(peer.clone())
                                .and_modify(|pending| pending.next_messages.push_back(*msg))
                                .or_insert(NodePendingMessages::new());
                        }
                    }
                }

                // Respond to sender with BroadcastOk
                vec![Message {
                    src: self.node_id.clone(),
                    dest: message.src.clone(),
                    body: Payload::BroadcastOk(BroadcastOk {
                        msg_id: self.get_msg_id(),
                        in_reply_to: *msg_id,
                    }),
                }]
            }
            Payload::BroadcastOk(BroadcastOk {
                msg_id: _,
                in_reply_to,
            }) => {
                match self.pending_messages.get_mut(&message.src) {
                    None => {
                        eprintln!(
                            "got broadcast_ok, but no pending messages for node: {:?}",
                            message
                        );
                        vec![]
                    }
                    Some(mut pending) => {
                        if Some(*in_reply_to) == pending.pending_message_id {
                            // TODO clear first pending message, and also send
                            // the next one immediately if it exists
                        }
                        vec![]
                    }
                }
            }

            Payload::Read(Read { msg_id }) => vec![Message {
                src: self.node_id.clone(),
                dest: message.src.clone(),
                body: Payload::ReadOk(ReadOk {
                    msg_id: self.get_msg_id(),
                    in_reply_to: *msg_id,
                    messages: self.seen_messages.clone(),
                }),
            }],
            Payload::ReadOk(_) => vec![],

            Payload::Topology(Topology { msg_id, topology }) => {
                eprintln!("got topology {:?}", topology);
                self.topology = topology.clone();

                vec![Message {
                    src: self.node_id.clone(),
                    dest: message.src.clone(),
                    body: Payload::TopologyOk(TopologyOk {
                        msg_id: self.get_msg_id(),
                        in_reply_to: *msg_id,
                    }),
                }]
            }
            Payload::TopologyOk(_) => vec![],
        }
    }

    pub fn pending_broadcasts(&mut self) -> Vec<Message> {
        todo!()
        // self.pending_messages
        //     .iter_mut()
        //     .flat_map(|(node, pending)| {
        //         pending.next_messages.front().map(|msg| Message {
        //             src: self.node_id.clone(),
        //             dest: node.clone(),
        //             body: Payload::Broadcast(Broadcast {
        //                 msg_id: self.get_msg_id(),
        //                 message: *msg,
        //             }),
        //         })
        //     })
        //     .collect()
    }
}
