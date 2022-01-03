use talk::{crypto::Identity, unicast::Acknowledger};

use crate::{
    network::NetworkInfo,
    peer::{peer::PeerId, shutdownable::Shutdownable},
    talk::{Instruction, Message},
};

use super::{client_handler, ClientHandler, Communicator, Handler};

pub struct FaultyClientHandler {
    communicator: Communicator<Message>,
}

impl FaultyClientHandler {
    pub fn new(communicator: Communicator<Message>) -> Self {
        FaultyClientHandler { communicator }
    }

    fn handle_message_testing(&self) {
        print!(
            "Faulty client #{} received the test",
            self.communicator.id()
        );
    }
}

#[async_trait::async_trait]
impl Handler<Message> for FaultyClientHandler {
    async fn handle_message(&mut self, _id: Identity, message: Message, _ack: Acknowledger) {
        match message {
            Message::Testing => self.handle_message_testing(),
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
