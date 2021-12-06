use std::collections::HashMap;
use std::time::Duration;

use super::runner::Runner;
use super::Database;
use super::PeerId;
use super::*;
use crate::crypto::identity_table::IdentityTable;
use crate::settings::RunnerSettings as Settings;
use crate::talk::FeedbackSender;
use crate::talk::{Message, Command, Feedback};
use talk::crypto::Identity;
use tokio::time::sleep;
/// A client is a peer that has a defined behavior in the system
/// Formally, it is a client runner. To make it easier, we will
/// define the client as the same entity as its runner
pub struct Client {
    runner: PeerRunner,
    identity_table: IdentityTable,
    database: Database,
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
                    self.runner.simulate_delay().await;
                    self.handle_message(id, message).await;
                }

                Some(instruction) = self.runner.outlet.recv() => {
                    self.handle_command(instruction).await;
                }
            }
        }
    }

    async fn handle_command(&mut self, instruction: Instruction) {
        match instruction {
            (Command::Execute(message, id), tx) => {
                self.handle_command_execute(message, &id, tx)
                //todo!("Implement the wait until, and return a res");
            }
            (Command::Testing(sender), _) => {
                self.handle_command_testing(sender)
            }
            _ => {}
        }
    }

    async fn handle_message(&mut self, _id: Identity, message: Message){
        match message {
            Message::Testing => {
                self.handle_message_testing(&message);
            }
            Message::ACK(id, res) => {
                if let Some(mut key) = self.database.request.get(&id)  {
                    ()
                } else {
                    // Returns an error
                }
            }
            _ => {}
        }
    }


    /// Handling command functions

    fn handle_command_execute(&mut self, message: Message, id: &RequestId, tx: FeedbackSender) {
        if self.database.request.contains_key(&id) {
            let _ = tx.send_feedback(Feedback::Error(String::from(
                "The message has already been sent",
            )),
            &self.runner.fuse);
        } else {
            let replicas = self.identity_table.replicas().iter();
            for replica_id in replicas {
                self.runner.peer.sender.spawn_send(
                    replica_id.clone(),
                    message.clone(),
                    &self.runner.fuse,
                );
            }
            self.database.request.insert(id.clone(), MessageResultDatabase::new(tx));
        }
    }


    fn handle_command_testing(&mut self, sender: Option<OneShotSender<Command>>) {
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


    /// Handling messages functions

    fn handle_message_testing(&mut self, message: &Message) {
        println!("Client #{} receives the test", self.id());
        let db = &mut self.database.received;
        db.insert(
            message.clone(),
            if db.contains_key(&message) {
                db.get(&message).unwrap() + 1
            } else {
                1
            },
        );
    }
}

impl Runner for Client {
    fn send(&self, target: &Identity, message: Message) {
        self.runner
            .peer
            .sender
            .spawn_send(target.clone(), message, &self.runner.fuse);
    }

    fn id(&self) -> PeerId {
        self.runner.peer.id
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use uuid::Uuid;

    use crate::talk::{Command, Message};

    #[test]
    fn equality_for_enum() {
        let mut db: HashMap<Message, usize> = HashMap::new();
        let e1 = Message::Plaintext(String::from("Hello"));
        db.insert(e1.clone(), 0);
        let e2 = Message::Plaintext(String::from("Hello"));
        assert_eq!(e1 == e2, true);
        assert_eq!(db.contains_key(&e2), true);
    }
}
