use std::collections::{HashMap, HashSet};

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

/// Ensures message IDs are unique and monotonically increasing.
#[derive(Debug)]
struct MessageIdCounter {
    _next_msg_id: usize,
}

impl MessageIdCounter {
    fn new() -> Self {
        Self { _next_msg_id: 0 }
    }

    fn next_msg_id(&mut self) -> usize {
        self._next_msg_id += 1;
        self._next_msg_id
    }
}

#[derive(Debug)]
struct PendingMessages {
    pending_nodes: HashSet<(String, i64)>,

    // TODO Clear out old sent_messages periodically. Assume they are lost.
    sent_messages: HashMap<usize, (String, i64)>,
}

impl PendingMessages {
    fn new() -> Self {
        Self {
            pending_nodes: HashSet::new(),
            sent_messages: HashMap::new(),
        }
    }

    fn clear_broadcast_ok(&mut self, broadcast_ok_in_reply_to: usize) {
        match self.sent_messages.get(&broadcast_ok_in_reply_to) {
            None => eprintln!(
                "got a broadcast_ok with unexpected ID {}",
                broadcast_ok_in_reply_to
            ),
            Some(node) => {
                self.pending_nodes.remove(node);
            }
        }
        self.sent_messages.remove(&broadcast_ok_in_reply_to);
    }

    fn broadcast_messages(
        &mut self,
        src_node_id: &str,
        msg_id_counter: &mut MessageIdCounter,
    ) -> Vec<Message> {
        self.pending_nodes
            .iter()
            .map(|(node, msg)| {
                broadcast_to_node_message(
                    src_node_id,
                    msg_id_counter,
                    node,
                    *msg,
                    &mut self.sent_messages,
                )
            })
            .collect()
    }

    fn broadcast_to_node(
        &mut self,
        src_node_id: &str,
        msg_id_counter: &mut MessageIdCounter,
        dest_node_id: &str,
        message: i64,
    ) -> Message {
        broadcast_to_node_message(
            src_node_id,
            msg_id_counter,
            dest_node_id,
            message,
            &mut self.sent_messages,
        )
    }
}

fn broadcast_to_node_message(
    src_node_id: &str,
    msg_id_counter: &mut MessageIdCounter,
    dest_node_id: &str,
    message: i64,
    sent_messages: &mut HashMap<usize, (String, i64)>,
) -> Message {
    let msg_id = msg_id_counter.next_msg_id();
    sent_messages.insert(msg_id, (dest_node_id.to_string(), message));
    Message {
        src: src_node_id.to_string(),
        dest: dest_node_id.to_string(),
        body: Payload::Broadcast(Broadcast { msg_id, message }),
    }
}

#[derive(Debug)]
pub struct Node {
    node_id: String,
    msg_id: MessageIdCounter,
    seen_messages: HashSet<i64>,

    /// Map from node ID to sibling node IDs
    topology: HashMap<String, Vec<String>>,

    /// Map from Node ID to pending messages for that node
    pending_messages: PendingMessages,
}

impl Node {
    pub fn new() -> Self {
        Self {
            node_id: "UNKNOWN_NODE".to_string(),
            msg_id: MessageIdCounter::new(),
            seen_messages: HashSet::new(),
            topology: HashMap::new(),
            pending_messages: PendingMessages::new(),
        }
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
                    msg_id: self.msg_id.next_msg_id(),
                    in_reply_to: *msg_id,
                    echo: echo.clone(),
                }),
            }],
            Payload::EchoOk(_) => vec![],

            Payload::Generate(Generate { msg_id: caller_id }) => {
                // The pair (node_id, next_msg_id) is unique
                let msg_id = self.msg_id.next_msg_id();
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

                // Respond to sender with BroadcastOk
                let mut responses = vec![Message {
                    src: self.node_id.clone(),
                    dest: message.src.clone(),
                    body: Payload::BroadcastOk(BroadcastOk {
                        msg_id: self.msg_id.next_msg_id(),
                        in_reply_to: *msg_id,
                    }),
                }];

                // Broadcast message to all peers if it is new
                if is_new {
                    let peers = self.topology.get(&self.node_id);
                    if let Some(peers) = peers {
                        let peers = peers.clone();

                        for peer in peers {
                            if peer == message.src {
                                // Don't send to the peer that just sent us this
                                continue;
                            }
                            self.pending_messages.pending_nodes.insert((peer.clone(), *msg));
                            responses.push(self.pending_messages.broadcast_to_node(
                                &self.node_id,
                                &mut self.msg_id,
                                &peer,
                                *msg,
                            ));
                        }
                    }
                }

                responses
            }

            Payload::BroadcastOk(BroadcastOk {
                msg_id: _,
                in_reply_to,
            }) => {
                self.pending_messages.clear_broadcast_ok(*in_reply_to);
                vec![]
            }

            Payload::Read(Read { msg_id }) => vec![Message {
                src: self.node_id.clone(),
                dest: message.src.clone(),
                body: Payload::ReadOk(ReadOk {
                    msg_id: self.msg_id.next_msg_id(),
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
                        msg_id: self.msg_id.next_msg_id(),
                        in_reply_to: *msg_id,
                    }),
                }]
            }
            Payload::TopologyOk(_) => vec![],
        }
    }

    pub fn pending_broadcasts(&mut self) -> Vec<Message> {
        // TODO: Clear out old pending messages

        self.pending_messages
            .broadcast_messages(&self.node_id, &mut self.msg_id)
    }
}
