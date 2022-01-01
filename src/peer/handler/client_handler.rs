use talk::{crypto::Identity, unicast::Acknowledger};


use crate::{
    database::client_database::{ClientDatabase, RequestResult},
    error::DatabaseError,
    network::NetworkInfo,
    peer::peer::PeerId,
    talk::{Command, CommandId, Feedback, Instruction, Message, Phase, CommandResult},
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
            self.broadast_to_replicas(&message).await;
        }
    }

    fn handle_instruction_testing(&mut self) {
        self.communicator
            .spawn_send_feedback(Feedback::Acknowledgement);
    }

    /// Handling messages functions

    async fn handle_command_acknowledgement(&mut self, id: &CommandId, request_result: RequestResult) {
        let (_, command_result, phase) = request_result.clone();
        if let Ok(count) = self.database.update_request(id, request_result) {
            let bound = match phase {
                Phase::ACK => self.communicator.network_info().n_ack(),
                Phase::CHK => self.communicator.network_info().nbr_faulty_replicas()
            };

            if count >= bound {
                self.database.complete_request(id).unwrap();
                self.communicator.send_feedback(Feedback::Result(command_result)).await.unwrap();
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

    async fn broadast_to_replicas(&self, message: &Message) {
        for replica in self.communicator.identity_table().replicas() {
            let _spawn = self
                .communicator
                .spawn_send(replica.clone(), message.clone()).await;
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

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use talk::{sync::fuse::Fuse};
    use tokio::time::timeout;

    use crate::{network::NetworkInfo, tests::util::Utils, peer::handler::{Communicator, Handler}, talk::{FeedbackChannel, Command, Message, Instruction}, crypto::identity_table::{IdentityTableBuilder, self}, banking::action::Action};

    use super::*;

    #[tokio::test]
    async fn instruction_execute_test() {
        /* Template to build a ClientHandler */
        let network_info = NetworkInfo::new(1, 2, 0, 0, 10, 1);
        let (mut keys, mut senders, mut receivers) = Utils::mock_network(3).await;
        let (client, sender, mut receiver) = Utils::pop(&mut keys, &mut senders, &mut receivers);
        let (replica1, sender1, mut receiver1) = Utils::pop(&mut keys, &mut senders, &mut receivers);
        let (replica2, sender2, mut receiver2) = Utils::pop(&mut keys, &mut senders, &mut receivers);

        let (rx, mut tx) = FeedbackChannel::channel();
        let client_id = client.clone();
        let identity_table = IdentityTableBuilder::new(network_info.clone())
            .add_peer(client.clone())
            .add_peer(replica1.clone())
            .add_peer(replica2.clone())
            .build();
        let mut client = ClientHandler::new(Communicator::new(0, client, sender, rx, network_info.clone(), identity_table.clone()));
        /* ------------------------ */

        // Test broadcast to replicas as well
        let cmd = Command::new(0, Action::Deposit(10));
        client.handle_instruction_execute(cmd.clone()).await;
        let message = Message::Command(cmd.clone());
        let (id1, msg1, _) = timeout(Duration::from_secs(2), receiver1.receive()).await.unwrap();
        let (id2, msg2 , _) = receiver2.receive().await;

        assert_eq!(id1, client_id);
        assert_eq!(id2, client_id);

        assert_eq!(msg1, message);
        assert_eq!(msg2, message);

        assert_eq!(client.database.contains_request(cmd.id()), true);

        client.handle_instruction_execute(cmd.clone()).await;
        let err = tx.recv().await.unwrap();
        println!("{:?}", err);

    }

    #[tokio::test]
    async fn correclty_handle_request() {
        let network_info = NetworkInfo::new(1, 2, 0, 0, 10, 7);
        let (mut keys, mut senders, mut receivers) = Utils::mock_network(3).await;
        let (client, sender, mut receiver) = Utils::pop(&mut keys, &mut senders, &mut receivers);
        let (replica1, sender1, mut receiver1) = Utils::pop(&mut keys, &mut senders, &mut receivers);
        let (replica2, sender2, mut receiver2) = Utils::pop(&mut keys, &mut senders, &mut receivers);

        let (rx, mut tx) = FeedbackChannel::channel();
        let client_id = client.clone();
        let identity_table = IdentityTableBuilder::new(network_info.clone())
            .add_peer(client.clone())
            .add_peer(replica1.clone())
            .add_peer(replica2.clone())
            .build();
        let mut client = ClientHandler::new(Communicator::new(0, client, sender, rx, network_info.clone(), identity_table.clone()));

        let cmd = Command::new(0, Action::Register);
        client.database.add_request(cmd.id().clone()).unwrap();

        // Case 1: Receives a bunch of requests with wrong round numbers
        let fuse = Fuse::new();
        for i in 10..100 {
            sender1.spawn_send(client_id.clone(), Message::CommandAcknowledgement(cmd.clone(), i, CommandResult::Success(None), Phase::ACK), &fuse);
        }

        for _ in 10..100 {
            let future = receiver.receive();
            timeout(Duration::from_millis(100), future).await.expect("Error when sending");
        }

        for (_, v) in client.database.requests().get(cmd.id()).expect("Failed to find the request").iter() {
            assert_eq!(*v, 1);
        }

        // Case 2: Need exactly 7 ACK
        let cmd = Command::new(0, Action::Register);
        client.database.add_request(cmd.id().clone()).unwrap();

        let mut i = 0;
        let mut feedback: Feedback = Feedback::Error(String::new());
        while {
            match timeout(Duration::from_millis(100), tx.recv()).await {
            Ok(v) => {feedback = v.unwrap(); false},
            Err(_) => true,
        }} {
            sender1.spawn_send(client_id.clone(), Message::CommandAcknowledgement(cmd.clone(), 0, CommandResult::Success(None), Phase::ACK), &fuse);
            let (id, message, ack) = timeout(Duration::from_millis(100), receiver.receive()).await.expect("Timeout");
            client.handle_message(id, message, ack).await;
            i += 1;
        }

        assert_eq!(i, network_info.n_ack());
        assert_eq!(feedback, Feedback::Result(CommandResult::Success(None)));

        // Needs exactly 2 + 1 CHK
        let cmd = Command::new(0, Action::Register);
        client.database.add_request(cmd.id().clone()).unwrap();

        let mut i = 0;
        while client.database.contains_request(cmd.id()) {
            sender1.spawn_send(client_id.clone(), Message::CommandAcknowledgement(cmd.clone(), 0, CommandResult::Success(None), Phase::CHK), &fuse);
            let (id, message, ack) = timeout(Duration::from_millis(100), receiver.receive()).await.expect("Timeout");
            client.handle_message(id, message, ack).await;
            i += 1;
        }

        assert_eq!(i, network_info.f() + 1);
        let res = timeout(Duration::from_secs(1), tx.recv()).await.expect("Timeout fb").unwrap();
        assert_eq!(res, Feedback::Result(CommandResult::Success(None)));
    }

}
