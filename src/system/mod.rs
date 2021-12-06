use std::collections::HashMap;

use crate::{
    settings::RunnerSettings,
    talk::{Command, Message, MessageResult, RequestId, FeedbackSender, Feedback, Instruction},
};

pub mod byzantine_system;
pub mod client;
pub mod peer;
pub mod peer_runner;
pub mod peer_system;
pub mod replica;
pub mod runner;

use talk::{
    crypto::Identity,
    sync::fuse::Fuse,
    unicast::{Receiver as TalkReceiver, Sender as TalkSender},
};
use tokio::sync::{
    oneshot::Sender as OneShotSender
};
type PeerIdentifier = (PeerType, PeerId);

// Definition of some senders and receivers
type InstructionSender = MPSCSender<Instruction>;
type InstructionReceiver = MPSCReceiver<Instruction>;
type Sender = TalkSender<Message>;
type Receiver = TalkReceiver<Message>;

pub enum PeerType {
    Client,
    Replica,
}

type RequestDatabase = HashMap<RequestId, MessageResultDatabase>;
type ResultHashMap = HashMap<MessageResult, usize>;
type MessageDatabase = HashMap<Message, usize>;
pub struct Database {
    received: MessageDatabase,
    request: RequestDatabase,
}
impl Database {
    pub fn new() -> Self {
        Database {
            received: HashMap::new(),
            request: HashMap::<RequestId, MessageResultDatabase>::new(),
        }
    }
}

struct MessageResultDatabase {
    results: ResultHashMap,
    result_inlet: FeedbackSender,
}
impl MessageResultDatabase {
    fn new(result_inlet: FeedbackSender) -> Self {
        MessageResultDatabase { results: HashMap::new(), result_inlet }
    }

    fn add(&mut self, key: MessageResult) -> Option<usize>{
        let value = self.results.get(&key).map_or(1, |x| x + 1);
        self.results.insert(key, value)
    }

}

pub type PeerId = usize;
struct Peer {
    id: PeerId,
    key: Identity,
    sender: Sender,
    receiver: Receiver,
}

use tokio::sync::mpsc::Receiver as MPSCReceiver;
use tokio::sync::mpsc::Sender as MPSCSender;

pub struct PeerRunner {
    peer: Peer,
    outlet: MPSCReceiver<Command>,
    keys_table: Vec<Identity>,
    fuse: Fuse,
    settings: RunnerSettings,
}

/// Define a system of peers. Command can be given to peer through the MPSC channel.
///
/// Each peer runs a `PeerRunner` that handles incoming request. We can pass information to each peer through
/// message handling (see https://tokio.rs/tokio/tutorial/channels).
///
/// The index of each peer runner inlet is the id of the peer, from 0 to size (excluded)
pub struct PeerSystem {
    size: usize,
    runner_inlets: Vec<MPSCSender<Command>>,
    fuse: Fuse,
}
