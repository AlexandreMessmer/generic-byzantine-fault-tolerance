use crate::peer::Peer;
use talk::{
    crypto::Identity,
    unicast::{Acknowledger, Message},
};
pub mod client_handler;
pub mod faulty_client_handler;
pub mod faulty_replica_handler;
pub mod replica_handler;

pub use client_handler::ClientHandler;
pub use faulty_client_handler::FaultyClientHandler;
pub use faulty_replica_handler::FaultyReplicaHandler;
pub use replica_handler::ReplicaHandler;

use crate::talk::Instruction;

use super::peer::PeerId;

#[async_trait::async_trait]
pub trait Handler<T>: Sync + Send
where
    T: Message,
{
    async fn handle_message(&mut self, peer: &Peer<T>, id: Identity, message: T, ack: Acknowledger);
    async fn handle_instruction(&mut self, peer: &Peer<T>, instruction: Instruction);
}
