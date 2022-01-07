use std::{
    time::{Duration, SystemTime},
};


use tokio::time::sleep;

use crate::{
    banking::{action::Action, banking::Money},
    network::{network::Network, NetworkInfo},
    peer::peer::PeerId,
    talk::Command,
};

pub type ScenarioItem = (PeerId, Action);
pub type Scenario = Vec<ScenarioItem>;
const WRITE_LOGS: bool = false;
pub struct Simulation {
    pub title: String,
    pub network: Network,
    pub scenario: Scenario,
}

impl Simulation {
    pub async fn new(
        title: String,
        report_folder: String,
        nbr_clients: usize,
        nbr_replicas: usize,
        nbr_faulty_replicas: usize,
        transmission_delay: u64,
        consensus_duration: f64,
    ) -> Self {
        let mut network_info = NetworkInfo::default_parameters(
            nbr_clients,
            nbr_replicas,
            0,
            nbr_faulty_replicas,
            transmission_delay,
            consensus_duration,
            report_folder,
        );
        network_info.set_write_logs(WRITE_LOGS);

        let mut simulation = Self {
            title,
            network: Network::setup(network_info).await,
            scenario: Vec::new(),
        };

        simulation.initiate_register_all().await;

        simulation
    }

    fn add_scenario_item(&mut self, client: PeerId, action: Action) {
        self.scenario.push((client, action.clone()));
        self.network.execute(client, Command::new(client, action));
    }

    pub fn deposit(&mut self, id: PeerId, amount: Money) {
        self.add_scenario_item(id, Action::Deposit(amount));
    }

    pub async fn initiate_register_all(&mut self) {
        self.network.register_all();
        for i in 0..self.network.network_info().nbr_clients() {
            self.network.execute_next(i).await;
            self.network.get_balance(i);
            self.network.execute_next(i).await;
        }

        sleep(Duration::from_secs_f64(self.network.network_info().consensus_duration() * 3.0)).await;
    }

    pub fn register(&mut self, id: PeerId) {
        self.add_scenario_item(id, Action::Register);
    }

    pub fn get(&mut self, id: PeerId) {
        self.add_scenario_item(id, Action::Get);
    }

    pub fn withdraw(&mut self, id: PeerId, amount: Money) {
        self.add_scenario_item(id, Action::Withdraw(amount));
    }

    /*
    pub fn add_scenario(&mut self, path: String) {
        let path = Path::new(&path);
        let display = path.display();

        // Open the path in read-only mode, returns `io::Result<File>`
        let lines = match File::open(&path) {
            Err(why) => panic!("couldn't open {}: {}", display, why),
            Ok(file) => BufReader::new(file).lines(),
        };

        for line in lines {
            if let Ok(line) = line {
                line.
            }
        }
    }
    */

    pub async fn simulate(&mut self) -> Duration {
        let time = SystemTime::now();
        self.network.execute_all().await;
        let _ = self.network.shutdown().await;
        time.elapsed()
            .expect("Failed to mesure the duration of the simulation")
    }

    pub async fn clone(&self) -> Self {
        let mut s = Self::new(
            self.title.clone(),
            self.network.network_info().report_folder().clone(),
            self.network.network_info().nbr_clients(),
            self.network.network_info().nbr_replicas(),
            self.network.network_info().nbr_faulty_replicas(),
            self.network.network_info().transmission_delay(),
            self.network.network_info().consensus_duration(),
        )
        .await;
        for (client, action) in self.scenario.iter() {
            s.add_scenario_item(*client, action.clone());
        }

        s
    }
}

#[cfg(test)]
mod tests {
    use crate::{tests::scenarios::{CLIENTS, REPLICAS, FAULTY}, network::network_info::DEFAULT_REPORT_FOLDER};

    use super::*;
    #[tokio::test]
    async fn consensus_overhead() {
        let simulation = Simulation::new(String::from("TEST"), String::from(DEFAULT_REPORT_FOLDER), CLIENTS, REPLICAS, FAULTY, 10, 0.1).await;
        let mut without_overhead = simulation.clone().await;
        let mut with_overhead = simulation.clone().await;

        without_overhead.deposit(0, 10);
        without_overhead.withdraw(0, 5);
        for _ in 0..100 {
            without_overhead.get(0);
            with_overhead.get(0);
        }

        with_overhead.deposit(0, 10);
        with_overhead.withdraw(0, 5);

        let without_overhead = without_overhead.simulate().await.as_secs_f64();
        let with_overhead = with_overhead.simulate().await.as_secs_f64();

        println!("Diff = {}s", with_overhead - without_overhead);
    }

    #[tokio::test]
    async fn test12() {
        let simulation = Simulation::new(String::from("TEST"), String::from(DEFAULT_REPORT_FOLDER), CLIENTS, REPLICAS, FAULTY, 10, 0.1).await;

    }
}