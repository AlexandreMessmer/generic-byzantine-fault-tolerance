use std::collections::{BTreeSet, HashMap};

use crate::{
    banking::transaction::Transaction,
    talk::{Command, CommandResult},
};

pub type Set = BTreeSet<Command>;
pub type ResultBuffer = HashMap<Command, CommandResult>;
/// Defines the data structures for a `ReplicaHanlder`.
/// `received` contains all the messages broadcasted in the network so far.
/// `delivered` contains all the messages that have been delivered (i.e. validated) by the `Replica` in
/// the previous rounds.
/// `pending` defines the set of non-conflicting messages acknowledged by the replica in the current round.
/// `ack_delievered` defines the set of messages delivered in the ACK phase of the current round.
/// `round` is the current round.
pub struct ReplicaDatabase {
    received: Set,
    delivered: Set,
    pending: Set,
    results: ResultBuffer,
    round: usize,
    log: Vec<Transaction>,
}

impl ReplicaDatabase {
    pub fn new() -> Self {
        ReplicaDatabase {
            received: BTreeSet::new(),
            delivered: BTreeSet::new(),
            pending: BTreeSet::new(),
            results: HashMap::new(),
            round: 1,
            log: Vec::new(),
        }
    }

    /// Returns true if the value was not present
    pub fn receive_command(&mut self, command: Command) -> bool {
        self.received.insert(command)

    }

    pub fn receive_set(&mut self, set: &mut Set) {
        self.received.append(set)
    }

    pub fn delivered_all(&mut self, set: &Set) {
        self.delivered = self.delivered.union(&set.clone()).cloned().collect();
    }

    /// Set pending set to {set}
    pub fn set_pending(&mut self, set: Set) -> () {
        self.pending = set;
    }

    pub fn reset_pending(&mut self) {
        self.pending.clear();
    }

    pub fn reset_result(&mut self) {
        self.results.clear();
    }

    /// Returns false if a command was overwritten
    pub fn add_result(&mut self, cmd: Command, res: CommandResult) -> bool {
        self.results
            .insert(cmd, res)
            .map(|_previous| false)
            .unwrap_or(true)
    }

    pub fn remove_result(&mut self, cmd: &Command) -> Option<CommandResult> {
        self.results.remove(cmd)
    }

    pub fn received(&self) -> &Set {
        &self.received
    }

    pub fn delivered(&self) -> &Set {
        &self.delivered
    }

    pub fn pending(&self) -> &Set {
        &self.pending
    }

    pub fn pending_mut(&mut self) -> &mut Set {
        &mut self.pending
    }

    pub fn results(&self) -> &ResultBuffer {
        &self.results
    }

    pub fn round(&self) -> &usize {
        &self.round
    }

    pub fn increment_round(&mut self) {
        self.round += 1;
    }

    pub fn received_mut(&mut self) -> &mut Set {
        &mut self.received
    }

    pub fn delivered_mut(&mut self) -> &mut Set {
        &mut self.delivered
    }

    pub fn results_mut(&mut self) -> &mut ResultBuffer {
        &mut self.results
    }

    pub fn log(&mut self, transaction: Transaction) {
        self.log.push(transaction)
    }

    pub fn logs_mut(&mut self) -> &mut Vec<Transaction> {
        &mut self.log
    }

    pub fn logs(&self) -> &Vec<Transaction> {
        &self.log
    }
}

#[cfg(test)]
mod tests {

    use crate::banking::action::Action;

    use super::*;

    #[test]
    fn receive_command_test() {
        let mut db = ReplicaDatabase::new();
        let mut cmds: Vec<Command> = Vec::new();
        for _ in 0..10 {
            cmds.push(Command::new(0, Action::Register));
        }

        for cmd in cmds.clone().into_iter() {
            db.receive_command(cmd);
        }

        for cmd in cmds.into_iter() {
            assert_eq!(db.received.contains(&cmd), true);
        }
    }

    #[test]
    fn receive_set_test() {
        let mut db = ReplicaDatabase::new();
        let mut cmds: BTreeSet<Command> = BTreeSet::new();
        for _ in 0..10 {
            cmds.insert(Command::new(0, Action::Register));
        }
        let unique_cmd = Command::new(0, Action::Register);
        db.receive_command(unique_cmd.clone());
        db.receive_set(&mut cmds);

        for cmd in cmds.into_iter() {
            assert_eq!(db.received.contains(&cmd), true);
        }

        assert_eq!(db.received.contains(&unique_cmd), true);
    }

    #[test]
    fn result_buffer_remove_test() {
        let mut db = ReplicaDatabase::new();
        let cmd1 = Command::new(0, Action::Register);
        let cmd2 = Command::new(0, Action::Register);
        let cmd3 = Command::new(0, Action::Register);
        db.add_result(cmd1.clone(), CommandResult::Success(None));
        db.add_result(cmd2.clone(), CommandResult::Success(None));
        db.add_result(cmd3.clone(), CommandResult::Success(None));

        assert_eq!(db.results.contains_key(&cmd1), true);

        db.remove_result(&cmd1);

        assert_eq!(db.results.contains_key(&cmd1), false);
    }
}
