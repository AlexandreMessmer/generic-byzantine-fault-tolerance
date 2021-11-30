use std::collections::HashMap;
use std::time::Duration;

use crate::crypto::identity_table::IdentityTable;
use crate::talk::feedback::Feedback;
use crate::{system::Peer, talk::command::Command, talk::message::Message};
use talk::{crypto::Identity, sync::fuse::Fuse};
use tokio::sync::mpsc::Receiver as MPSCReceiver;
use tokio::time::sleep;
use uuid::Uuid;

use super::Database;
use super::PeerId;
use super::runner::Runner;
use super::*;
use crate::talk::RequestId;
/// A client is a peer that has a defined behavior in the system
/// Formally, it is a client runner. To make it easier, we will
/// define the client as the same entity as its runner
pub struct Client {
    runner: PeerRunner,
    identity_table: IdentityTable,
    database: Database
}

impl Client {
    pub fn new(runner: PeerRunner, identity_table: &IdentityTable) -> Self {
        Client {
            runner,
            identity_table: identity_table.clone(),
            database: Database::new(),
        }
    }

    pub async fn run(&mut self) {
        loop {
            tokio::select! {
                (id, message, _) = self.runner.peer.receiver.receive() => {
                    self.handle_message(id, message).await;
                }

                Some(command) = self.runner.outlet.recv() => {
                    self.handle_command(command).await;
                }
            }
        }
    }

    async fn handle_command(&mut self, command: Command) {
        match command {
            Command::Execute(message, id, tx) => {
                if self.database.request.contains_key(&id) {
                    let _ = tx.send(Feedback::Error(String::from("The message has already been sent")));
                }
                else {
                    let replicas = self.identity_table.replicas().iter();
                    for replica_id in replicas {
                        self.runner.peer.sender.spawn_send(
                            replica_id.clone(),
                            message.clone(),
                            &self.runner.fuse,
                        );
                    }
                    self.database.request.insert(id, HashMap::new());
                }
                //todo!("Implement the wait until, and return a res");
            },
            Command::Testing(sender) => {
                println!("Client #{} starts testing...", self.id());
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
                println!("Client #{} receives the test", self.id())
            },
            Message::ACK(id, res) => {
                (id, res);
            }
            _ => {}
        }
    }

    async fn wait_for_replicas() -> u64 {
        sleep(Duration::from_secs(10)).await;
        2
    }

}

impl Runner for Client {
    fn send(&self, target: &Identity, message: Message){
        self.runner.peer.sender.spawn_send(target.clone(), message, &self.runner.fuse);
    }

    fn id(&self) -> PeerId{
        self.runner.peer.id
    }

}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use uuid::Uuid;

    use crate::talk::{command::Command, message::Message};

    #[test]
    fn equality_for_enum(){
        let mut db: HashMap<Message, usize> = HashMap::new();
        let e1 = Message::Plaintext(String::from("Hello"));
        db.insert(e1.clone(), 0);
        let e2 = Message::Plaintext(String::from("Hello"));
        assert_eq!(e1 == e2, true);
        assert_eq!(db.contains_key(&e2), true);
    }
}