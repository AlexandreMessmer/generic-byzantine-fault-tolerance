use tokio::sync::oneshot;
use talk::{crypto::Identity, unicast::{Acknowledger}};

use crate::{talk::{Instruction, RequestId, FeedbackSender, Feedback, Command, MessageResult, Message}, network::NetworkInfo, error::DatabaseError, database::client_database::ClientDatabase, peer::Peer};

use super::Handler;
pub struct ClientHandler {
    database: ClientDatabase,
}

impl ClientHandler{
    pub fn new() -> Self {
        ClientHandler {
            database: ClientDatabase::new(),
        }
    }

    /// Handling command functions
    fn handle_command_execute(&mut self, peer: &Peer<Message>, message: Message, id: &RequestId, tx: FeedbackSender) {
        // Do not execute if there is a db error
        if self.database.contains_request(id) {
            tx.send_feedback(
                Feedback::Error(format!("Request #{} already exists", id))
            );
        } else {
            self.database.add_request(id.clone(), tx).unwrap();
            self.broadast_to_replicas(peer, &message);
        }
    }

    fn handle_command_testing(&mut self, peer: &Peer<Message>, sender: Option<oneshot::Sender<Command>>) {
        println!("Client #{} starts testing... (broadcast to every peer)", peer.id());
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

    /// Handling messages functions

    fn handle_request_answer(&mut self, peer: &Peer<Message>, id: RequestId, message: Message, message_result: MessageResult, bound: usize) {
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
    fn handle_message_testing(&mut self, peer: &Peer<Message>, message: &Message) {
        println!(
            "Client #{} receives {:?} during the test",
            peer.id(),
            message
        );
    }

    fn handle_error(e: DatabaseError) {
        panic!("{}", e.error_message())
    }

    fn broadast_to_replicas(&self, peer: &Peer<Message>, message: &Message) {
        for replica in peer.identity_table().replicas() {
            let spawn = peer.spawn_send(replica.clone(), message.clone());
        }
    }

}
#[async_trait::async_trait]
impl Handler<Message> for ClientHandler{
    async fn handle_message(&self, peer: &Peer<Message>, id: Identity, message: Message, ack: Acknowledger) {
        let clone = message.clone();
        match message {
            Message::Testing => {
                self.handle_message_testing(peer, &message);
            }
            Message::ACK(id, request_message, message_result, _) => {
                self.handle_request_answer(peer, id, clone, message_result, peer.network_info().n_ack())
            }
            Message::CHK(id, request_message, message_result, _) => {
                self.handle_request_answer(peer, id, clone, message_result, peer.network_info().nbr_faulty_replicas())
            }
            _ => {}
        }
    }

    async fn handle_instruction(&self, peer: &Peer<Message>, instruction: Instruction) {
        match instruction {
            (Command::Execute(message, id), tx) => {
                self.handle_command_execute(&peer, message, &id, tx)
            }
            (Command::Testing(sender), _) => self.handle_command_testing(&peer, sender),
            _ => {}
        }
    }
}

