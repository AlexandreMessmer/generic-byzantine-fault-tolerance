use talk::{crypto::Identity, unicast::Acknowledger};

use crate::{
    network::NetworkInfo,
    peer::{peer::PeerId},
    talk::{Command, Instruction, Message},
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
                println!("Replica #{} receives the test!", self.communicator.id())
            }
            _ => {}
        }
    }
    async fn handle_instruction(&mut self, instruction: Instruction) {
        match instruction {
            (Command::Testing(sender), _) => {
                println!("Replica #{} starts testing...", self.communicator.id());
                for client in self.communicator.identity_table().clients().iter() {
                    self.communicator
                        .spawn_send(client.clone(), Message::Testing);
                }
                for replica in self.communicator.identity_table().replicas() {
                    self.communicator
                        .spawn_send(replica.clone(), Message::Testing);
                }
                if let Some(rx) = sender {
                    let _ = rx.send(Command::Answer);
                }
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
