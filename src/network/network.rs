use std::collections::{HashSet, HashMap, VecDeque};
use std::ops::Range;
use std::slice::SliceIndex;
use std::sync::Arc;
use std::time::{Duration, SystemTimeError};

use doomstack::Top;
use futures::future::join_all;

use futures::sink::Feed;
use talk::time::{timeout, Timeout};
use talk::{crypto::Identity, sync::fuse::Fuse, unicast::test::UnicastSystem};
use tokio::sync::{mpsc, Mutex};
use tokio::sync::mpsc::error::SendError;
use tokio::task::JoinHandle;

use super::{NetworkInfo, NetworkPeer};

use crate::banking::action::Action;
use crate::banking::banking::Money;
use crate::peer::coordinator::Coordinator;
use crate::talk::{Command, instruction};
use crate::{
    crypto::identity_table::{IdentityTable, IdentityTableBuilder},
    peer::{handler::HandlerBuilder, peer::PeerId, runner::Runner, Peer},
    talk::{Feedback, FeedbackChannel, FeedbackReceiver, FeedbackSender, Instruction, Message},
    types::*,
};

type MessagePeer = Peer<Message>;
const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(10);

// A network of `Peer<Message>`.
pub struct Network {
    network_info: NetworkInfo,
    peers_inlets: Vec<InstructionSender>,
    pending_execution: HashMap<PeerId, VecDeque<Command>>,
    feedback_outlet: FeedbackReceiver,
    identity_table: IdentityTable,
    _fuse: Fuse,
}

impl Network {
    pub async fn setup(network_info: NetworkInfo) -> Self {
        let mut inlets: Vec<InstructionSender> = Vec::new();
        let mut outlets: Vec<InstructionReceiver> = Vec::new();
        let (feedback_inlet, feedback_outlet) = FeedbackChannel::channel();
        let size = network_info.size();
        for _ in 0..size {
            let (tx, rx) = mpsc::channel::<Instruction>(32);
            inlets.push(tx);
            outlets.push(rx);
        }
        let inlets = inlets;
        let outlets = outlets;

        let UnicastSystem {
            keys,
            senders,
            receivers,
        } = UnicastSystem::<Message>::setup(size).await.into();

        let (peers, coordinator, identity_table) = Self::compose_peers(
            network_info.clone(),
            keys,
            senders,
            receivers,
            outlets,
            feedback_inlet,
        );

        let mut pending_execution: HashMap<PeerId, VecDeque<Command>> = HashMap::new();

        let fuse = Fuse::new();
        {
            for peer in peers {
                pending_execution.insert(*peer.id(), VecDeque::new());
                fuse.spawn(async move {
                    peer.run().await;
                });
            }

            fuse.spawn(async move {
                coordinator.run().await;
            });
        }

        Network {
            network_info,
            peers_inlets: inlets,
            pending_execution,
            feedback_outlet,
            identity_table,
            _fuse: fuse,
        }
    }

    fn compose_peers(
        network_info: NetworkInfo,
        keys: Vec<Identity>,
        senders: Vec<UnicastSender<Message>>,
        receivers: Vec<UnicastReceiver<Message>>,
        outlets: Vec<InstructionReceiver>,
        feedback_inlet: FeedbackSender,
    ) -> (Vec<MessagePeer>, Coordinator, IdentityTable) {
        let (keys, senders, receivers) =
            (keys.into_iter(), senders.into_iter(), receivers.into_iter());
        let size = network_info.size();
        let ids = (0..size).into_iter();

        let mut identity_table = IdentityTableBuilder::new(network_info.clone());
        for key in keys.clone() {
            identity_table.add_peer(key.clone());
        }
        let identity_table = identity_table.build();

        let (client_range, faulty_client_range, replica_range, faulty_replica_range) =
            network_info.compute_ranges();

        let coordinator = Coordinator::new(network_info.clone());

        let peers: Vec<MessagePeer> = ids
            .zip(keys)
            .zip(senders)
            .zip(receivers)
            .zip(outlets)
            .map(|((((id, key), sender), receiver), outlet)| {
                let peer_type = NetworkPeer::get_corresponding_type(
                    &id,
                    &client_range,
                    &faulty_client_range,
                    &replica_range,
                    &faulty_replica_range,
                )
                .unwrap();
                let handler = HandlerBuilder::handler(
                    peer_type,
                    id,
                    key,
                    sender,
                    feedback_inlet.clone(),
                    &coordinator,
                    network_info.clone(),
                    identity_table.clone(),
                );
                Peer::<Message>::new(receiver, outlet, handler)
            })
            .collect::<Vec<_>>();

        (peers, coordinator, identity_table)
    }

    fn peer_inlet(&self, id: PeerId) -> Option<&InstructionSender> {
        self.peers_inlets.get(id)
    }

    fn inlet(&self, peer_type: NetworkPeer, index: usize) -> Option<&InstructionSender> {
        let id = Self::get_id(&self, peer_type, index);
        id.map(|id| self.peer_inlet(id)).flatten()
    }
    fn get_id(&self, peer_type: NetworkPeer, index: usize) -> Option<PeerId> {
        match peer_type {
            NetworkPeer::Client => {
                let (client_ids, _) = self.identity_table.client_ids();
                Self::compute_id(index, client_ids)
            }
            NetworkPeer::FaultyClient => {
                let (_, ids) = self.identity_table.client_ids();
                Self::compute_id(index, ids)
            }
            NetworkPeer::Replica => {
                let (ids, _) = self.identity_table.replica_ids();
                Self::compute_id(index, ids)
            }
            NetworkPeer::FaultyReplica => {
                let (_, ids) = self.identity_table.replica_ids();
                Self::compute_id(index, ids)
            }
        }
    }

    fn compute_id(index: usize, range: &Range<usize>) -> Option<usize> {
        let id = index + range.start;
        if range.contains(&id) {
            return Some(id);
        }
        None
    }

    /// Sends an instruction to a given Peer, index by type `NetworkPeer`
    /// For example, if the `Network is (Client1, Client2, Replica1, Replica2), an instructon is
    /// given to Replica1 with `NetworkPeer::Replica` and 0 as index.
    async fn send_instruction_with_type(
        &self,
        instruction: Instruction,
        peer_type: NetworkPeer,
        index: usize,
    ) -> Option<JoinHandle<Result<(), SendError<Instruction>>>> {
        self.inlet(peer_type, index).map(|sender| {
            let sender = sender.clone();
            tokio::spawn(async move { sender.send(instruction).await })
        })
    }

    async fn send_instruction(
        &self,
        instruction: Instruction,
        id: PeerId,
    ) -> Option<JoinHandle<Result<(), SendError<Instruction>>>> {
        self.peer_inlet(id).map(|sender| {
            let sender = sender.clone();
            tokio::spawn(async move { sender.send(instruction).await })
        })
    }

    /// Execute the given command and wait for the result
    fn execute(&mut self, client: PeerId, command: Command) -> bool {
        self.pending_execution.get_mut(&client).map(|pending_commands| {
            pending_commands.push_back(command);
            true
        }).unwrap_or(false)
    }

    async fn execute_next(&mut self, client: PeerId) -> bool {
        let command = self.pending_execution.get_mut(&client).map(|vec| vec.pop_front()).flatten();
        if let Some(command) = command {
            let instruction = Instruction::Execute(command.clone());
            self.send_instruction(instruction, client).await.expect(&format!("#{} failed to execute {:#?}", client, command));
            return true;
        }
        false
    }

    // Assume that no client is faulty
    async fn execute_all(&mut self) {
        // Execute one action for every peer 
        for i in 0..self.network_info.nbr_clients() {
            self.execute_next(i).await;
        }

        let mut has_next = false;
        // Once a client ends, try to execute the next one
        while let Some(feedback) = self.feedback_outlet.recv().await {
            let from = feedback.from();
            match feedback {
                Feedback::Error(id, msg) => println!("Client #{} failed: {}", id, msg),
                Feedback::Acknowledgement(id) => println!("Client #{} request is successful", id),
                Feedback::Result(id, res) => println!("Client #{} request is successful ({})", id, res),
                Feedback::ShutdownComplete(_) => {},
            }
            if !has_next {
                if self.pending_execution.clone().iter_mut().all(|(_, pending)| pending.is_empty()) {
                    break;
                }
            }
            has_next = self.execute_next(from).await;
        }
    }
    //async fn execute_multiple(&mut self, )

    /* Banking operations */

    pub fn deposit(&mut self, client: PeerId, amount: Money) -> bool {
        self.execute(client, Command::new(client, Action::Deposit(amount)))
    }

    pub fn register(&mut self, client: PeerId) -> bool {
        self.execute(client, Command::new(client, Action::Register))
    }

    pub fn get_balance(&mut self, client: PeerId) -> bool {
        self.execute(client, Command::new(client, Action::Get))
    }

    pub fn withdraw(&mut self, client: PeerId, amount: Money) -> bool {
        self.execute(client, Command::new(client, Action::Withdraw(amount)))
    }

    pub fn register_all(&mut self) -> Vec<bool> {
        let mut feedbacks: Vec<bool> = Vec::new();
        for i in 0..self.network_info().nbr_clients() {
            let res = self.register(i);
            feedbacks.push(res);
        }

        feedbacks
    }
    /* _______________________ */

    pub fn network_info(&self) -> &NetworkInfo {
        &self.network_info
    }

    pub fn identity_table(&self) -> &IdentityTable {
        &self.identity_table
    }

    async fn receive_feedback(&mut self) -> Option<Feedback> {
        self.feedback_outlet.recv().await
    }

    /// Blocking function to shutdown the network.
    pub async fn shutdown(&mut self) -> Result<Duration, SystemTimeError> {
        println!("Start shutdown");
        let mut handles: Vec<JoinHandle<Result<Result<(), SendError<Instruction>>, Top<Timeout>>>> =
            Vec::new();
        for peer in self.peers_inlets.clone() {
            let future = async move { peer.send(Instruction::Shutdown).await };
            let future = timeout(SHUTDOWN_TIMEOUT.clone(), future);
            let handle = tokio::spawn(future);

            handles.push(handle);
        }
        tokio::join!(join_all(handles));
        self.wait_until_shutdown().await;
        self.network_info().elapsed()
    }
    /// Wait until every peers are shut downed.
    /// Timeout with a default timer.
    pub async fn wait_until_shutdown(&mut self) {
        let mut i = 0;
        while let Ok(Some(feedback)) = timeout(SHUTDOWN_TIMEOUT.clone(), self.receive_feedback()).await {
            if let Feedback::ShutdownComplete(_) = feedback {
                i += 1;
            }
            if i >= self.network_info().size() {
                break;
            }
        }
    }
}

impl Drop for Network {
    fn drop(&mut self) {
        println!("[{:#?}] Network: shutdown", self.network_info.elapsed().unwrap());
    }
}

#[cfg(test)]
mod tests {

    use std::time::Duration;

    use crate::network::network_info;

    use super::*;

    #[tokio::test]
    async fn building_network_works() {
        let network_info = NetworkInfo::with_default_report_folder(5, 5, 0, 0, 100, 3);
        let mut network = Network::setup(network_info).await;
        for i in 0..10 {
            network.send_instruction(Instruction::Testing, i).await;
            network.send_instruction(Instruction::Shutdown, i).await;
        }

        while let Some(_) = network.receive_feedback().await {}

        //network.shutdown().await;
    }

    #[tokio::test]
    async fn sending_instruction_with_type_reaches_every_peers() {
        let network_info = NetworkInfo::with_default_report_folder(2, 2, 2, 2, 10, 0);
        let mut network = Network::setup(network_info).await;
        let types: [NetworkPeer; 4] = [
            NetworkPeer::Client,
            NetworkPeer::FaultyClient,
            NetworkPeer::Replica,
            NetworkPeer::FaultyReplica,
        ];
        for i in 0..2 {
            for t in types {
                let future = network.send_instruction_with_type(Instruction::Testing, t, i);
                timeout(SHUTDOWN_TIMEOUT, future).await.unwrap();
            }
        }

        while let Ok(Some(feedback)) =
            timeout(Duration::from_secs(1), network.receive_feedback()).await
        {
            if let Feedback::Acknowledgement(_) = feedback {} else {panic!()}
        }
    }

    #[tokio::test]
    async fn end_to_end_test() {
        let network_info = NetworkInfo::default(3, 11, 3, 0, 20);
        let mut network = Network::setup(network_info).await;
        
        network.register_all();
        for i in 0..3 {
            for _ in 0..10 {
                network.deposit(i, 1);
            }
        }
        network.execute_all().await;
        network.shutdown().await;

    }

    #[tokio::test]
    async fn end_to_end_test2() {
        let network_info = NetworkInfo::default_parameters(3, 11, 3, 0, 20, 1.0, String::from("resources/test1"));
        let mut network = Network::setup(network_info).await;
        network.register_all();
        for i in 0..3 {
            for _ in 0..10 {
                network.deposit(i, 1);
                network.withdraw(i, 1);
            }
        }

        network.execute_all().await;
        network.shutdown().await;

    }
}
