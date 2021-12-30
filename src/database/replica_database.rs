use std::{collections::{BTreeMap, BTreeSet, HashMap}, ptr::eq};

use crate::talk::{Command, CommandResult, Message, RoundNumber, Phase};

pub type Set = BTreeSet<Command>;
pub type ResultBuffer = Vec<ResultBufferData>;
pub type ResultBufferData = (RoundNumber, Command, CommandResult, Phase);
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
    pending: HashMap<RoundNumber, Set>,
    results_buffer: ResultBuffer,
    round: usize,
}

impl ReplicaDatabase {
    pub fn new() -> Self {
        let received = BTreeSet::new();
        let delivered = BTreeSet::new();
        let pending = HashMap::new();
        let results_buffer = Vec::new();
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

    pub fn delivered_set(&mut self, set: &Set) {
        self.delivered = self.received.union(set).cloned().collect()
    }

    /// Set pending_{round} to {set}
    /// Returns true if the modification has been done
    pub fn set_pending(&mut self, round: RoundNumber, set: Set) -> bool {
        if self.pending.contains_key(&round) {
            self.pending.insert(round, set);
            return true;
        }
        false
    }

    pub fn add_result(&mut self, k: RoundNumber, cmd: Command, res: CommandResult, phase: Phase) {
        self.results_buffer.push((k, cmd, res, phase))
    }

    /// The result is executed speculatively. This operation is costly.
    pub fn remove_result(&mut self, k: RoundNumber, cmd: Command) {
        self.results_buffer = self.results_buffer.clone()
            .into_iter()
            .filter(|(round, command, _, _)| !(k.eq(round) && cmd.eq(command)))
            .collect();
    }

    pub fn received(&self) -> &Set {
        &self.received
    }

    pub fn delivered(&self) -> &Set {
        &self.delivered
    }

    pub fn pending(&self, k: &RoundNumber) -> Option<&Set> {
        self.pending.get(k)
    }

    pub fn results_buffer(&self) -> &ResultBuffer {
        &self.results_buffer
    }

    pub fn round(&self) -> &usize {
        &self.round
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
        db.add_result(1, cmd1.clone(), cmd1.execute(), Phase::ACK);
        db.add_result(1, cmd2.clone(), cmd2.execute(), Phase::ACK);
        db.add_result(1, cmd3.clone(), cmd3.execute(), Phase::ACK);

        assert_eq!(db.results_buffer.contains(&(1, cmd1.clone(), cmd1.execute(), Phase::ACK)), true);

        db.remove_result(1, cmd1.clone());

        assert_eq!(db.results_buffer.contains(&(1, cmd1.clone(), cmd1.execute(), Phase::ACK)), false);
    }

}