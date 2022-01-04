use talk::{crypto::Identity, unicast::Acknowledger};

use crate::{
    network::NetworkInfo,
    peer::peer::PeerId,
    talk::{Instruction, Message},
};

use super::{Handler, ReplicaHandler};

pub struct FaultyReplicaHandler {
    replica_handler: ReplicaHandler,
}

impl FaultyReplicaHandler {
    pub fn new(replica_handler: ReplicaHandler) -> Self {
        FaultyReplicaHandler { replica_handler }
    }

    fn handle_message_testing(&self) {
        print!("Faulty replica #{} received the test", self.id());
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
            Instruction::Shutdown => self.replica_handler.shutdown().await,
            _ => {}
        }
    }

    fn id(&self) -> &PeerId {
        self.replica_handler.id()
    }

    fn network_info(&self) -> &NetworkInfo {
        self.replica_handler.network_info()
    }
}
