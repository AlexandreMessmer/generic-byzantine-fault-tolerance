use talk::crypto::Identity;
use crate::system::PeerId;

#[derive(Debug)]
pub struct IdentityTable {
    clients: Vec<Identity>,
    replicas: Vec<Identity>,
}

impl IdentityTable {
    pub fn new(clients: Vec<Identity>, replicas: Vec<Identity>) -> Self {
        IdentityTable { clients, replicas }
    }

    pub fn initiate() -> IdentityTableBuilder {
        IdentityTableBuilder::new()
    }

    pub fn client_identity(&self, id: PeerId) -> Option<Identity> {
        if id < self.clients.len() {
            return Some(self.clients.get(id).unwrap().clone());
        }

        None
    }

    pub fn replica_identity(&self, id: PeerId) -> Option<Identity> {
        if id < self.replicas.len() {
            return Some(self.replicas.get(id).unwrap().clone());
        }

        None
    }

    pub fn replicas(&self) -> &Vec<Identity> {
        &self.replicas
    }

    pub fn clients(&self) -> &Vec<Identity> {
        &self.clients
    }

    pub fn nbr_clients(&self) -> usize {
        self.clients.len()
    }

    pub fn nbr_replicas(&self) -> usize {
        self.replicas.len()
    }
}

pub struct IdentityTableBuilder {
    clients: Vec<Identity>,
    replicas: Vec<Identity>,
}

impl IdentityTableBuilder {
    fn new() -> Self {
        IdentityTableBuilder {
            clients: Vec::<_>::new(),
            replicas: Vec::<_>::new(),
        }
    }

    pub fn add_client(&mut self, client: &Identity) -> &Self {
        self.clients.push(client.clone());
        self
    }

    pub fn add_replica(&mut self, replica: &Identity) -> &Self {
        self.replicas.push(replica.clone());
        self
    }

    pub fn build(self) -> IdentityTable {
        IdentityTable {
            clients: self.clients,
            replicas: self.replicas,
        }
    }
}

impl Clone for IdentityTable {
    fn clone(&self) -> Self {
        Self {
            clients: self.clients.clone(),
            replicas: self.replicas.clone(),
        }
    }
}
