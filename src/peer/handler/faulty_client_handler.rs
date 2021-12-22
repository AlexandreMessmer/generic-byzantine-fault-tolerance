use talk::{crypto::Identity, unicast::Acknowledger};

use crate::{
    peer::Peer,
    talk::{Instruction, Message},
};

use super::Handler;

pub struct FaultyClientHandler {}

impl FaultyClientHandler {
    pub fn new() -> Self {
        FaultyClientHandler {}
    }
}

#[async_trait::async_trait]
impl Handler<Message> for FaultyClientHandler {
    async fn handle_message(
        &mut self,
        peer: &Peer<Message>,
        id: Identity,
        message: Message,
        ack: Acknowledger,
    ) {
        match message {
            _ => (),
        }
    }
    async fn handle_instruction(&mut self, peer: &Peer<Message>, instruction: Instruction) {
        match instruction {
            _ => {}
        }
    }
}
