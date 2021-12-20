use crate::peer::Peer;
use talk::{
    crypto::Identity,
    unicast::{Acknowledger, Message},
};
pub mod client_handler;
pub mod replica_handler;
pub mod faulty_client_handler;
pub mod faulty_replica_handler;

pub use client_handler::ClientHandler;
pub use replica_handler::ReplicaHandler;
pub use faulty_client_handler::FaultyClientHandler;
pub use faulty_replica_handler::FaultyReplicaHandler;

use crate::talk::Instruction;

#[async_trait::async_trait]
pub trait Handler<T> : Sync + Send
where
    T: Message,
{
    async fn handle_message(&self, peer: &Peer<T>, id: Identity, message: T, ack: Acknowledger);
    async fn handle_instruction(&self, peer: &Peer<T>, instruction: Instruction);
}
