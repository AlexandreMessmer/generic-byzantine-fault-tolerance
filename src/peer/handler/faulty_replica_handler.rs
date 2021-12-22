use talk::{crypto::Identity, unicast::Acknowledger};

use crate::{
    network::NetworkInfo,
    peer::{peer::PeerId},
    talk::{Instruction, Message},
};

use super::{peer_handler::PeerHandler, Handler};

pub struct FaultyReplicaHandler {
    peer_handler: PeerHandler<Message>,
}

impl FaultyReplicaHandler {
    pub fn new(peer_handler: PeerHandler<Message>) -> Self {
        FaultyReplicaHandler { peer_handler }
    }
}

#[async_trait::async_trait]
impl Handler<Message> for FaultyReplicaHandler {
    async fn handle_message(&mut self, _id: Identity, message: Message, _ack: Acknowledger) {
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
