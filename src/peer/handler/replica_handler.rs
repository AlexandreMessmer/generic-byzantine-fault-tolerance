use talk::{crypto::Identity, unicast::Acknowledger};

use crate::{
    peer::Peer,
    talk::{Command, Instruction, Message},
};

use super::Handler;

pub struct ReplicaHandler {}

impl ReplicaHandler {
    pub fn new() -> Self {
        ReplicaHandler {}
    }
}

#[async_trait::async_trait]
impl Handler<Message> for ReplicaHandler {
    async fn handle_message(
        &mut self,
        peer: &Peer<Message>,
        id: Identity,
        message: Message,
        ack: Acknowledger,
    ) {
        match message {
            Message::Testing => {
                println!("Replica #{} receives the test!", peer.id())
            }
            _ => {}
        }
    }
    async fn handle_instruction(&mut self, peer: &Peer<Message>, instruction: Instruction) {
        match instruction {
            (Command::Testing(sender), _) => {
                println!("Replica #{} starts testing...", peer.id());
                for client in peer.identity_table().clients().iter() {
                    peer.spawn_send(client.clone(), Message::Testing);
                }
                for replica in peer.identity_table().replicas() {
                    peer.spawn_send(replica.clone(), Message::Testing);
                }
                if let Some(rx) = sender {
                    let _ = rx.send(Command::Answer);
                }
            }
            _ => {}
        }
    }
}
