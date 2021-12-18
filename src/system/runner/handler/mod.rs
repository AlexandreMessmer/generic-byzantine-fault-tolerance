use talk::{crypto::Identity, unicast::{Acknowledger}};
pub mod client_handler;
pub mod replica_handler;

use crate::talk::{Instruction, Message};

#[async_trait::async_trait]
pub trait Handler {
    async fn handle_message(&self, id: Identity, message: Message, ack: Acknowledger);
    async fn handle_instruction(&self, instruction: Instruction);
}