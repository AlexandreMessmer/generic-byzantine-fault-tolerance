use futures::channel::oneshot;
use talk::{crypto::Identity, unicast::{Acknowledger, Message}};

use crate::{talk::{Instruction, RequestId, FeedbackSender, Feedback, Command, MessageResult}, system::{Client, network_info::NetworkInfo}, error::DatabaseError, database::client_database::ClientDatabase};

use super::Handler;
pub struct ClientHandler {
    network_info: NetworkInfo,
    database: ClientDatabase,
}

impl ClientHandler{
    pub fn new(network_info: NetworkInfo) -> Self {
        ClientHandler {
            network_info,
            database: ClientDatabase::new(),
        }
    }

    /// Handling command functions
    fn handle_command_execute<T: Message>(&mut self, message: T, id: &RequestId, tx: FeedbackSender) {
        // Do not execute if there is a db error
        if self.database.contains_request(id) {
            tx.send_feedback(
                Feedback::Error(format!("Request #{} already exists", id))
            );
        } else {
            self.database.add_request(id.clone(), tx).unwrap();
            self.broadast_to_replicas(&message);
        }
    }

    fn handle_command_testing(&mut self, sender: Option<oneshot::Sender<Command>>) {
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

    fn handle_request_answer<T: Message>(&mut self, id: RequestId, message: T, message_result: MessageResult, bound: usize) {
        if self.database.contains_request(&id) {
            let count = self.database
            .update_request(id.clone(), message.clone())
            .unwrap();
        if let Some(nbr) = count {
            if nbr == bound - 1 {
                let completion_result = self.database
                    .complete_request(&id);
                match completion_result {
                    Ok(feedback_sender) => {
                        feedback_sender.send_feedback(Feedback::Result(message_result));
                    },
                    Err(database_error) => Self::handle_error(database_error),
                }
            }
        }
        }
    }
    fn handle_message_testing<T: Message>(&mut self, message: &T) {
        println!(
            "Client #{} receives {:?} during the test",
            self.id(),
            message
        );
    }

    fn handle_error(e: DatabaseError) {
        panic!("{}", e.error_message())
    }

    fn broadast_to_replicas<T: Message>(&self, message: &T) {
        for replica in self.identity_table.replicas() {
            self.send(replica, message.clone());
        }
    }

}
#[async_trait::async_trait]
impl<T> Handler<T> for ClientHandler where T: Message {
    async fn handle_message(&self, id: Identity, message: T, ack: Acknowledger) {
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

    async fn handle_instruction(&self, instruction: Instruction) {
        match instruction {
            (Command::Execute(message, id), tx) => {
                self.handle_command_execute(message, &id, tx)
            }
            (Command::Testing(sender), _) => self.handle_command_testing(sender),
            _ => {}
        }
    }
}

