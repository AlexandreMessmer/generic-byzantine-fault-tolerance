use std::ops::Range;

use talk::{crypto::Identity, unicast::test::UnicastSystem, sync::fuse::Fuse};
use tokio::sync::mpsc;

use super::{network_info, NetworkInfo};

use crate::{
    crypto::identity_table::{IdentityTableBuilder, IdentityTable},
    peer::{Peer, handler::{Handler, ClientHandler, FaultyClientHandler, ReplicaHandler, FaultyReplicaHandler}, runner::Runner},
    talk::{Instruction, Message},
    types::*,
};

type MessagePeer = Peer<Message>;

// A network of `Peer<Message>`.
pub struct Network {
    network_info: NetworkInfo,
    peers_inlets: Vec<InstructionSender>,
    identity_table: IdentityTable,
}

impl Network {
    pub async fn setup(network_info: NetworkInfo) -> Self {
        let mut inlets: Vec<InstructionSender> = Vec::new();
        let mut outlets: Vec<InstructionReceiver> = Vec::new();
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

        let (peers, identity_table)  = Self::compose_peers(network_info.clone(), keys, senders, receivers, outlets);

        let fuse = Fuse::new();
        {
            for mut peer in peers {
                fuse.spawn(async move {
                    peer.run().await;
                });
            }
        }

        Network {
            network_info,
            peers_inlets: inlets,
            identity_table,
        }
    }

    fn compose_peers(
        network_info: NetworkInfo,
        keys: Vec<Identity>,
        senders: Vec<UnicastSender<Message>>,
        receivers: Vec<UnicastReceiver<Message>>,
        outlets: Vec<InstructionReceiver>,
    ) -> (Vec<MessagePeer>, IdentityTable) {
        let (keys, senders, receivers) =
            (keys.into_iter(), senders.into_iter(), receivers.into_iter());

        let size = network_info.size();
        let identity_table = IdentityTableBuilder::new(network_info.clone());
        let ids = (0..size).into_iter();

        let mut start: usize = 0;
        let mut end: usize = 0;
        let (client_range, faulty_client_range, replica_range, faulty_replica_range) = network_info.compute_ranges();

        for key in keys {
            identity_table.add_peer(key.clone());
        }
        let identity_table = identity_table.build();

        let peers: Vec<MessagePeer> = ids.zip(keys)
            .zip(senders)
            .zip(receivers)
            .zip(outlets)
            .map(|((((id, key), sender), receiver), outlet)| {
                let handler = Self::get_corresponding_handler(&id, &client_range, &faulty_client_range, &replica_range, &faulty_replica_range);
                Peer::<Message>::new(
                    id,
                    key,
                    sender,
                    receiver,
                    outlet,
                    network_info,
                    identity_table.clone(),
                    handler,
                )
            })
            .collect::<Vec<_>>();

        (peers, identity_table)
    }


    fn get_corresponding_handler(id: &usize, client_range: &Range<usize>, faulty_client_range: &Range<usize>, replica_range: &Range<usize>, faulty_replica_range: &Range<usize>) -> Box<dyn Handler<Message>> {
        if client_range.contains(id) {
            return Box::new(ClientHandler::new());
        }
        else if faulty_client_range.contains(id) {
            return Box::new(FaultyClientHandler::new());
        }
        else if replica_range.contains(id) {
            return Box::new(ReplicaHandler::new());
        }
        else {
            return Box::new(FaultyReplicaHandler::new());
        }
    }


}

enum NetworkPeer {
    Client,
    FaultyClient,
    Replica,
    FaultyReplica,
}

impl NetworkPeer {
    pub fn get_corresponding_handler(id: usize, network_info: NetworkInfo, client_range: Range<usize>, faulty_client_range: Range<usize>, replica_range: Range<usize>, faulty_replica_range: Range<usize>) -> Box<dyn Handler<Message>> {
        if client_range.contains(&id) {
            return Box::new(ClientHandler::new());
        }
        else if faulty_client_range.contains(&id) {
            return Box::new(FaultyClientHandler::new());
        }
        else if replica_range.contains(&id) {
            return Box::new(ReplicaHandler::new());
        }
        else {
            return Box::new(FaultyReplicaHandler::new());
        }
    }
}

