use talk::{crypto::Identity, unicast::Acknowledger};

use crate::{
    network::NetworkInfo,
    peer::peer::PeerId,
    talk::{Instruction, Message},
};

use super::{communicator::Communicator, Handler};

pub struct ReplicaHandler {
    communicator: Communicator<Message>,
}

impl ReplicaHandler {
    pub fn new(communicator: Communicator<Message>) -> Self {
        ReplicaHandler { communicator }
    }
}

#[async_trait::async_trait]
impl Handler<Message> for ReplicaHandler {
    async fn handle_message(&mut self, _id: Identity, message: Message, _ack: Acknowledger) {
        match message {
            Message::Testing => {
                println!("Replica #{} received the test", self.communicator.id())
            }
            _ => {}
        }
    }
    async fn handle_instruction(&mut self, instruction: Instruction) {
        match instruction {
            Instruction::Testing => {
                println!("Replica #{} received the test", self.communicator.id())
            }
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
