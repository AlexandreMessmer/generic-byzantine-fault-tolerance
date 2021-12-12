use super::PeerId;
use super::*;
use crate::crypto::identity_table::IdentityTable;
use crate::database::Database;
use crate::talk::FeedbackSender;
use crate::talk::{Command, Message};
use talk::crypto::Identity;
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
            (Command::Testing(sender), _) => self.handle_command_testing(sender),
            _ => {}
        }
    }

    async fn handle_message(&mut self, _id: Identity, message: Message) {
        let clone = message.clone();
        match message {
            Message::Testing => {
                self.handle_message_testing(&message);
            }
            Message::ACK(id, _) => {
                self.database.update_request(id, message);
            }
            _ => {}
        }
    }

    /// Handling command functions

    fn handle_command_execute(&mut self, message: Message, id: &RequestId, tx: FeedbackSender) {
        //self.database.
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
        println!(
            "Client #{} receives {:?} during the test",
            self.id(),
            message
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
