pub mod client;
pub mod peer;
pub mod peer_runner;
pub mod replica;

use super::*;

pub use client::Client;
use doomstack::Top;
pub use replica::Replica;
use talk::unicast::{Acknowledger, Acknowledgement, SenderError};
use tokio::task::JoinHandle;

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

pub struct PeerRunner
{
    peer: Peer,
    outlet: InstructionReceiver,
    keys_table: Vec<Identity>,
    fuse: Fuse,
    settings: RunnerSettings,
}

/// Defines the API of a Runner
/// Note that it should also implements an async method run()
#[async_trait::async_trait]
pub trait Runner {
    async fn run(&mut self);
    fn fuse(&self) -> &Fuse;
}


#[async_trait::async_trait]
pub trait Communicator {
    /// The communicator sends the message within the calling thread
    async fn send_message(&self, remote: Identity, message: Message) -> Result<Acknowledgement, Top<SenderError>>;

    /// Block until the communicator receives the message
    async fn receive_message(&mut self) -> (Identity, Message, Acknowledger);

    /// The communicator sends the message in another thread.
    fn spawn_send_message(&self, remote: Identity, message: Message, fuse: &Fuse) -> JoinHandle<Option<Result<Acknowledgement, Top<SenderError>>>>;
}

pub trait Identifiable {
    /// Return the identifier of the peer within the system
    fn id(&self) -> PeerId;
}
