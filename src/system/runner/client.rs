use std::sync::Arc;

use super::PeerId;
use super::*;
use crate::crypto::identity_table::IdentityTable;
use crate::database::{self, Database};
use crate::error::DatabaseError;
use crate::talk::{Command, Message};
use crate::talk::{Feedback, FeedbackSender, MessageResult};
use futures::TryFutureExt;
use talk::crypto::Identity;
use talk::unicast::{Acknowledgement, SenderError};
use tokio::task::JoinHandle;
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

    async fn handle_instruction(&mut self, instruction: Instruction) {
        match instruction {
            (Command::Execute(message, id), tx) => self.handle_command_execute(message, &id, tx),
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
            Message::ACK(id, res, _) => {
                self.handle_request_answer(id, clone, res, self.runner.settings.n_ack())
            }
            Message::CHK(id, res, _) => {
                self.handle_request_answer(id, clone, res, self.runner.settings.nbr_byzantine())
            }
            _ => {}
        }
    }

    /// Handling command functions

    fn handle_command_execute(&mut self, message: Message, id: &RequestId, tx: FeedbackSender) {
        // Do not execute if there is a db error
        if self.database.contains_request(id) {
            tx.send_feedback(Feedback::Error(format!("Request #{} already exists", id)));
        } else {
            self.database.add_request(id.clone(), tx).unwrap();
            self.broadast_to_replicas(&message);
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

    fn handle_request_answer(
        &mut self,
        id: RequestId,
        message: Message,
        message_result: MessageResult,
        bound: usize,
    ) {
        if self.database.contains_request(&id) {
            let count = self
                .database
                .update_request(id.clone(), message.clone())
                .unwrap();
            if let Some(nbr) = count {
                if nbr == bound - 1 {
                    let completion_result = self.database.complete_request(&id);
                    match completion_result {
                        Ok(feedback_sender) => {
                            feedback_sender.send_feedback(Feedback::Result(message_result));
                        }
                        Err(database_error) => Self::handle_error(database_error),
                    }
                }
            }
        }
    }
    fn handle_message_testing(&mut self, message: &Message) {
        println!(
            "Client #{} receives {:?} during the test",
            self.id(),
            message
        );
    }

    fn handle_error(e: DatabaseError) {
        panic!("{}", e.error_message())
    }

    fn broadast_to_replicas(&self, message: &Message) {
        for replica in self.identity_table.replicas() {
            self.send(replica, message.clone());
        }
    }

    fn send(&self, target: &Identity, message: Message) {
        self.runner
            .peer()
            .spawn_send_message(target.clone(), message, &self.runner.fuse);
    }
}

#[async_trait::async_trait]
impl Runner for Client {
    async fn run(&mut self) {
        loop {
            tokio::select! {
                (id, message, _) = self.runner.peer.receive_message() => {
                    self.runner.simulate_delay().await;
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

impl Identifiable for Client {
    fn id(&self) -> PeerId {
        self.runner.id()
    }
}

#[cfg(test)]
mod tests {
    use crate::talk::Message;
    use std::collections::HashMap;

    use super::*;

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
