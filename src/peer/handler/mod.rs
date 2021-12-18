use talk::{crypto::Identity, unicast::{Acknowledger, Message}};
pub mod client_handler;
pub mod replica_handler;

use crate::talk::Instruction;

#[async_trait::async_trait]
pub trait Handler<T> where T: Message {
    async fn handle_message(&self, id: Identity, message: T, ack: Acknowledger);
    async fn handle_instruction(&self, instruction: Instruction);
}