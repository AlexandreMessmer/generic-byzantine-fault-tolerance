pub mod client;
pub mod peer;
pub mod peer_runner;
pub mod replica;

use super::*;

pub use client::Client;
pub use replica::Replica;

pub type PeerId = usize;
pub struct Peer {
    id: PeerId,
    key: Identity,
    sender: Sender,
    receiver: Receiver,
}

pub type PeerIdentifier = (PeerType, PeerId);

pub enum PeerType {
    Client,
    Replica,
}

pub struct PeerRunner {
    peer: Peer,
    outlet: InstructionReceiver,
    keys_table: Vec<Identity>,
    fuse: Fuse,
    settings: RunnerSettings,
}

/// Defines the API of a Runner
/// Note that it should also implements an async method run()
pub trait Runner {
    fn send(&self, target: &Identity, message: Message);
    fn id(&self) -> PeerId;
}
