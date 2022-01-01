use std::{
    collections::BTreeSet,
    time::{Duration},
};

use talk::{crypto::Identity, time::timeout, unicast::Acknowledger};
use tokio::sync::broadcast::error::RecvError;

use crate::{
    banking::banking::Banking,
    banking::transaction::Transaction,
    database::{
        replica_database::{ReplicaDatabase, Set},
    },
    error::BankingError,
    network::NetworkInfo,
    peer::{
        coordinator::{ProposalData, ProposalSignedData},
        peer::PeerId,
    },
    talk::{Command, CommandResult, Instruction, Message, Phase, RoundNumber},
    types::*,
};

use super::{communicator::Communicator, Handler};

pub struct ReplicaHandler {
    communicator: Communicator<Message>,
    proposal_inlet: MPSCSender<ProposalSignedData>,
    proposal_outlet: BroadcastReceiver<ProposalData>,
    database: ReplicaDatabase,
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
            banking: Banking::new(),
        }
    }

    fn handle_command(&mut self, command: Command) {
        self.database.receive_command(command);
    }

    /// Implements task 1b and 1c
    fn handle_replica_broadcast(
        &mut self,
        round: RoundNumber,
        set: BTreeSet<Command>,
        phase: Phase,
    ) {
        if round.eq(self.database.round()) {
            match phase {
                Phase::ACK => self.database.receive_set(&set),
                Phase::CHK => self.database.receive_set(&set),
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
            if !Self::is_there_conflict(&received_diff_delivered, &received_diff_delivered) {
                let unprocessed_commands = unprocessed_commands.into_iter();
                for command in unprocessed_commands {
                    let result = self.execute(&command);
                    self.database.add_result(command.clone(), result.clone());
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
                        self.database.remove_result(&command);
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
                            .unwrap_or(self.execute(&command));

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
                .spawn_send(
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

    async fn propose(&mut self, data: ProposalSignedData) -> Result<ProposalData, RecvError> {
        self.proposal_inlet.send(data).await.unwrap();
        self.proposal_outlet.recv().await
    }

    async fn broadcast_to_replicas(&self, set: Set, phase: Phase) {
        let message = Message::ReplicaBroadcast(*self.database.round(), set, phase);
        let replicas = self.communicator.identity_table().replicas();
        for replica in replicas {
            if !self.communicator.key().eq(replica) {
                self.communicator
                    .spawn_send(replica.clone(), message.clone())
                    .await;
            }
        }
    }

    fn is_there_conflict(set1: &Set, set2: &Set) -> bool {
        let set1 = set1.into_iter();
        let mut set2 = set2.into_iter();
        for elem in set1 {
            if !set2.all(|item| !item.conflict(elem)) {
                return true;
            }
        }

        return false;
    }

    /// Execute the given command, and stores the transaction in the log.
    /// Returns the result
    fn execute(&mut self, command: &Command) -> CommandResult {
        let action = command.action();
        let id = *self.communicator.id();
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
                .map(|res| CommandResult::Success(Some(res)))
                .unwrap_or(CommandResult::Failure(format!(
                    "Client #{} cannot deposit because he is not registered",
                    self.communicator.id()
                ))),
            crate::banking::action::Action::Withdraw(amount) => self
                .banking
                .withdraw(&id, *amount)
                .map(|amount| CommandResult::Success(Some(amount)))
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
        return result;
    }

    fn rollback(&mut self, command: &Command) -> Result<(), BankingError> {
        let action = command.action();
        let id = self.communicator.id();
        let banking = &mut self.banking;
        let speculative_result = self
            .database
            .results()
            .get(command)
            .map(|result| {
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
            .unwrap_or(Err(BankingError::UnsufficientBalance));
        for transaction in self.database.log_mut().iter_mut() {
            if transaction.id().eq(command.id()) {
                transaction.rollback();
                break;
            }
        }
        speculative_result
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
    use crate::banking::action::Action;

    use super::*;

    #[test]
    fn borrow_test_checker() {
        let mut set: Set = BTreeSet::new();
        for _ in 0..15 {
            set.insert(Command::new(0, Action::Register));
        }

        ReplicaHandler::is_there_conflict(&set, &set);
    }
}
