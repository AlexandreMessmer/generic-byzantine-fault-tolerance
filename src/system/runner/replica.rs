use super::*;
use crate::crypto::identity_table::IdentityTable;
use crate::{talk::Command, talk::Message};
use talk::crypto::Identity;

/// A replica is a peer that has a defined behavior in the system
/// Formally, it is a replica runner. To make it easier, we will
/// define the replica as the same entity as its runner

pub struct Replica {
    runner: PeerRunner,
    identity_table: IdentityTable,
}

impl Replica {
    pub fn new(runner: PeerRunner, identity_table: &IdentityTable) -> Self {
        Replica {
            runner,
            identity_table: identity_table.clone(),
        }
    }

    async fn handle_instruction(&mut self, instruction: Instruction) {
        match instruction {
            (Command::Testing(sender), _) => {
                println!("Replica #{} starts testing...", self.id());
                for client in self.identity_table.clients().iter() {
                    self.send(client, Message::Testing);
                }
                for replica in self.identity_table.replicas() {
                    self.send(replica, Message::Testing);
                }
                if let Some(rx) = sender {
                    let _ = rx.send(Command::Answer);
                }
            }
            _ => {}
        }
    }

    async fn handle_message(&mut self, _id: Identity, message: Message) {
        match message {
            Message::Testing => {
                println!("Replica #{} receives the test!", self.id())
            }
            _ => {}
        }
    }

    fn send(&self, target: &Identity, message: Message) {
        self.runner
            .peer()
            .spawn_send_message(target.clone(), message, self.fuse());
    }
}

#[async_trait::async_trait]
impl Runner for Replica {
    async fn run(&mut self) {
        loop {
            tokio::select! {
                (id, message, _) = self.runner.peer.receive_message() => {
                    self.handle_message(id, message).await;
                }

                Some(instruction) = self.runner.outlet.recv() => {
                    self.handle_instruction(instruction).await;
                }
            }
        }
    }

    fn fuse(&self) -> &Fuse {
        self.runner.fuse()
    }
}

impl Identifiable for Replica {
    fn id(&self) -> PeerId {
        self.runner.id()
    }
}
