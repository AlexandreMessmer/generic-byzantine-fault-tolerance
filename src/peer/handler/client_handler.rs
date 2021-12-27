use talk::{crypto::Identity, unicast::Acknowledger};
use tokio::sync::oneshot;

use crate::{
    database::client_database::ClientDatabase,
    error::DatabaseError,
    network::NetworkInfo,
    peer::peer::PeerId,
    talk::{Feedback, FeedbackSender, Instruction, Message, MessageResult, RequestId},
};

use super::{Communicator, Handler};
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
    fn handle_instruction_execute(&mut self, message: Message, id: &RequestId) {
        // Do not execute if there is a db error
        if self.database.contains_request(id) {
            self.communicator.send_feedback(Feedback::Error(format!(
                "Request #{} is already handled",
                *id
            )));
        } else {
            self.database.add_request(id.clone()).unwrap();
            self.broadast_to_replicas(&message);
        }
    }

    fn handle_instruction_testing(&mut self) {
        self.communicator
            .spawn_send_feedback(Feedback::Acknowledgement);
    }

    /// Handling messages functions

    fn handle_request_answer(
        &mut self,
        id: RequestId,
        message: Message,
        message_result: MessageResult,
        bound: usize,
    ) {
        if let Ok(()) = self
            .database
            .update_request(&id, message.clone())
            .map(|count| {
                if count == bound {
                    self.database.complete_request(&id);
                    // Needs to send feedback
                };
                ()
            })
        {}
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
            Instruction::Execute(message, id) => self.handle_instruction_execute(message, &id),
            Instruction::Testing => self.handle_instruction_testing(),
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
