use std::time::Duration;

use tokio::time::sleep;

use crate::crypto::identity_table::IdentityTable;

pub struct NetworkInfo {
    nbr_clients: usize,
    nbr_replicas: usize,
    nbr_byzantine_clients: usize,
    nbr_byzantine_replicas: usize,
    identity_table: IdentityTable,
    transmition_delay: Duration,
    n_ack: usize,
}

impl NetworkInfo {
    pub async fn simulate_delay(&self) {
        sleep(self.transmition_delay.clone()).await;
    }

    pub fn nbr_clients(&self) -> usize {
        self.nbr_clients
    }

    pub fn nbr_replicas(&self) -> usize {
        self.nbr_replicas
    }

    pub fn nbr_byzantine_clients(&self) -> usize {
        self.nbr_byzantine_clients
    }

    pub fn nbr_byzantine_replicas(&self) -> usize {
        self.nbr_byzantine_replicas
    }

    pub fn n_ack(&self) {
        self.n_ack;
    }
}