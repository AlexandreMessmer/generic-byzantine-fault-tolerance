use std::collections::HashMap;

use crate::talk::{message::Message, RequestId, command::Command};

pub mod byzantine_system;
pub mod client;
pub mod peer;
pub mod peer_runner;
pub mod peer_system;
pub mod replica;
pub mod settings;
pub mod runner;

use talk::{unicast::{Receiver as TalkReceiver, Sender as TalkSender}, crypto::Identity, sync::fuse::Fuse};

type PeerIdentifier = (PeerType, PeerId);
type Sender = TalkSender<Message>;
type Receiver = TalkReceiver<Message>;

pub enum PeerType {
    Client,
    Replica,
}

type RequestDatabase = HashMap<RequestId, MessageDatabase>;
type MessageDatabase = HashMap<Message, usize>;
pub struct Database {
    received: MessageDatabase,
    request: RequestDatabase,
}
impl Database {
    pub fn new() -> Self {
        Database {
            received: HashMap::new(),
            request: HashMap::new(),
        }
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