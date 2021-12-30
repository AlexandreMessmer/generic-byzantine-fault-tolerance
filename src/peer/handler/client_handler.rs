use talk::{crypto::Identity, unicast::Acknowledger};
use tokio::sync::oneshot;

use crate::{
    database::client_database::{ClientDatabase, Request},
    error::DatabaseError,
    network::NetworkInfo,
    peer::peer::PeerId,
    talk::{command, Command, CommandId, Feedback, FeedbackSender, Instruction, Message, Phase},
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
    async fn handle_instruction_execute(&mut self, command: Command) {
        let id = command.id().clone();
        let message = Message::Command(command);
        // Do not execute if there is a db error
        if self.database.contains_request(&id) {
            self.communicator
                .send_feedback(Feedback::Error(format!(
                    "Request #{} is already handled",
                    id
                )))
                .await
                .unwrap();
        } else {
            self.database.add_request(id).unwrap();
            self.broadast_to_replicas(&message);
        }
    }

    fn handle_instruction_testing(&mut self) {
        self.communicator
            .spawn_send_feedback(Feedback::Acknowledgement);
    }

    /// Handling messages functions

    async fn handle_command_acknowledgement(&mut self, id: &CommandId, request: Request) {
        let (_, _, phase) = &request;
        let phase = phase.clone();
        if let Ok(count) = self.database.update_request(id, request) {
            let bound = if phase == Phase::ACK {
                self.communicator.network_info().n_ack()
            } else {
                self.communicator.network_info().nbr_faulty_replicas()
            };

            if count >= bound {
                self.database.complete_request(id).unwrap();
                self.acknowledge_completed_request().await;
            }
        }
    }

    async fn acknowledge_completed_request(&self) -> () {}
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
        match message {
            Message::Testing => {
                self.handle_message_testing(&message);
            }
            Message::CommandAcknowledgement(command, round, command_result, phase) => {
                self.handle_command_acknowledgement(command.id(), (round, command_result, phase))
                    .await;
            }
            _ => {}
        }
    }

    async fn handle_instruction(&mut self, instruction: Instruction) {
        match instruction {
            Instruction::Execute(command) => self.handle_instruction_execute(command).await,
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
