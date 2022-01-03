use std::{collections::BTreeSet, fs::File, io::Write, path::Path, time::Duration};

use talk::{crypto::Identity, unicast::Acknowledger};
use tokio::sync::broadcast::error::RecvError;
use tokio::time::timeout;

use crate::{
    banking::banking::Banking,
    banking::transaction::Transaction,
    database::replica_database::{ReplicaDatabase, Set},
    error::BankingError,
    network::NetworkInfo,
    peer::{
        coordinator::{ProposalData, ProposalSignedData},
        peer::PeerId,
        shutdownable::Shutdownable,
    },
    relation::{conflict::ConflictingRelation, Relation},
    talk::{Command, CommandResult, Instruction, Message, Phase, RoundNumber},
    types::*,
};

use super::{communicator::Communicator, Handler};

pub struct ReplicaHandler {
    communicator: Communicator<Message>,
    proposal_inlet: MPSCSender<ProposalSignedData>,
    proposal_outlet: BroadcastReceiver<ProposalData>,
    database: ReplicaDatabase,
    received_to_resolve: Set, // Cache the received commands that need to resolve the conflicting relation
    // If a command is not in this set, it does not conflict with any other command in received \ delivered.
    banking: Banking,
}

impl ReplicaHandler {
    pub fn new(
        communicator: Communicator<Message>,
        proposal_inlet: MPSCSender<ProposalSignedData>,
        proposal_outlet: BroadcastReceiver<ProposalData>,
    ) -> Self {
        ReplicaHandler {
            communicator,
            proposal_inlet,
            proposal_outlet,
            database: ReplicaDatabase::new(),
            received_to_resolve: BTreeSet::new(),
            banking: Banking::new(),
        }
    }

    fn handle_command(&mut self, command: Command) {
        if self.database.receive_command(command.clone()) {
            if !self.database.delivered().contains(&command) {
                self.received_to_resolve.insert(command);
            }
        }
    }

    /// Implements task 1b and 1c
    fn handle_replica_broadcast(
        &mut self,
        round: RoundNumber,
        mut set: BTreeSet<Command>,
        phase: Phase,
    ) {
        if round.eq(self.database.round()) {
            for cmd in set.iter() {
                if !self.database.delivered().contains(cmd) {
                    self.received_to_resolve.insert(cmd.clone());
                }
            }
            match phase {
                Phase::ACK => self.database.receive_set(&mut set),
                Phase::CHK => self.database.receive_set(&mut set),
            }
        }
    }

    /// Returns true if there are new command to process
    /// This corresponds to the condition to enter task 2
    fn is_pending(unprocessed_commands: &Set) -> bool {
        !unprocessed_commands.is_empty()
    }

    fn compute_unprocessed_commands(&self) -> (Set, Set) {
        let received = self.database.received();
        let g_del = self.database.delivered();
        let pending_k = self.database.pending();

        let received_minus_del: BTreeSet<Command> = received.difference(g_del).cloned().collect();
        let new_commands: BTreeSet<Command> =
            received_minus_del.difference(pending_k).cloned().collect();

        (new_commands, received_minus_del)
    }

    /// Defines task 2
    async fn process_commands(&mut self) {
        let (unprocessed_commands, received_diff_delivered) = self.compute_unprocessed_commands();
        if Self::is_pending(&unprocessed_commands) {
            if !self.is_there_conflict(&received_diff_delivered) {
                self.received_to_resolve.clear();
                let unprocessed_commands = unprocessed_commands.into_iter();
                for command in unprocessed_commands {
                    let result = self.execute(&command);
                    self.acknowledge_client(command, result, Phase::ACK).await;
                }

                self.database.set_pending(received_diff_delivered);
                self.broadcast_to_replicas(self.database.pending().clone(), Phase::ACK)
                    .await;
            } else {
                self.broadcast_to_replicas(received_diff_delivered, Phase::CHK)
                    .await;
                let (k, nc_set, c_set) = self
                    .propose((
                        self.communicator.key().clone(),
                        *self.database.round(),
                        self.database.pending().clone(),
                        unprocessed_commands,
                    ))
                    .await
                    .expect("Fails to unwrap the proposal");
                if k.eq(self.database.round()) {
                    let pending = self.database.pending();
                    let pending_diff_nc_set: Set = pending.difference(&nc_set).cloned().collect();
                    let pending_diff_nc_set = pending_diff_nc_set.into_iter();

                    for command in pending_diff_nc_set {
                        self.rollback(&command).expect("Rollback failed");
                    }

                    let nc_set_diff_delivered: Set = nc_set
                        .difference(self.database.delivered())
                        .cloned()
                        .collect();
                    let nc_set_diff_delivered = nc_set_diff_delivered.into_iter();

                    for command in nc_set_diff_delivered {
                        let result = self
                            .database
                            .results_mut()
                            .remove(&command)
                            .unwrap_or_else(|| self.execute(&command));

                        self.acknowledge_client(command, result, Phase::CHK).await;
                    }

                    let mut c_set_ordered: Vec<Command> = c_set
                        .difference(self.database.delivered())
                        .cloned()
                        .collect();
                    c_set_ordered.sort();

                    let c_set_ordered = c_set_ordered.into_iter();

                    for command in c_set_ordered {
                        let result = self.execute(&command);
                        self.acknowledge_client(command, result, Phase::CHK).await;
                    }

                    self.database.delivered_all(&nc_set);
                    self.database.delivered_all(&c_set);

                    self.database.increment_round();
                    self.database.reset_pending();
                    self.database.reset_result();
                    self.received_to_resolve.clear();
                }
            }
        }
    }

    async fn acknowledge_client(
        &self,
        command: Command,
        command_result: CommandResult,
        phase: Phase,
    ) -> bool {
        if let Some(key) = self
            .communicator
            .identity_table()
            .get_client_id(*command.issuer())
        {
            self.communicator
                .spawn_send_message(
                    key.clone(),
                    Message::CommandAcknowledgement(
                        command,
                        *self.database.round(),
                        command_result,
                        phase,
                    ),
                )
                .await;

            return true;
        }
        false
    }

    /// Warning: It will block if mutliple replicas are spawned on the same thread
    async fn propose(&mut self, data: ProposalSignedData) -> Result<ProposalData, RecvError> {
        self.proposal_inlet.send(data).await.unwrap();
        self.proposal_outlet.recv().await
    }

    async fn broadcast_to_replicas(&self, set: Set, phase: Phase) {
        let message = Message::ReplicaBroadcast(*self.database.round(), set, phase);
        let replicas = self.communicator.identity_table().replicas();
        for replica in replicas.iter() {
            if !self.communicator.key().eq(replica) {
                self.communicator
                    .spawn_send_message(replica.clone(), message.clone())
                    .await;
            }
        }
    }

    fn is_there_conflict(&self, set: &Set) -> bool {
        let mut set2 = set.into_iter();
        for e1 in self.received_to_resolve.iter() {
            if set2.any(|e2| ConflictingRelation::is_related(&e1, e2)) {
                return true;
            }
        }
        false
    }

    /// Execute the given command, and stores the transaction in the log.
    /// Returns the result
    fn execute(&mut self, command: &Command) -> CommandResult {
        //println!("Execute {:?}", command);
        let action = command.action();
        let id = *command.issuer();
        let result = match action {
            crate::banking::action::Action::Register => {
                if self.banking.register(id) {
                    CommandResult::Success(None)
                } else {
                    CommandResult::Failure(format!("Client #{} is already registered", id))
                }
            }
            crate::banking::action::Action::Get => self
                .banking
                .get(&id)
                .map(|amount| CommandResult::Success(Some(amount)))
                .unwrap_or(CommandResult::Failure(format!(
                    "Client #{} is not registered",
                    id
                ))),
            crate::banking::action::Action::Deposit(amount) => self
                .banking
                .deposit(&id, *amount)
                .map(|_res| CommandResult::Success(None))
                .unwrap_or(CommandResult::Failure(format!(
                    "Client #{} cannot deposit because he is not registered",
                    self.communicator.id()
                ))),
            crate::banking::action::Action::Withdraw(amount) => self
                .banking
                .withdraw(&id, *amount)
                .map(|_amount| CommandResult::Success(None))
                .unwrap_or_else(|err| match err {
                    BankingError::ClientNotFound => {
                        CommandResult::Failure(format!("Client #{} is not registered", id))
                    }
                    BankingError::UnsufficientBalance => {
                        CommandResult::Failure(format!("Unsufficient balance"))
                    }
                }),
        };
        self.database
            .log(Transaction::from_command(command, &result));
        self.database.add_result(command.clone(), result.clone());
        return result;
    }

    fn rollback(&mut self, command: &Command) -> Result<(), BankingError> {
        //println!("Rollback: {:?}", command);
        let action = command.action();
        let id = command.issuer();
        let banking = &mut self.banking;
        let speculative_result = self
            .database
            .remove_result(&command)
            .map(|result| {
                //println!("Rollback successful");
                match result {
                    // Only rollback the effect if the command was successful
                    CommandResult::Success(_) => match action {
                        crate::banking::action::Action::Register => {
                            banking.unregister(id);
                            Ok(())
                        }
                        crate::banking::action::Action::Get => Ok(()),
                        crate::banking::action::Action::Deposit(amount) => {
                            banking.withdraw(id, *amount).map(|_| ())
                        }
                        crate::banking::action::Action::Withdraw(amount) => {
                            banking.deposit(id, *amount).map(|_| ())
                        }
                    },
                    CommandResult::Failure(_) => Ok(()),
                }
            })
            .unwrap_or(Err(BankingError::ClientNotFound));
        for transaction in self.database.logs_mut().iter_mut() {
            if transaction.id().eq(command.id()) {
                transaction.rollback();
                break;
            }
        }
        speculative_result
    }

    pub fn write_logs(&self) {
        let path = format!(
            "{}/log_replica{}.txt",
            self.network_info().report_folder(),
            self.communicator.id()
        );
        let path = Path::new(&path);
        let display = path.display();

        // Open a file in write-only mode, returns `io::Result<File>`
        let mut file = match File::create(&path) {
            Err(why) => panic!("Couldn't create {}: {}", display, why),
            Ok(file) => file,
        };
        for transaction in self.database.logs().iter() {
            write!(file, "{} \n", transaction).expect("Cannot write logs");
        }
        write!(file, "{:#?} \n", self.banking.clients()).expect("Fails to write logs");

        println!(
            "[{:#?}] #{} wrote logs",
            self.network_info().elapsed().unwrap(),
            self.id()
        )
    }
}

#[async_trait::async_trait]
impl Handler<Message> for ReplicaHandler {
    async fn handle_message(&mut self, _id: Identity, message: Message, _ack: Acknowledger) {
        match message {
            Message::Testing => {
                println!("Replica #{} received the test", self.communicator.id())
            }
            Message::Command(command) => self.handle_command(command),
            Message::ReplicaBroadcast(k, set, phase) => {
                self.handle_replica_broadcast(k, set, phase)
            }
            _ => {}
        }
        let process = self.process_commands();
        let process = timeout(Duration::from_secs(10), process);
        process.await.unwrap();
    }
    async fn handle_instruction(&mut self, instruction: Instruction) {
        match instruction {
            Instruction::Testing => {
                println!("Replica #{} received the test", self.communicator.id())
            }
            Instruction::Shutdown => {
                self.write_logs();
                self.communicator.shutdown().await;
            }
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
    use talk::unicast::test::UnicastSystem;

    use crate::{
        banking::action::Action,
        crypto::identity_table::IdentityTableBuilder,
        peer::{coordinator::Coordinator, handler::ClientHandler},
        talk::FeedbackChannel,
        tests::util::Utils,
    };

    use super::*;

    #[tokio::test]
    async fn correctly_receives_incoming_commands() {
        let network_info = NetworkInfo::with_default_report_folder(0, 3, 0, 0, 10, 1);
        let (mut keys, mut senders, mut receivers) = Utils::mock_network(3).await;
        let (replica3, _sender3, mut _receiver3) =
            Utils::pop(&mut keys, &mut senders, &mut receivers);
        let (replica1, sender1, mut _receiver1) =
            Utils::pop(&mut keys, &mut senders, &mut receivers);
        let (replica2, _sender2, mut _receiver2) =
            Utils::pop(&mut keys, &mut senders, &mut receivers);

        let (rx1, mut _tx1) = FeedbackChannel::channel();

        let coordinator = Coordinator::new(network_info.clone());

        let identity_table = IdentityTableBuilder::new(network_info.clone())
            .add_peer(replica1.clone())
            .add_peer(replica2.clone())
            .add_peer(replica3.clone())
            .build();
        let mut rh1 = ReplicaHandler::new(
            Communicator::new(
                0,
                replica1.clone(),
                sender1,
                rx1,
                network_info.clone(),
                identity_table.clone(),
            ),
            coordinator.proposer(),
            coordinator.subscribe(),
        );

        let cmd = Command::new(0, Action::Register);
        rh1.handle_command(cmd.clone());

        assert_eq!(rh1.database.received().contains(&cmd), true);
    }

    #[tokio::test]
    async fn correctly_receives_broadcast() {
        /* Template for a Network of 3 replicas */
        let network_info = NetworkInfo::with_default_report_folder(0, 3, 0, 0, 10, 1);
        let (mut keys, mut senders, mut receivers) = Utils::mock_network(3).await;
        let (replica3, sender3, mut _receiver3) =
            Utils::pop(&mut keys, &mut senders, &mut receivers);
        let (replica1, sender1, mut _receiver1) =
            Utils::pop(&mut keys, &mut senders, &mut receivers);
        let (replica2, sender2, mut _receiver2) =
            Utils::pop(&mut keys, &mut senders, &mut receivers);

        let (rx1, mut _tx1) = FeedbackChannel::channel();
        let (rx2, mut _tx2) = FeedbackChannel::channel();
        let (rx3, mut _tx3) = FeedbackChannel::channel();

        let coordinator = Coordinator::new(network_info.clone());

        let identity_table = IdentityTableBuilder::new(network_info.clone())
            .add_peer(replica1.clone())
            .add_peer(replica2.clone())
            .add_peer(replica3.clone())
            .build();
        let mut rh1 = ReplicaHandler::new(
            Communicator::new(
                0,
                replica1.clone(),
                sender1,
                rx1,
                network_info.clone(),
                identity_table.clone(),
            ),
            coordinator.proposer(),
            coordinator.subscribe(),
        );

        let mut rh2 = ReplicaHandler::new(
            Communicator::new(
                0,
                replica2.clone(),
                sender2,
                rx2,
                network_info.clone(),
                identity_table.clone(),
            ),
            coordinator.proposer(),
            coordinator.subscribe(),
        );

        let mut _rh3 = ReplicaHandler::new(
            Communicator::new(
                0,
                replica3.clone(),
                sender3,
                rx3,
                network_info.clone(),
                identity_table.clone(),
            ),
            coordinator.proposer(),
            coordinator.subscribe(),
        );
        /* ______________________ */

        let cmd1 = Command::new(4, Action::Register);
        let cmd2 = Command::new(5, Action::Register);
        let mut set: Set = BTreeSet::new();
        set.insert(cmd1.clone());
        set.insert(cmd2.clone());

        rh1.handle_replica_broadcast(*rh1.database.round(), set.clone(), Phase::ACK);

        for cmd in set.iter() {
            assert_eq!(rh1.database.received().contains(cmd), true);
        }

        rh2.handle_replica_broadcast(12345, set.clone(), Phase::ACK);

        for cmd in set.iter() {
            assert_eq!(rh2.database.received().contains(cmd), false);
        }
    }

    #[tokio::test]
    async fn compute_correctly_unprocessed_commands() {
        let network_info = NetworkInfo::with_default_report_folder(0, 3, 0, 0, 10, 1);
        let (mut keys, mut senders, mut receivers) = Utils::mock_network(3).await;
        let (replica3, sender3, mut _receiver3) =
            Utils::pop(&mut keys, &mut senders, &mut receivers);
        let (replica1, sender1, mut _receiver1) =
            Utils::pop(&mut keys, &mut senders, &mut receivers);
        let (replica2, sender2, mut _receiver2) =
            Utils::pop(&mut keys, &mut senders, &mut receivers);

        let (rx1, mut _tx1) = FeedbackChannel::channel();
        let (rx2, mut _tx2) = FeedbackChannel::channel();
        let (rx3, mut _tx3) = FeedbackChannel::channel();

        let coordinator = Coordinator::new(network_info.clone());

        let identity_table = IdentityTableBuilder::new(network_info.clone())
            .add_peer(replica1.clone())
            .add_peer(replica2.clone())
            .add_peer(replica3.clone())
            .build();
        let mut rh1 = ReplicaHandler::new(
            Communicator::new(
                0,
                replica1.clone(),
                sender1,
                rx1,
                network_info.clone(),
                identity_table.clone(),
            ),
            coordinator.proposer(),
            coordinator.subscribe(),
        );

        let mut _rh2 = ReplicaHandler::new(
            Communicator::new(
                0,
                replica2.clone(),
                sender2,
                rx2,
                network_info.clone(),
                identity_table.clone(),
            ),
            coordinator.proposer(),
            coordinator.subscribe(),
        );

        let mut _rh3 = ReplicaHandler::new(
            Communicator::new(
                0,
                replica3.clone(),
                sender3,
                rx3,
                network_info.clone(),
                identity_table.clone(),
            ),
            coordinator.proposer(),
            coordinator.subscribe(),
        );

        let cmd1 = Command::new(5, Action::Register);
        let cmd2 = Command::new(4, Action::Get);
        let cmd3 = Command::new(5, Action::Deposit(0));
        let cmd4 = Command::new(6, Action::Deposit(1));
        let cmd5 = Command::new(7, Action::Deposit(2));
        let cmd6 = Command::new(8, Action::Withdraw(6));

        let vec = vec![
            cmd1.clone(),
            cmd2.clone(),
            cmd3.clone(),
            cmd4.clone(),
            cmd5.clone(),
            cmd6.clone(),
        ];
        let db = &mut rh1.database;

        for cmd in vec.iter() {
            db.received_mut().insert(cmd.clone());
        }

        db.delivered_mut().insert(cmd1.clone());
        db.delivered_mut().insert(cmd2.clone());

        db.pending_mut().insert(cmd3.clone());
        db.pending_mut().insert(cmd4.clone());

        let db = &rh1.database;
        let (unprocessed, received_dif_del) = rh1.compute_unprocessed_commands();

        assert_eq!(unprocessed.contains(&cmd5), true);
        assert_eq!(unprocessed.contains(&cmd6), true);
        assert_eq!(unprocessed.contains(&cmd4), false);

        for cmd in vec.iter() {
            if !db.delivered().contains(cmd) {
                assert_eq!(received_dif_del.contains(cmd), true);
            } else {
                assert_eq!(received_dif_del.contains(cmd), false);
            }
        }
    }

    #[tokio::test]
    async fn correctly_acknowledge_client() {
        let network_info = NetworkInfo::with_default_report_folder(1, 2, 0, 0, 10, 1);
        let (mut keys, mut senders, mut receivers) = Utils::mock_network(3).await;
        let (replica3, sender3, mut _receiver3) =
            Utils::pop(&mut keys, &mut senders, &mut receivers);
        let (replica1, sender1, mut receiver1) =
            Utils::pop(&mut keys, &mut senders, &mut receivers);
        let (replica2, sender2, mut _receiver2) =
            Utils::pop(&mut keys, &mut senders, &mut receivers);

        let (rx1, mut _tx1) = FeedbackChannel::channel();
        let (rx2, mut _tx2) = FeedbackChannel::channel();
        let (rx3, mut _tx3) = FeedbackChannel::channel();

        let coordinator = Coordinator::new(network_info.clone());

        let identity_table = IdentityTableBuilder::new(network_info.clone())
            .add_peer(replica1.clone())
            .add_peer(replica2.clone())
            .add_peer(replica3.clone())
            .build();
        let mut _client = ClientHandler::new(Communicator::new(
            0,
            replica1.clone(),
            sender1,
            rx1,
            network_info.clone(),
            identity_table.clone(),
        ));

        let rh2 = ReplicaHandler::new(
            Communicator::new(
                1,
                replica2.clone(),
                sender2,
                rx2,
                network_info.clone(),
                identity_table.clone(),
            ),
            coordinator.proposer(),
            coordinator.subscribe(),
        );

        let mut _rh3 = ReplicaHandler::new(
            Communicator::new(
                2,
                replica3.clone(),
                sender3,
                rx3,
                network_info.clone(),
                identity_table.clone(),
            ),
            coordinator.proposer(),
            coordinator.subscribe(),
        );

        let cmd = Command::new(0, Action::Register);
        let result = CommandResult::Success(None);
        assert_eq!(
            timeout(
                Duration::from_secs(1),
                rh2.acknowledge_client(cmd.clone(), result.clone(), Phase::ACK)
            )
            .await
            .unwrap(),
            true
        );

        let (id, msg, _) = timeout(Duration::from_secs(1), receiver1.receive())
            .await
            .unwrap();
        assert_eq!(
            msg,
            Message::CommandAcknowledgement(cmd.clone(), 1, result.clone(), Phase::ACK)
        );
        assert_eq!(id, replica2.clone());

        let cmd2 = Command::new(35, Action::Get);
        assert_eq!(
            timeout(
                Duration::from_secs(1),
                rh2.acknowledge_client(cmd2.clone(), result.clone(), Phase::ACK)
            )
            .await
            .unwrap(),
            false
        );
    }

    #[tokio::test]
    async fn correctly_recover_consensus() {
        let network_info = NetworkInfo::with_default_report_folder(1, 3, 2, 0, 10, 3);
        let mut mock_network = UnicastSystem::<Message>::setup(6).await;

        let (client1, client_sender1, mut _client_receiver1) =
            Utils::pop_from_network(&mut mock_network);
        let (replica2, replica_sender2, mut _replica_receiver2) =
            Utils::pop_from_network(&mut mock_network);
        let (replica1, _replica_sender1, mut _replica_receiver1) =
            Utils::pop_from_network(&mut mock_network);
        let (replica3, replica_sender3, mut _replica_receiver3) =
            Utils::pop_from_network(&mut mock_network);
        // Faulty replica doesn't send anything
        let (rx1, mut _tx1) = FeedbackChannel::channel();
        let (rx2, mut _tx2) = FeedbackChannel::channel();
        let (rx3, mut _tx3) = FeedbackChannel::channel();
        let (_rx4, mut _tx4) = FeedbackChannel::channel();

        let coordinator = Coordinator::new(network_info.clone());

        let identity_table = IdentityTableBuilder::new(network_info.clone())
            .add_peer(replica1.clone())
            .add_peer(replica2.clone())
            .add_peer(replica3.clone())
            .build();
        let mut _client = ClientHandler::new(Communicator::new(
            0,
            client1.clone(),
            client_sender1,
            rx1,
            network_info.clone(),
            identity_table.clone(),
        ));

        let mut _rh2 = ReplicaHandler::new(
            Communicator::new(
                1,
                replica2.clone(),
                replica_sender2,
                rx2,
                network_info.clone(),
                identity_table.clone(),
            ),
            coordinator.proposer(),
            coordinator.subscribe(),
        );

        let mut _rh3 = ReplicaHandler::new(
            Communicator::new(
                2,
                replica3.clone(),
                replica_sender3,
                rx3,
                network_info.clone(),
                identity_table.clone(),
            ),
            coordinator.proposer(),
            coordinator.subscribe(),
        );
        // Replica handlers must be runned on different threads
    }
    #[tokio::test]
    async fn process_non_conflicting_commands_correctly() {
        let network_info = NetworkInfo::with_default_report_folder(1, 3, 0, 2, 10, 3);
        let mut mock_network = UnicastSystem::<Message>::setup(6).await;
        let (client1, _cli_ent_sender1, mut client_receiver1) =
            Utils::pop_from_network(&mut mock_network);
        let (replica2, _replica_sender2, mut replica_receiver2) =
            Utils::pop_from_network(&mut mock_network);
        let (replica1, replica_sender1, mut replica_receiver1) =
            Utils::pop_from_network(&mut mock_network);
        let (replica3, _replica_sender3, mut replica_receiver3) =
            Utils::pop_from_network(&mut mock_network);
        let (rx1, mut _tx1) = FeedbackChannel::channel();

        println!(
            "Client: {:#?} \n R1: {:#?} \n R2: {:#?}, \n, R3: {:#?} \n",
            client1, replica1, replica2, replica3
        );

        let coordinator = Coordinator::new(network_info.clone());

        let identity_table = IdentityTableBuilder::new(network_info.clone())
            .add_peer(client1.clone())
            .add_peer(replica1.clone())
            .add_peer(replica2.clone())
            .add_peer(replica3.clone())
            .build();
        println!("TABLE: {:#?}", identity_table);

        let mut rh1 = ReplicaHandler::new(
            Communicator::new(
                1,
                replica1.clone(),
                replica_sender1,
                rx1,
                network_info.clone(),
                identity_table.clone(),
            ),
            coordinator.proposer(),
            coordinator.subscribe(),
        );

        // Replica handlers must be runned on different threads

        let cmd1 = Command::new(0, Action::Deposit(1));
        let cmd2 = Command::new(0, Action::Deposit(2));
        let cmd3 = Command::new(0, Action::Register);
        let cmd4 = Command::new(0, Action::Deposit(4));
        rh1.database.received_mut().insert(cmd1.clone());
        rh1.database.received_mut().insert(cmd2.clone());
        rh1.database.received_mut().insert(cmd3.clone());
        rh1.database.received_mut().insert(cmd4.clone());

        rh1.database.delivered_mut().insert(cmd3.clone());
        rh1.database.pending_mut().insert(cmd4.clone());

        rh1.banking.register(0);
        rh1.banking.deposit(&0, 4).expect("Failed to execute cmd 4");
        rh1.database.increment_round();
        rh1.database.increment_round();

        if let Err(_) = timeout(Duration::from_secs(3), rh1.process_commands()).await {
            panic!("Failed to process commands");
        }

        let mut i = 0;
        while let Ok((_, Message::CommandAcknowledgement(_, k, res, phase), _)) =
            timeout(Duration::from_secs(1), client_receiver1.receive()).await
        {
            assert_eq!(phase, Phase::ACK);
            assert_eq!(k, 3);
            if let CommandResult::Success(_) = res {
            } else {
                panic!("Unvalid resut");
            }

            i += 1;
        }

        assert_eq!(i, 2);

        // Replica must receives the broadcast
        let mut set2: Set = BTreeSet::new();
        set2.insert(cmd1.clone());
        set2.insert(cmd2.clone());
        set2.insert(cmd4.clone());
        let broadcast = Message::ReplicaBroadcast(3, set2, Phase::ACK);
        let (_, recv, _) = timeout(Duration::from_secs(10), replica_receiver2.receive())
            .await
            .expect("Replica 2 doesn't receive the broadcast");
        assert_eq!(broadcast, recv);
        let (_, recv, _) = timeout(Duration::from_secs(10), replica_receiver3.receive())
            .await
            .expect("Replica 2 doesn't receive the broadcast");
        assert_eq!(broadcast, recv);

        // Must not receive more
        let res = timeout(Duration::from_secs(1), replica_receiver2.receive()).await;
        if let Ok(_) = res {
            panic!("Replica 2 should only receive one msg");
        }
        let res = timeout(Duration::from_secs(1), replica_receiver1.receive()).await;
        if let Ok(_) = res {
            panic!("Replica 1 should not receive any msg");
        }

        let res = timeout(Duration::from_secs(1), replica_receiver3.receive()).await;
        if let Ok(_) = res {
            panic!("Replica 3 should only receive one msg");
        }

        assert_eq!(rh1.database.results().contains_key(&cmd1), true);
        assert_eq!(rh1.database.results().contains_key(&cmd2), true);

        assert_eq!(rh1.banking.get(&0), Some(7));
    }

    #[tokio::test]
    async fn is_conflict_working() {
        /*
        let mut set: Set = BTreeSet::new();

        let network_info = NetworkInfo::new(0, 1, 0, 0, 10, 3);
        let mut mock_network = UnicastSystem::<Message>::setup(6).await;
        let (replica1, replica_sender1, mut _replica_receiver1) =
            Utils::pop_from_network(&mut mock_network);
        let (rx1, mut _tx1) = FeedbackChannel::channel();
        let coordinator = Coordinator::new(network_info.clone());

        let identity_table = IdentityTableBuilder::new(network_info.clone())
            .add_peer(replica1.clone())
            .build();
        println!("TABLE: {:#?}", identity_table);

        let mut replica = ReplicaHandler::new(
            Communicator::new(
                1,
                replica1.clone(),
                replica_sender1,
                rx1,
                network_info.clone(),
                identity_table.clone(),
            ),
            coordinator.proposer(),
            coordinator.subscribe(),
        );

        set.insert(Command::new(0, Action::Deposit(10)));
        set.insert(Command::new(1, Action::Get));
        replica.handle_replica_broadcast(0, set.clone(), Phase::ACK);

        assert_eq!(replica.is_there_conflict(&set), false);

        set.insert(Command::new(0, Action::Register));

        assert_eq!(ReplicaHandler::is_there_conflict(&set), true);
        set.insert(Command::new(0, Action::Withdraw(3)));

        assert_eq!(ReplicaHandler::is_there_conflict(&set), true);
        */
    }

    #[tokio::test]
    async fn execute_correctly() {
        let network_info = NetworkInfo::with_default_report_folder(1, 3, 0, 2, 10, 3);
        let mut mock_network = UnicastSystem::<Message>::setup(6).await;
        let (client1, _client_sender1, mut _client_receiver1) =
            Utils::pop_from_network(&mut mock_network);
        let (replica2, _replica_sender2, mut _replica_receiver2) =
            Utils::pop_from_network(&mut mock_network);
        let (replica1, replica_sender1, mut _replica_receiver1) =
            Utils::pop_from_network(&mut mock_network);
        let (replica3, _replica_sender3, mut _replica_receiver3) =
            Utils::pop_from_network(&mut mock_network);
        let (rx1, mut _tx1) = FeedbackChannel::channel();

        println!(
            "Client: {:#?} \n R1: {:#?} \n R2: {:#?}, \n, R3: {:#?} \n",
            client1, replica1, replica2, replica3
        );

        let coordinator = Coordinator::new(network_info.clone());

        let identity_table = IdentityTableBuilder::new(network_info.clone())
            .add_peer(client1.clone())
            .add_peer(replica1.clone())
            .add_peer(replica2.clone())
            .add_peer(replica3.clone())
            .build();
        println!("TABLE: {:#?}", identity_table);

        let mut replica = ReplicaHandler::new(
            Communicator::new(
                1,
                replica1.clone(),
                replica_sender1,
                rx1,
                network_info.clone(),
                identity_table.clone(),
            ),
            coordinator.proposer(),
            coordinator.subscribe(),
        );

        let registration = Command::new(0, Action::Register);
        let deposit = Command::new(0, Action::Deposit(10));
        let get = Command::new(0, Action::Get);
        let withdraw = Command::new(0, Action::Withdraw(5));

        if let CommandResult::Failure(_) = replica.execute(&withdraw) {
        } else {
            panic!("Withdraw should fail");
        }
        if let CommandResult::Failure(_) = replica.execute(&get) {
        } else {
            panic!("Get should fail: client is not registered");
        }
        if let CommandResult::Failure(_) = replica.execute(&deposit) {
        } else {
            panic!("Deposit should fail");
        }

        if let CommandResult::Success(_) = replica.execute(&registration) {
        } else {
            panic!("Registration should succeed");
        }

        if let CommandResult::Failure(_) = replica.execute(&registration) {
        } else {
            panic!("Client should already be registered");
        }

        if let CommandResult::Failure(_) = replica.execute(&withdraw) {
        } else {
            panic!("Client hsould not have enough money");
        }

        if let CommandResult::Success(_) = replica.execute(&deposit) {
        } else {
            panic!("Client should be able to deposit");
        }

        if let CommandResult::Success(_) = replica.execute(&withdraw) {
        } else {
            panic!("Client should withdraw 5");
        }

        for transaction in replica.database.logs_mut().iter() {
            println!("{}", transaction);
        }
    }

    #[tokio::test]
    async fn correctly_rollback_commands() {
        let network_info = NetworkInfo::with_default_report_folder(1, 3, 0, 2, 10, 3);
        let mut mock_network = UnicastSystem::<Message>::setup(6).await;
        let (client1, _client_sender1, mut _client_receiver1) =
            Utils::pop_from_network(&mut mock_network);
        let (replica2, _replica_sender2, mut _replica_receiver2) =
            Utils::pop_from_network(&mut mock_network);
        let (replica1, replica_sender1, mut _replica_receiver1) =
            Utils::pop_from_network(&mut mock_network);
        let (replica3, _replica_sender3, mut _replica_receiver3) =
            Utils::pop_from_network(&mut mock_network);
        let (rx1, mut _tx1) = FeedbackChannel::channel();

        println!(
            "Client: {:#?} \n R1: {:#?} \n R2: {:#?}, \n, R3: {:#?} \n",
            client1, replica1, replica2, replica3
        );

        let coordinator = Coordinator::new(network_info.clone());

        let identity_table = IdentityTableBuilder::new(network_info.clone())
            .add_peer(client1.clone())
            .add_peer(replica1.clone())
            .add_peer(replica2.clone())
            .add_peer(replica3.clone())
            .build();
        println!("TABLE: {:#?}", identity_table);

        let mut replica = ReplicaHandler::new(
            Communicator::new(
                1,
                replica1.clone(),
                replica_sender1,
                rx1,
                network_info.clone(),
                identity_table.clone(),
            ),
            coordinator.proposer(),
            coordinator.subscribe(),
        );

        let registration = Command::new(0, Action::Register);
        let deposit = Command::new(0, Action::Deposit(10));
        let _get = Command::new(0, Action::Get);
        let withdraw = Command::new(0, Action::Withdraw(5));

        let res = replica.execute(&registration);
        replica.database.add_result(registration.clone(), res);
        let __rollback = replica
            .rollback(&registration)
            .expect("Unregister should be successful");
        assert_eq!(replica.banking.get(&0), None);

        replica.banking.register(0);
        let res = replica.execute(&deposit);
        replica.database.add_result(deposit.clone(), res);
        replica.execute(&Command::new(1, Action::Deposit(10)));
        replica
            .rollback(&deposit)
            .expect("Deposit should be removed");

        assert_eq!(replica.banking.get(&0), Some(0));

        replica.banking.register(0);
        replica.banking.register(1);
        replica.banking.deposit(&1, 12).unwrap();
        replica.banking.deposit(&0, 10).unwrap();
        let res = replica.execute(&withdraw);
        replica.database.add_result(withdraw.clone(), res);
        replica
            .rollback(&withdraw)
            .expect("Rollback should succeed");

        assert_eq!(replica.banking.get(&0), Some(10));
        assert_eq!(replica.banking.get(&1), Some(12));
    }

    #[tokio::test]
    async fn correctly_execute_conflicting_messages() {
        let network_info = NetworkInfo::with_default_report_folder(2, 3, 0, 2, 10, 3);
        let mut mock_network = UnicastSystem::<Message>::setup(7).await;
        let (client1, _client_sender1, mut client_receiver1) =
            Utils::pop_from_network(&mut mock_network);
        let (client2, _client_sender2, mut client_receiver2) =
            Utils::pop_from_network(&mut mock_network);
        let (replica2, _replica_sender2, replica_receiver2) =
            Utils::pop_from_network(&mut mock_network);
        let (replica1, replica_sender1, _replica_receiver1) =
            Utils::pop_from_network(&mut mock_network);
        let (replica3, _replica_sender3, replica_receiver3) =
            Utils::pop_from_network(&mut mock_network);
        let (faulty_replica1, _faulty_replica_sender1, faulty_replica_receiver1) =
            Utils::pop_from_network(&mut mock_network);
        let (faulty_replica2, _faulty_replica_sender2, faulty_replica_receiver2) =
            Utils::pop_from_network(&mut mock_network);

        let (rx1, mut _tx1) = FeedbackChannel::channel();

        println!(
            "Client: {:#?} \n R1: {:#?} \n R2: {:#?}, \n, R3: {:#?} \n",
            client1, replica1, replica2, replica3
        );

        let mut coordinator = Coordinator::new(network_info.clone());

        let identity_table = IdentityTableBuilder::new(network_info.clone())
            .add_peer(client1.clone())
            .add_peer(client2.clone())
            .add_peer(replica1.clone())
            .add_peer(replica2.clone())
            .add_peer(replica3.clone())
            .add_peer(faulty_replica1.clone())
            .add_peer(faulty_replica2.clone())
            .build();
        println!("TABLE: {:#?}", identity_table);

        let mut replica = ReplicaHandler::new(
            Communicator::new(
                1,
                replica1.clone(),
                replica_sender1,
                rx1,
                network_info.clone(),
                identity_table.clone(),
            ),
            coordinator.proposer(),
            coordinator.subscribe(),
        );

        // We previously register different clients

        for i in 0..2 {
            replica.banking.register(i);
        }

        // Define the commands
        let mut cmds: Vec<Command> = Vec::new();
        let cmd1 = Command::new(0, Action::Deposit(1));
        let cmd2 = Command::new(1, Action::Deposit(2));
        let cmd3 = Command::new(0, Action::Deposit(3));
        let cmd4 = Command::new(0, Action::Deposit(4));
        let cmd5 = Command::new(1, Action::Deposit(5));
        let cmd6 = Command::new(1, Action::Deposit(6));
        let cmd7 = Command::new(0, Action::Deposit(7));
        let cmd8 = Command::new(1, Action::Deposit(8));
        let cmd9 = Command::new(0, Action::Get);
        let cmd10 = Command::new(1, Action::Deposit(10));
        let cmd11 = Command::new(0, Action::Deposit(11));
        let cmd12 = Command::new(1, Action::Deposit(12));
        let cmd13 = Command::new(0, Action::Withdraw(10));
        cmds.push(cmd1.clone());
        cmds.push(cmd2.clone());
        cmds.push(cmd3.clone());
        cmds.push(cmd4.clone());
        cmds.push(cmd5.clone());
        cmds.push(cmd6.clone());
        cmds.push(cmd7.clone());
        cmds.push(cmd8.clone());
        cmds.push(cmd9.clone());
        cmds.push(cmd10.clone());
        cmds.push(cmd11.clone());
        cmds.push(cmd12.clone());
        cmds.push(cmd13.clone());

        for cmd in cmds.iter() {
            replica.handle_command(cmd.clone());
            if cmd1.ne(cmd) && cmd8.ne(cmd) && cmd12.ne(cmd) && cmd13.ne(cmd) {
                replica.database.delivered_mut().insert(cmd.clone());
                replica.execute(cmd);
            }
        }

        replica.database.pending_mut().insert(cmd1.clone());
        replica.execute(&cmd1);
        replica.database.pending_mut().insert(cmd8.clone());
        replica.execute(&cmd8);
        replica.database.increment_round();
        replica.database.increment_round();

        let round: usize = 3;

        println!("Received : {:#?}", replica.database.received());
        println!("Delivered: {:#?}", replica.database.delivered());
        println!("Pending: : {:#?}", replica.database.pending());
        let (unprocessed, r_dif_g) = replica.compute_unprocessed_commands();
        println!("Unprocessed: {:#?}, dif: {:#?}", unprocessed, r_dif_g);

        assert_eq!(ReplicaHandler::is_pending(&unprocessed), true);
        //assert_eq!(ReplicaHandler::is_there_conflict(&r_dif_g), true);

        let mut nc_set = BTreeSet::<Command>::new();
        let mut c_set = BTreeSet::<Command>::new();
        nc_set.insert(cmd1.clone());
        nc_set.insert(cmd12.clone());
        c_set.insert(cmd8.clone());
        c_set.insert(cmd13.clone());
        // Spawn a coordinator, that broadcast a predefined result when he receives the message
        tokio::spawn(async move {
            if let Some(_) = coordinator.receiver().recv().await {
                coordinator
                    .broadcaster()
                    .send((3, nc_set.clone(), c_set.clone()))
                    .unwrap();
                println!("Coordinator completed his duty!")
            }
        });

        timeout(Duration::from_secs(10), replica.process_commands())
            .await
            .expect("Processing commands failed");

        let mut replicas = vec![
            replica_receiver2,
            replica_receiver3,
            faulty_replica_receiver1,
            faulty_replica_receiver2,
        ];
        let mut set_test = BTreeSet::<Command>::new();
        set_test.insert(cmd1.clone());
        set_test.insert(cmd8.clone());
        set_test.insert(cmd12.clone());
        set_test.insert(cmd13.clone());
        for replica in replicas.iter_mut() {
            let (_, msg, _) = timeout(Duration::from_secs(10), replica.receive())
                .await
                .expect("0ne replica did not receive the broadcast");
            match msg {
                Message::ReplicaBroadcast(k, set, phase) => {
                    assert_eq!(k, round);
                    assert_eq!(set, set_test);
                    assert_eq!(phase, Phase::CHK);
                }
                _ => panic!("Wrong broadcast"),
            }
        }

        let (_, msg, _) = timeout(Duration::from_secs(1), client_receiver1.receive())
            .await
            .expect("Client #0 fails");
        match msg {
            Message::CommandAcknowledgement(cmd, k, res, phase) => {
                if cmd.ne(&cmd1) && cmd.ne(&cmd13) {
                    panic!();
                }
                assert_eq!(k, round);
                assert_eq!(res, CommandResult::Success(None));
                assert_eq!(phase, Phase::CHK);
            }
            _ => panic!(),
        }
        let (_, msg, _) = timeout(Duration::from_secs(1), client_receiver2.receive())
            .await
            .expect("Client #0 fails");
        match msg {
            Message::CommandAcknowledgement(cmd, k, res, phase) => {
                if cmd.ne(&cmd8) && cmd.ne(&cmd12) {
                    panic!();
                }
                assert_eq!(k, round);
                assert_eq!(res, CommandResult::Success(None));
                assert_eq!(phase, Phase::CHK);
            }
            _ => panic!(),
        }

        let (_, msg, _) = timeout(Duration::from_secs(1), client_receiver2.receive())
            .await
            .expect("Client #0 fails");
        match msg {
            Message::CommandAcknowledgement(cmd, k, res, phase) => {
                if cmd.ne(&cmd8) && cmd.ne(&cmd12) {
                    panic!();
                }
                assert_eq!(k, round);
                assert_eq!(res, CommandResult::Success(None));
                assert_eq!(phase, Phase::CHK);
            }
            _ => panic!(),
        }

        let (_, msg, _) = timeout(Duration::from_secs(1), client_receiver1.receive())
            .await
            .expect("Client #0 fails");
        match msg {
            Message::CommandAcknowledgement(cmd, k, res, phase) => {
                if cmd.ne(&cmd1) && cmd.ne(&cmd13) {
                    panic!();
                }
                assert_eq!(k, round);
                assert_eq!(res, CommandResult::Success(None));
                assert_eq!(phase, Phase::CHK);
            }
            _ => panic!(),
        }

        assert_eq!(replica.database.delivered(), replica.database.received());
        assert_eq!(*replica.database.round(), 4);
        assert_eq!(replica.database.pending().is_empty(), true);
        assert_eq!(replica.database.results().is_empty(), true);

        for log in replica.database.logs() {
            println!("{}", log);
        }

        assert_eq!(replica.banking.get(&0), Some(16));
        assert_eq!(replica.banking.get(&1), Some(43));

        replica.write_logs();
    }
}
