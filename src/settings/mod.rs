use std::time::Duration;

pub mod runner_settings;
pub mod system_settings;

const BYZANTINE_F: usize = 2;
const BYZANTINE_RATE: f32 = 1.0 / 10.0;
const DEFAULT_TRANSMISSION_DELAY: Duration = Duration::from_millis(20);

#[derive(Clone, Debug)]
pub struct RunnerSettings {
    transmission_delay: Duration,
    n_ack: usize,
}

#[derive(Clone, Debug)]

pub struct SystemSettings {
    nbr_clients: usize,
    nbr_replicas: usize,
    nbr_byzantine_clients: usize,
    nbr_byzantine_replicas: usize,
    transmition_delay: Duration,
    n_ack: usize,
}

fn default_n_ack(peers: usize) -> usize {
    (peers + BYZANTINE_F + 1) / 2
}
