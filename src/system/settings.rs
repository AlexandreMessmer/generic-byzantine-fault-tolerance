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
impl Settings {
    pub fn new(nbr_clients: usize, nbr_replicas: usize, nbr_byzantine_clients: usize, nbr_byzantine_replicas: usize, transmition_delay: Duration, n_ack: usize) -> Self {
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
        Settings::new(nbr_clients, nbr_replicas, )
        
    }

    pub async fn delay(&self) {
        sleep(self.transmition_delay.clone()).await;
    }

    pub fn byzantine_peers(&self) {
        self.byzantine_peers;
    }

    pub fn n_ack(&self){
        self.n_ack;
    }
}
