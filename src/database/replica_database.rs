use std::{collections::{BTreeMap, BTreeSet, HashMap}, ptr::eq};

use crate::talk::{Command, CommandResult, Message, RoundNumber, Phase};

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
    results_buffer: ResultBuffer,
    round: usize,
}

impl ReplicaDatabase {
    pub fn new() -> Self {
        let received = BTreeSet::new();
        let delivered = BTreeSet::new();
        let pending = BTreeSet::new();
        let results_buffer = HashMap::new();
        let round = 1;

        ReplicaDatabase {
            received,
            delivered,
            pending,
            results_buffer,
            round,
        }
    }

    /// Returns true if the value was not present
    pub fn receive_command(&mut self, command: Command) -> bool {
        self.received.insert(command)
    }

    pub fn receive_set(&mut self, set: &Set) {
        self.received = self.received.union(set).cloned().collect()
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
        self.results_buffer.clear();
    }
    
    pub fn add_result(&mut self, cmd: Command, res: CommandResult) {
        self.results_buffer.insert(cmd, res);
    }

    
    pub fn remove_result(&mut self, cmd: &Command) -> bool {
        self.results_buffer.remove(cmd).map(|_| true)
            .unwrap_or(false)
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

    pub fn results_buffer(&self) -> &ResultBuffer {
        &self.results_buffer
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

    pub fn results_buffer_mut(&mut self) -> &mut ResultBuffer {
        &mut self.results_buffer
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn receive_command_test() {
        let mut db = ReplicaDatabase::new();
        let mut cmds: Vec<Command> = Vec::new(); 
        for _ in 0..10 {
            cmds.push(Command::new());
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
            cmds.insert(Command::new());
        }
        let unique_cmd = Command::new();
        db.receive_command(unique_cmd.clone());
        db.receive_set(&cmds);

        for cmd in cmds.into_iter() {
            assert_eq!(db.received.contains(&cmd), true);
        }

        assert_eq!(db.received.contains(&unique_cmd), true);
    }

    #[test]
    fn result_buffer_remove_test() {
        let mut db = ReplicaDatabase::new();
        let cmd1 = Command::new();
        let cmd2 = Command::new();
        let cmd3 = Command::new();
        db.add_result(cmd1.clone(), cmd1.execute());
        db.add_result(cmd2.clone(), cmd2.execute());
        db.add_result(cmd3.clone(), cmd3.execute());

        assert_eq!(db.results_buffer.contains_key(&cmd1), true);

        db.remove_result(&cmd1);

        assert_eq!(db.results_buffer.contains_key(&cmd1), false);
    }

}