use std::ops::{Index, Range};

use crate::{network::NetworkInfo, peer::peer::PeerId};
use talk::crypto::Identity;

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
        &self.clients
    }

    pub fn replicas(&self) -> &Vec<Identity> {
        &self.replicas
    }

    pub fn is_faulty(&self, id: &PeerId) -> bool {
        self.faulty_client_range.contains(id) || self.faulty_replica_range.contains(id)
    }

    /// Returns a reference to the range of idenfiers for clients.
    /// The returned value is a tuple that contains the range of reliable clients and
    /// the range of faulty clients
    pub fn client_ids(&self) -> (&Range<PeerId>, &Range<PeerId>) {
        (&self.client_range, &self.faulty_client_range)
    }

    /// Returns a reference to the range of idenfiers for replicas.
    /// The returned value is a tuple that contains the range of reliable replicas and
    /// the range of faulty replicas
    pub fn replica_ids(&self) -> (&Range<PeerId>, &Range<PeerId>) {
        (&self.replica_range, &self.faulty_replica_range)
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
        let (client_range, faulty_client_range, replica_range, faulty_replica_range) =
            self.network_info.compute_ranges();
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

#[cfg(test)]
mod tests {}
