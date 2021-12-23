use talk::{crypto::Identity, unicast::Acknowledger};
use tokio::sync::oneshot;

use crate::{
    database::client_database::ClientDatabase,
    error::DatabaseError,
    network::NetworkInfo,
    peer::{peer::PeerId},
    talk::{Command, Feedback, FeedbackSender, Instruction, Message, MessageResult, RequestId},
};

use super::{Handler, Communicator};
pub struct ClientHandler {
    communicator: Communicator<Message>,
    database: ClientDatabase,
}

impl ClientHandler {
    pub fn new(communicator: Communicator<Message>) -> Self {
        ClientHandler {
            communicator,
            database: ClientDatabase::new(),
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

    fn handle_command_testing(&mut self, sender: Option<oneshot::Sender<Command>>) {
        println!(
            "Client #{} starts testing... (broadcast to every peer)",
            self.communicator.id()
        );
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
    fn handle_message_testing(&self, message: &Message) {
        println!(
            "Client #{} receives {:?} during the test",
            self.communicator.id(),
            message
        );
    }

    fn handle_error(e: DatabaseError) {
        panic!("{}", e.error_message())
    }

    fn broadast_to_replicas(&self, message: &Message) {
        for replica in self.communicator.identity_table().replicas() {
            let _spawn = self
                .communicator
                .spawn_send(replica.clone(), message.clone());
        }
    }
}
#[async_trait::async_trait]
impl Handler<Message> for ClientHandler {
    async fn handle_message(&mut self, _id: Identity, message: Message, _ack: Acknowledger) {
        let clone = message.clone();
        match message {
            Message::Testing => {
                self.handle_message_testing(&message);
            }
            Message::ACK(id, _request_message, message_result, _) => self.handle_request_answer(
                id,
                clone,
                message_result,
                self.communicator.network_info().n_ack(),
            ),
            Message::CHK(id, _request_message, message_result, _) => self.handle_request_answer(
                id,
                clone,
                message_result,
                self.communicator.network_info().nbr_faulty_replicas(),
            ),
            _ => {}
        }
    }

    async fn handle_instruction(&mut self, instruction: Instruction) {
        match instruction {
            (Command::Execute(message, id), tx) => self.handle_command_execute(message, &id, tx),
            (Command::Testing(sender), _) => self.handle_command_testing(sender),
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
