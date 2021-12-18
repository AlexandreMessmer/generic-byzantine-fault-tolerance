use talk::{crypto::Identity, unicast::Acknowledger};

use crate::{talk::{Message, Instruction}, system::Client};

use super::Handler;
pub struct ClientHandler{}

#[async_trait::async_trait]
impl Handler for ClientHandler {
    async fn handle_message(&self, id: Identity, message: Message, ack: Acknowledger) {

    }

    async fn handle_instruction(&self, instruction: Instruction) {
    }
}
