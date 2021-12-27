use std::ops::Range;
use std::time::Duration;

use doomstack::Top;
use futures::future::join_all;
use futures::join;
use talk::time::{timeout, Timeout};
use talk::{crypto::Identity, sync::fuse::Fuse, unicast::test::UnicastSystem};
use tokio::sync::mpsc::error::SendError;
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;
use tokio::time::error::Elapsed;

use super::{NetworkInfo, NetworkPeer};

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
    feedback_outlet: FeedbackReceiver,
    identity_table: IdentityTable,
    fuse: Fuse,
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

        let (peers, identity_table) = Self::compose_peers(
            network_info.clone(),
            keys,
            senders,
            receivers,
            outlets,
            feedback_inlet,
        );

        let fuse = Fuse::new();
        {
            for peer in peers {
                fuse.spawn(async move {
                    peer.run().await;
                });
            }
        }

        Network {
            network_info,
            peers_inlets: inlets,
            feedback_outlet,
            identity_table,
            fuse,
        }
    }

    fn compose_peers(
        network_info: NetworkInfo,
        keys: Vec<Identity>,
        senders: Vec<UnicastSender<Message>>,
        receivers: Vec<UnicastReceiver<Message>>,
        outlets: Vec<InstructionReceiver>,
        feedback_inlet: FeedbackSender,
    ) -> (Vec<MessagePeer>, IdentityTable) {
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
                    network_info.clone(),
                    identity_table.clone(),
                );
                Peer::<Message>::new(receiver, outlet, handler)
            })
            .collect::<Vec<_>>();

        (peers, identity_table)
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
    pub async fn send_instruction_with_type(
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

    pub async fn send_instruction(
        &self,
        instruction: Instruction,
        id: PeerId,
    ) -> Option<JoinHandle<Result<(), SendError<Instruction>>>> {
        self.peer_inlet(id).map(|sender| {
            let sender = sender.clone();
            tokio::spawn(async move { sender.send(instruction).await })
        })
    }

    pub fn network_info(&self) -> &NetworkInfo {
        &self.network_info
    }

    pub fn identity_table(&self) -> &IdentityTable {
        &self.identity_table
    }

    pub async fn receive_feedback(&mut self) -> Option<Feedback> {
        self.feedback_outlet.recv().await
    }

    pub async fn shutdown(self) {
        let mut handles: Vec<JoinHandle<Result<Result<(), SendError<Instruction>>, Top<Timeout>>>> =
            Vec::new();
        for peer in self.peers_inlets.clone() {
            let future = async move { peer.send(Instruction::Shutdown).await };
            let future = timeout(Duration::from_secs(5), future);
            let handle = tokio::spawn(future);

            handles.push(handle);
        }
        let res = join_all(handles).await;
    }
    /// Wait until every peers are shut downed.
    /// Timeout with a default timer.
    pub async fn wait_until_shutdown(&mut self) {
        while let Ok(Some(_)) = timeout(SHUTDOWN_TIMEOUT.clone(), self.receive_feedback()).await {}
    }
}

impl Drop for Network {
    fn drop(&mut self) {
        println!("Network: shutdown");
    }
}

#[cfg(test)]
mod tests {

    use std::time::Duration;

    use futures::{future, FutureExt};

    use crate::network::network_info;

    use super::*;

    #[tokio::test]
    async fn building_network_works() {
        let network_info = NetworkInfo::new(5, 5, 0, 0, Duration::from_millis(10), 3);
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
        let network_info = NetworkInfo::new(2, 2, 2, 2, Duration::from_millis(10), 0);
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
            assert_eq!(feedback, Feedback::Acknowledgement);
        }
    }
}
