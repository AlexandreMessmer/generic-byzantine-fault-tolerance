use std::collections::{BTreeSet, HashMap, HashSet};
use std::time::Duration;

use talk::crypto::Identity;
use tokio::sync::broadcast::error::SendError;
use tokio::sync::broadcast::{self, Receiver as BroadcastReceiver, Sender as BroadcastSender};
use tokio::sync::mpsc;

use crate::network::NetworkInfo;
use crate::talk::{Command, RoundNumber};
use crate::types::*;

use super::runner::Runner;

type ProposalSet = BTreeSet<Command>;
/// Represents data handled by the coordinator, namely (k, NCSet, CSet) (according to the paper)
pub type ProposalSignedData = (Identity, RoundNumber, ProposalSet, ProposalSet);
pub type ProposalData = (RoundNumber, ProposalSet, ProposalSet);
type ReceivedMap = HashMap<RoundNumber, HashMap<Identity, (ProposalSet, ProposalSet)>>;
const BUFFER_SIZE: usize = 100;
pub struct Coordinator {
    network_info: NetworkInfo,
    broadcaster: BroadcastSender<ProposalData>,
    receiver: MPSCReceiver<ProposalSignedData>,
    sender: MPSCSender<ProposalSignedData>, // Cannot be used, only to add a proposer.
    received: ReceivedMap,
    already_received: HashMap<RoundNumber, HashSet<Identity>>,
    validated: HashSet<RoundNumber>, // To discard treated consensus
}

impl Coordinator {
    pub fn new(network_info: NetworkInfo) -> Self {
        let (broadcaster, _) = broadcast::channel(BUFFER_SIZE);
        let (sender, receiver) = mpsc::channel(BUFFER_SIZE);
        let coordinator = Coordinator {
            network_info,
            broadcaster,
            receiver,
            sender,
            received: HashMap::new(),
            already_received: HashMap::new(),
            validated: HashSet::new(),
        };
        coordinator
    }

    pub fn subscribe(&self) -> BroadcastReceiver<ProposalData> {
        self.broadcaster.subscribe()
    }

    pub fn proposer(&self) -> MPSCSender<ProposalSignedData> {
        self.sender.clone()
    }

    fn propose(&mut self, data: ProposalSignedData) -> Option<(ProposalSet, ProposalSet)> {
        let (from, k, nc, c) = data;
        let is_unique = self
            .already_received
            .get(&k)
            .map(|hash_map| !hash_map.contains(&from))
            .unwrap_or(true);
        if !self.validated.contains(&k) && is_unique {
            let map = self.received.entry(k).or_insert(HashMap::new());

            // Checks uniqueness
            if map.contains_key(&from) {
                map.remove(&from);
                self.already_received
                    .entry(k)
                    .or_insert(HashSet::new())
                    .insert(from.clone());
            } else {
                map.insert(from, (nc, c));
                return self.validate(k);
            }
        }

        None
    }

    /// Returns Some(NCSet, CSet) if it is validated
    fn validate(&mut self, k: RoundNumber) -> Option<(ProposalSet, ProposalSet)> {
        let is_complete = self
            .received
            .get(&k)
            .map(|id_map| id_map.len() >= self.network_info.n_ack());

        if let Some(true) = is_complete {
            return self.received.remove(&k).map(|id_map| {
                let values: Vec<(ProposalSet, ProposalSet)> = id_map.into_values().collect();
                let (non_conflictings, conflictings): (Vec<ProposalSet>, Vec<ProposalSet>) =
                    values.into_iter().unzip();

                let mut reduced_nc: HashMap<Command, usize> = HashMap::new();
                let mut reduced_conflicting: BTreeSet<Command> = conflictings.into_iter().fold(
                    BTreeSet::<Command>::new(),
                    |mut acc, mut item| {
                        acc.append(&mut item);
                        acc
                    },
                );
                let non_conflictings = non_conflictings.into_iter();

                for commands in non_conflictings {
                    for cmd in commands {
                        reduced_nc
                            .entry(cmd)
                            .and_modify(|nbr| *nbr += 1)
                            .or_insert(1);
                    }
                }

                let reduced_nc = reduced_nc.into_iter();
                let threshold: usize = self.network_info.n_ack() + 1;
                let threshold: usize = if threshold % 2 == 0 {
                    threshold / 2
                } else {
                    (threshold + 1) / 2
                };

                let reduced_non_conflicting: ProposalSet =
                    reduced_nc.fold(BTreeSet::new(), |mut acc, (command, nbr)| {
                        if nbr >= threshold {
                            acc.insert(command);
                        } else {
                            reduced_conflicting.insert(command);
                        }
                        acc
                    });

                let reduced_conflicting: ProposalSet = reduced_conflicting
                    .difference(&reduced_non_conflicting)
                    .cloned()
                    .collect();

                self.validated.insert(k);

                (reduced_non_conflicting, reduced_conflicting)
            });
        }

        None
    }

    async fn broadcast(
        &self,
        k: RoundNumber,
        nc_set: BTreeSet<Command>,
        c_set: BTreeSet<Command>,
    ) -> Result<usize, SendError<ProposalData>> {
        tokio::time::sleep(Duration::from_millis(
            self.network_info.transmition_delay() * self.network_info.consensus_slowdown(),
        ))
        .await;
        self.broadcaster.send((k, nc_set, c_set))
    }
    pub fn received(&self) -> &ReceivedMap {
        &self.received
    }

    #[cfg(test)]
    pub fn receiver(&mut self) -> &mut MPSCReceiver<ProposalSignedData> {
        &mut self.receiver
    }

    #[cfg(test)]
    pub fn broadcaster(&self) -> &BroadcastSender<ProposalData> {
        &self.broadcaster
    }
}

#[async_trait::async_trait]
impl Runner for Coordinator {
    async fn run(mut self) {
        while let Some(data) = self.receiver.recv().await {
            let (_, k, _, _) = data;
            let proposition = self.propose(data);
            if let Some((nc_set, c_set)) = proposition {
                self.broadcast(k, nc_set, c_set).await.unwrap();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use talk::unicast::test::UnicastSystem;

    use crate::{banking::action::Action, network::network::Network, talk::Message};

    use super::*;

    #[test]
    fn btree_set_equality() {
        let mut set1: BTreeSet<u64> = BTreeSet::new();
        let mut set2: BTreeSet<u64> = BTreeSet::new();

        set1.insert(1);
        set1.insert(2);

        set2.insert(1);
        set2.insert(2);

        assert_eq!(&set1, &set2);
    }

    #[test]
    fn tuple_map() {
        let mut map: HashMap<i32, i32> = HashMap::new();
        map.insert(1, 1);
        map.insert(2, 2);

        let _values: Vec<i32> = map.into_values().collect();
    }

    #[tokio::test]
    async fn propose_correctly_handle_unique_messages() {
        let network_info = NetworkInfo::new(5, 5, 0, 0, 10, 4);
        let mut coordinator = Coordinator::new(network_info);
        let UnicastSystem {
            mut keys,
            senders: _,
            receivers: _,
        } = UnicastSystem::<Message>::setup(2).await;

        let id1 = keys.remove(0);
        let id2 = keys.remove(0);

        let mut nc_set: BTreeSet<Command> = BTreeSet::new();
        let cmd1 = Command::new(0, Action::Register);
        let cmd2 = Command::new(0, Action::Register);
        let cmd3 = Command::new(0, Action::Register);
        nc_set.insert(cmd1.clone());
        nc_set.insert(cmd2.clone());
        nc_set.insert(cmd3.clone());

        let mut c_set: ProposalSet = BTreeSet::new();
        let cmd4 = Command::new(0, Action::Register);
        let cmd5 = Command::new(0, Action::Register);
        let cmd6 = Command::new(0, Action::Register);
        c_set.insert(cmd4.clone());
        c_set.insert(cmd5.clone());
        c_set.insert(cmd6.clone());

        coordinator.propose((id1.clone(), 1, nc_set.clone(), c_set.clone()));
        let p1 = coordinator.received();
        let id_map = p1.get(&1).unwrap();
        let (set1, set2) = id_map.get(&id1).unwrap();
        assert_eq!(*set1, nc_set);
        assert_eq!(*set2, c_set);

        for _ in 0..10 {
            coordinator.propose((id1.clone(), 1, BTreeSet::new(), BTreeSet::new()));

            let contains = coordinator.received().get(&1).unwrap().contains_key(&id1);
            assert_eq!(contains, false);
        }

        coordinator.propose((id2.clone(), 1, nc_set.clone(), c_set.clone()));
        let p1 = coordinator.received();
        let id_map = p1.get(&1).unwrap();
        let (set1, set2) = id_map.get(&id2).unwrap();
        assert_eq!(*set1, nc_set);
        assert_eq!(*set2, c_set);

        for _ in 0..10 {
            coordinator.propose((id2.clone(), 1, BTreeSet::new(), BTreeSet::new()));

            let contains = coordinator.received().get(&1).unwrap().contains_key(&id2);
            assert_eq!(contains, false);
        }
    }

    #[tokio::test]
    async fn validate_correctly_handle_conflicting_sets() {
        let network_info = NetworkInfo::new(5, 5, 0, 0, 10, 4);
        let mut coordinator = Coordinator::new(network_info.clone());
        let network = Network::setup(network_info).await;

        let ids: Vec<Identity> = network.identity_table().replicas().clone();
        let round = 1;
        let mut nc_set: BTreeSet<Command> = BTreeSet::new();
        let cmd1 = Command::new(0, Action::Register);
        let cmd2 = Command::new(0, Action::Register);
        let cmd3 = Command::new(0, Action::Register);
        nc_set.insert(cmd1.clone());
        nc_set.insert(cmd2.clone());
        nc_set.insert(cmd3.clone());

        let mut c_set: ProposalSet = BTreeSet::new();
        let cmd4 = Command::new(0, Action::Register);
        let cmd5 = Command::new(0, Action::Register);
        let cmd6 = Command::new(0, Action::Register);
        c_set.insert(cmd4.clone());
        c_set.insert(cmd5.clone());
        c_set.insert(cmd6.clone());

        for i in 0..network.network_info().nbr_replicas() {
            if i == 2 {
                nc_set.remove(&cmd1);
            }
            if let Some((nc, c)) = coordinator.propose((
                ids.get(i).unwrap().clone(),
                round,
                nc_set.clone(),
                c_set.clone(),
            )) {
                println!("NCSET: {:#?}, CSET: {:#?} (iter: {})", nc, c, i);
                assert_eq!(i, 3);
                assert_eq!(nc.len(), 2);
                assert_eq!(c.len(), 4);

                assert_eq!(nc.contains(&cmd1), false);
                assert_eq!(nc.contains(&cmd2), true);
                assert_eq!(nc.contains(&cmd3), true);
                assert_eq!(c.contains(&cmd4), true);
                assert_eq!(c.contains(&cmd5), true);
                assert_eq!(c.contains(&cmd6), true);
                assert_eq!(c.contains(&cmd1), true);
                assert_eq!(c.contains(&cmd2), false);
                assert_eq!(c.contains(&cmd3), false);
                assert_eq!(nc.contains(&cmd4), false);
                assert_eq!(nc.contains(&cmd5), false);
                assert_eq!(nc.contains(&cmd6), false);
            }
        }
    }

    #[tokio::test]
    async fn does_not_propose_out_of_order() {
        let network_info = NetworkInfo::new(5, 5, 0, 0, 10, 3);
        let mut coordinator = Coordinator::new(network_info.clone());
        let network = Network::setup(network_info.clone()).await;

        let ids: Vec<Identity> = network.identity_table().replicas().clone();
        let round = 1;
        let mut nc_set: BTreeSet<Command> = BTreeSet::new();
        let cmd1 = Command::new(0, Action::Register);
        let cmd2 = Command::new(0, Action::Register);
        let cmd3 = Command::new(0, Action::Register);
        nc_set.insert(cmd1.clone());
        nc_set.insert(cmd2.clone());
        nc_set.insert(cmd3.clone());

        let mut c_set: ProposalSet = BTreeSet::new();
        let cmd4 = Command::new(0, Action::Register);
        let cmd5 = Command::new(0, Action::Register);
        let cmd6 = Command::new(0, Action::Register);
        c_set.insert(cmd4.clone());
        c_set.insert(cmd5.clone());
        c_set.insert(cmd6.clone());

        let mut i: usize = 0;
        while let None = coordinator.propose((
            ids.get(i).unwrap().clone(),
            round,
            nc_set.clone(),
            c_set.clone(),
        )) {
            println!("{}", i);
            i += 1;
        }

        println!("End loop");
        let received = coordinator.received.clone();

        while i < network_info.n_ack() {
            let _proposed = coordinator.propose((
                ids.get(i).unwrap().clone(),
                round,
                nc_set.clone(),
                c_set.clone(),
            ));
            i += 1;
        }

        assert_eq!(received, coordinator.received.clone());
    }
}
