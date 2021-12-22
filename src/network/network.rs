use std::ops::Range;

use talk::{crypto::Identity, sync::fuse::Fuse, unicast::test::UnicastSystem};
use tokio::sync::mpsc;

use super::{network_info, NetworkInfo, NetworkPeer};

use crate::{
    crypto::identity_table::{IdentityTable, IdentityTableBuilder},
    peer::{
        handler::{
            ClientHandler, FaultyClientHandler, FaultyReplicaHandler, Handler, HandlerBuilder,
            ReplicaHandler,
        },
        runner::Runner,
        Peer,
    },
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

        let (peers, identity_table) =
            Self::compose_peers(network_info.clone(), keys, senders, receivers, outlets);

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
                    network_info.clone(),
                    identity_table.clone(),
                );
                Peer::<Message>::new(receiver, outlet, handler)
            })
            .collect::<Vec<_>>();

        (peers, identity_table)
    }
}
