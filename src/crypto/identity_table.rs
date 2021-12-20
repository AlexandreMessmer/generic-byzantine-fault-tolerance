use std::{collections::HashMap, ops::Range};

use crate::{peer::Peer, system::PeerId, network::NetworkInfo};
use talk::{crypto::Identity, unicast::Message};

#[derive(Debug, Clone)]
pub struct IdentityTable {
    clients: Vec<Identity>,
    replicas: Vec<Identity>,
    client_range: Range<PeerId>,
    faulty_client_range: Range<PeerId>,
    replica_range: Range<PeerId>,
    faulty_replica_range: Range<PeerId>,
}

impl IdentityTable {
    pub fn clients(&self) -> &Vec<Identity> {
        &self.clients()
    }

    pub fn replicas(&self) -> &Vec<Identity> {
        &self.replicas()
    }
}

pub struct IdentityTableBuilder {
    peers_mapping: Vec<Identity>,
    network_info: NetworkInfo,
}

impl IdentityTableBuilder {
    pub fn new(network_info: NetworkInfo) -> Self {
        IdentityTableBuilder {
            peers_mapping: Vec::new(),
            network_info,
        }
    }

    pub fn add_peer(&mut self, peer: Identity) -> &Self {
        self.peers_mapping.push(peer);
        self
    }

    pub fn build(self) -> IdentityTable {
        let (client_range, faulty_client_range, replica_range, faulty_replica_range) = self.network_info.compute_ranges();
        let (clients, replicas) = self.peers_mapping.split_at(faulty_client_range.end);
        IdentityTable {
            clients: clients.to_vec(),
            replicas: replicas.to_vec(),
            client_range,
            faulty_client_range,
            replica_range,
            faulty_replica_range,
        }
    }


}
