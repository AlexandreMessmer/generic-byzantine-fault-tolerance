use talk::{crypto::Identity, unicast::Acknowledger};

use crate::{
    network::NetworkInfo,
    peer::{peer::PeerId, shutdownable::Shutdownable},
    talk::{Instruction, Message},
};

use super::{communicator::Communicator, Handler, ReplicaHandler};

pub struct FaultyReplicaHandler {
    communicator: Communicator<Message>,
}

impl FaultyReplicaHandler {
    pub fn new(communicator: Communicator<Message>) -> Self {
        FaultyReplicaHandler { communicator}
    }

    fn handle_message_testing(&self) {
        print!(
            "Faulty replica #{} received the test",
            self.id()
        );
    }
}

#[async_trait::async_trait]
impl Handler<Message> for FaultyReplicaHandler {
    async fn handle_message(&mut self, _id: Identity, message: Message, _ack: Acknowledger) {
        match message {
            Message::Testing => {
                self.handle_message_testing();
            }
            _ => (),
        }
    }
    async fn handle_instruction(&mut self, instruction: Instruction) {
        match instruction {
            Instruction::Shutdown => self.communicator.shutdown().await,
            _ => {}
        }
    }

    fn id(&self) -> &PeerId {
        self.communicator.id()
    }

    fn network_info(&self) -> &NetworkInfo {
        self.communicator.network_info()
    }
}
