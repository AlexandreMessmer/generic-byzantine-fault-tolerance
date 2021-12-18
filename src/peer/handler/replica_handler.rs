use talk::{unicast::{Message, Acknowledger}, crypto::Identity};

use crate::talk::Instruction;

use super::Handler;

pub struct ReplicaHandler{

}

#[async_trait::async_trait]
impl<T: Message> Handler<T> for ReplicaHandler {
    async fn handle_message(&self, id: Identity, message: T, ack: Acknowledger){

    }
    async fn handle_instruction(&self, instruction: Instruction){

    }
}