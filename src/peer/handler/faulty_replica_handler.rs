use talk::{crypto::Identity, unicast::Acknowledger};

use crate::{talk::{Message, Instruction}, peer::Peer};

use super::Handler;

pub struct FaultyReplicaHandler;

impl FaultyReplicaHandler {
    pub fn new() -> Self {
        FaultyReplicaHandler {}
    }
}


#[async_trait::async_trait]
impl Handler<Message> for FaultyReplicaHandler {
    async fn handle_message(&self, peer: &Peer<Message>, id: Identity, message: Message, ack: Acknowledger) {
        match message {
            _ => ()
        }
    }
    async fn handle_instruction(&self, peer: &Peer<Message>, instruction: Instruction) {
        match instruction {
            _ => {}
        }
    }
}