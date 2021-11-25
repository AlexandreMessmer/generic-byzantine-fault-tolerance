use talk::crypto::Identity;

pub struct IdentityTable {
    pub clients: Vec<Identity>,
    pub replicas: Vec<Identity>,
}

impl IdentityTable {
    pub fn new(clients: Vec<Identity>, replicas: Vec<Identity>) -> Self {
        IdentityTable { clients, replicas }
    }

    pub fn initiate() -> IdentityTableBuilder {
        IdentityTableBuilder::new()
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
