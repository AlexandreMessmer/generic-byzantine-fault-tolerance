use std::time::Duration;

use tokio::time::sleep;


pub struct Settings{
    transmition_delay: Duration,
    byzantine_peers: usize,
}

impl Settings{
    pub fn new(transmition_delay: Duration, byzantine_peers: usize) -> Self{
        Settings {
            transmition_delay,
            byzantine_peers,
        }
    }

    pub fn default_settings(peers: usize) -> Self {
        Settings {
            transmition_delay: Duration::from_millis(20),
            byzantine_peers: (peers / 5) as usize,
        }
    }

    pub async fn delay(&self){
        sleep(self.transmition_delay.clone()).await;
    }

    pub fn byzantine_peers(&self){
        self.byzantine_peers;
    }
}