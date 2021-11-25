use std::time::Duration;

use tokio::time::sleep;

pub struct Settings {
    nbr_clients: usize,
    nbr_replicas: usize,
    nbr_byzantine_clients: usize,
    nbr_byzantine_replicas: usize,
    transmition_delay: Duration,
    n_ack: usize,
}
const BYZANTINE_F: usize = 2;
const BYZANTINE_RATE: f32 = 1.0 / 10.0;
const DEFAULT_TRANSMISSION_DELAY: Duration = Duration::from_millis(20);

impl Settings {
    pub fn new(
        nbr_clients: usize,
        nbr_replicas: usize,
        nbr_byzantine_clients: usize,
        nbr_byzantine_replicas: usize,
        transmition_delay: Duration,
        n_ack: usize,
    ) -> Self {
        Settings {
            nbr_clients,
            nbr_replicas,
            nbr_byzantine_clients,
            nbr_byzantine_replicas,
            transmition_delay,
            n_ack,
        }
    }

    pub fn default_settings(nbr_clients: usize, nbr_replicas: usize) -> Self {
        Settings::new(
            nbr_clients,
            nbr_replicas,
            (nbr_clients as f32 * BYZANTINE_RATE) as usize,
            BYZANTINE_F,
            DEFAULT_TRANSMISSION_DELAY,
            (nbr_replicas + BYZANTINE_F + 1) / 2,
        )
    }

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
