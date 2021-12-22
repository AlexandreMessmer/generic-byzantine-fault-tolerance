use talk::{crypto::Identity, unicast::Acknowledger};

use crate::{
    crypto::identity_table::IdentityTable,
    network::NetworkInfo,
    peer::{peer::PeerId, Peer},
    talk::{Instruction, Message},
    types::*,
};

use super::{Handler, PeerHandler};

pub struct FaultyClientHandler {
    peer_handler: PeerHandler<Message>,
}

impl FaultyClientHandler {
    pub fn new(peer_handler: PeerHandler<Message>) -> Self {
        FaultyClientHandler { peer_handler }
    }
}

#[async_trait::async_trait]
impl Handler<Message> for FaultyClientHandler {
    async fn handle_message(&mut self, id: Identity, message: Message, ack: Acknowledger) {
        match message {
            _ => (),
        }
    }
    async fn handle_instruction(&mut self, instruction: Instruction) {
        match instruction {
            _ => {}
        }
    }

    fn id(&self) -> &PeerId {
        self.peer_handler.id()
    }

    fn network_info(&self) -> &NetworkInfo {
        self.peer_handler.network_info()
    }
}
