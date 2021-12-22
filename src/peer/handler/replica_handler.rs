use talk::{crypto::Identity, unicast::Acknowledger};

use crate::{
    network::NetworkInfo,
    peer::{peer::PeerId},
    talk::{Command, Instruction, Message},
};

use super::{peer_handler::PeerHandler, Handler};

pub struct ReplicaHandler {
    peer_handler: PeerHandler<Message>,
}

impl ReplicaHandler {
    pub fn new(peer_handler: PeerHandler<Message>) -> Self {
        ReplicaHandler { peer_handler }
    }
}

#[async_trait::async_trait]
impl Handler<Message> for ReplicaHandler {
    async fn handle_message(&mut self, _id: Identity, message: Message, _ack: Acknowledger) {
        match message {
            Message::Testing => {
                println!("Replica #{} receives the test!", self.peer_handler.id())
            }
            _ => {}
        }
    }
    async fn handle_instruction(&mut self, instruction: Instruction) {
        match instruction {
            (Command::Testing(sender), _) => {
                println!("Replica #{} starts testing...", self.peer_handler.id());
                for client in self.peer_handler.identity_table().clients().iter() {
                    self.peer_handler
                        .spawn_send(client.clone(), Message::Testing);
                }
                for replica in self.peer_handler.identity_table().replicas() {
                    self.peer_handler
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
        self.peer_handler.id()
    }

    fn network_info(&self) -> &NetworkInfo {
        self.peer_handler.network_info()
    }
}
