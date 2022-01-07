use std::{fs::File, io::Write, path::Path, time::Duration, collections::HashMap};

use rand::thread_rng;
use rand_distr::{Distribution, Uniform};

use crate::{tests::simulation::Simulation};

pub struct Scenarios {
    file: File,
}

pub const GENERATION_LOOP: usize = 100;
pub const CLIENTS: usize = 2;
pub const REPLICAS: usize = 6;
pub const FAULTY: usize = 1;
pub const DEFAULT_PRECISION: usize = 1;
pub const DEFAULT_REPORT_FOLDER: &str = "reports/scenarios/logs";
pub const DEFAULT_TRANSMISSION_DELAY_MS: u64 = 300;
pub const DEFAULT_CONSENSUS_DURATION_S: f64 = 5.0;
/// Defines some testing scenarios
impl Scenarios {
    pub fn new(path: &str) -> Self {
        let path = format!("{}", path);
        let path = Path::new(&path);
        let display = path.display();

        // Open a file in write-only mode, returns `io::Result<File>`
        let file = match File::create(&path) {
            Err(why) => panic!("Couldn't create {}: {}", display, why),
            Ok(file) => file,
        };

        Scenarios { file }
    }

    fn write(&mut self, line: String) {
        write!(self.file, "{} \n", line).expect("Error while writing the scenario");
    }

    async fn parametrizable_scenario(
        &mut self,
        title: String,
        report_folder: String,
        precision: usize,
        transmission_delay: u64,
        consensus_duration: f64,
        probability_of_conflict: f64,
        nbr_clients: usize,
        nbr_replicas: usize,
        nbr_faulty_replicas: usize,
    ) -> Duration {
        let title_line = format!("SCENARIO: {} \n \t > PARAMS: \n\t\t - precision: (number of generated commands: {}, repetitions: {}) \n\t\t - transmission delay: {}ms \n\t\t - consensus duration: {}s \n\t\t - probability of conflict: {} \n\t\t - clients: {} \n\t\t - replica: {} (with faulty: {})", title, GENERATION_LOOP, precision, transmission_delay, consensus_duration, probability_of_conflict, nbr_clients, nbr_replicas + nbr_faulty_replicas, nbr_faulty_replicas);
        self.write(title_line);
        assert!(0.0 <= probability_of_conflict);
        assert!(probability_of_conflict <= 1.0);
        let proba = Uniform::new(0.0, 1.0);
        let rng = &mut thread_rng();

        let client_distr = Uniform::new(0.0, nbr_clients as f64);
        let mut simulations: Vec<Simulation> = Vec::new();
        for _ in 0..precision {
            let mut simulation = Simulation::new(
                title.clone(),
                report_folder.clone(),
                nbr_clients,
                nbr_replicas,
                nbr_faulty_replicas,
                transmission_delay,
                consensus_duration,
            )
            .await;
            for mut _i in 0..GENERATION_LOOP {
                let p = proba.sample(rng);
                let client = client_distr.sample(rng) as usize;
                if probability_of_conflict > 0.0 && p < probability_of_conflict {
                    // Add a conflicting command
                    simulation.get(client);
                    simulation.withdraw(client, 10);
                    _i += 1;
                    //   simulation.withdraw(id, amount)
                } else {
                    let action = proba.sample(rng);
                    if action < 0.2 {
                        simulation.get(client);
                    } else {
                        simulation.deposit(client, 10);
                    }
                }
            }

            simulations.push(simulation);
        }
        let time_taken = Self::simulate_average(&simulations).await;
        self.write(format!("\t > COMPLETE IN: {:#?}s", time_taken));

        time_taken
    }

    async fn parametrizable_scenario_with_pb(
        &mut self,
        title: String,
        report_folder: String,
        precision: usize,
        transmission_delay: u64,
        consensus_duration: f64,
        probability_of_conflict: u64,
        nbr_clients: usize,
        nbr_replicas: usize,
        nbr_faulty_replicas: usize,
    ) -> Duration {
        let title_line = format!("SCENARIO: {} \n \t > PARAMS: \n\t\t - precision: (number of generated commands: {}, repetitions: {}) \n\t\t - transmission delay: {}ms \n\t\t - consensus duration: {}s \n\t\t - probability of conflict: {}% \n\t\t - clients: {} \n\t\t - replica: {} (with faulty: {})", title, GENERATION_LOOP, precision, transmission_delay, consensus_duration, probability_of_conflict, nbr_clients, nbr_replicas + nbr_faulty_replicas, nbr_faulty_replicas);
        self.write(title_line);

        let mut simulations: Vec<Simulation> = Vec::new();
        for _ in 0..precision {
            let mut simulation = Simulation::new(
                title.clone(),
                report_folder.clone(),
                nbr_clients,
                nbr_replicas,
                nbr_faulty_replicas,
                transmission_delay,
                consensus_duration,
            )
            .await;
            let proba = Uniform::new(0.0, 1.0);
            let rng = &mut thread_rng();
            let indices: Vec<usize> = Vec::new();

            let conflict = GENERATION_LOOP - 2 * probability_of_conflict as usize;
            let mut i = 0;
            while i < GENERATION_LOOP {
                if i >= conflict {
                    // Add a conflicting command
                    simulation.get(i % nbr_clients);
                    simulation.withdraw(i % nbr_clients, 10);
                    i += 1;
                    //   simulation.withdraw(id, amount)
                } else {
                    let action = proba.sample(rng);
                    if action < 0.2 {
                        simulation.get(i % nbr_clients);
                    } else {
                        simulation.deposit(i % nbr_clients, 10);
                    }
                }

                i += 1;
            }

            simulations.push(simulation);
        }
        let time_taken = Self::simulate_average(&simulations).await;
        self.write(format!("\t > COMPLETE IN: {:#?}s", time_taken));

        time_taken
    }

    async fn simulate_average(simulations: &Vec<Simulation>) -> Duration {
        let nbr = simulations.len();
        let mut total_time_elapsed: f64 = 0.0;
        for simulation in simulations.iter() {
            let mut simulation = simulation.clone().await;
            let time_taken = simulation.simulate().await.as_secs_f64();
            total_time_elapsed += time_taken;
        }
        total_time_elapsed /= nbr as f64;

        Duration::from_secs_f64(total_time_elapsed)
    }

    fn write_sep(&mut self) {
        self.write(String::from("+---------------------------------------------------+"));
    }
    async fn with_inscreasing_transmission_delay(
        &mut self,
        low: u64,
        high: u64,
        step: u64,
        consensus_duration: f64,
        probability_of_conflict: f64,
    ) {
        let mut transmission_delay = low;
        self.write_sep();
        self.write(format!("START FUNCTION: f(transmission_delay) on [{}, {}]s with a step of {}s ",
            low, high, step
        ));
        self.write_sep();
        let mut scenarios: Vec<(u64, Duration)> = Vec::new();

        while transmission_delay < high {
            let scenario = self
                .parametrizable_scenario(
                    format!("INCREASING TRANSMISSION DELAY: {}ms", transmission_delay),
                    format!("{}/transmission_delay", DEFAULT_REPORT_FOLDER),
                    DEFAULT_PRECISION,
                    transmission_delay,
                    consensus_duration,
                    probability_of_conflict,
                    CLIENTS,
                    REPLICAS,
                    FAULTY,
                )
                .await;
            scenarios.push((transmission_delay, scenario));
            transmission_delay += step;
        }
        self.write(String::from("+---------------------------------------------------+"));
        self.write(format!("RESULTS: "));
        for (t, d) in scenarios {
            self.write(format!("\t f({}) = {:#?}", t, d));
        }
        self.write(String::from("+---------------------------------------------------+"));
    }

    fn consensus_duration(transmission_delay_ms: u64, slowdown_factor: f64) -> f64 {
        let consensus_duration_ms = transmission_delay_ms as f64 * slowdown_factor;
        consensus_duration_ms / 1000.0
    }
    /// Scenario: We seek for the relationship S between the transmission delay T and the consensus latency C such that the algorithm is worth it
    /// In other word, we seek for the S such that C = S*T and for a given probability of conflict P, the time taken with and without conflict is the same.
    /// We simulate it for P = 0.01, 0.05, 0.1, 0.2 and for P = 0.00 (no conflict)
    pub async fn slowdown_factor(&mut self, low: f64, high: f64, step: f64, transmission_delay: u64){
        let mut factor = low as f64;
        let probabilities: [u64; 5] = [0, 1, 5, 10, 20]; // In %
        self.write_sep();
        self.write(format!("START FUNCTION: f(slowdown_factor) on [{}, {}]s with a step of {}ms ",
            low, high, step
        ));
        self.write_sep();
        let mut scenarios: HashMap<u64, Vec<(f64, Duration)>> = HashMap::new();
        for p in probabilities {
            scenarios.insert(p, Vec::new());
        }
        while factor < high {
            for p in probabilities {
                println!("FACTOR {}, CONSENSUS {}, PROBABILITY {}%", factor, Self::consensus_duration(transmission_delay, factor), p);
                let scenario = self
                .parametrizable_scenario_with_pb(
                    format!("SLOWDOWN FACTOR (P = {}): {}", p, factor),
                    format!("{}", DEFAULT_REPORT_FOLDER),
                    DEFAULT_PRECISION,
                    transmission_delay,
                    Self::consensus_duration(transmission_delay, factor),
                    p,
                    CLIENTS,
                    REPLICAS,
                    FAULTY,
                )
                .await;
            scenarios.get_mut(&p).unwrap().push((factor, scenario));
            }
            factor *= step;
        }
        self.write(String::from("+---------------------------------------------------+"));
        self.write(format!("RESULTS: "));
        for (p, vec) in scenarios {
            for (t, d) in vec {
                self.write(format!("\t \t f_{}({}) = {:#?}", p, t, d));
            }
            self.write(String::from(""));
        }
        self.write(String::from("+---------------------------------------------------+"));
    }

    pub async fn expected_latency(&mut self, p: f64) {
        self.write_sep();
        self.write(format!("EXPECTED LATENCY WITH TRANSMISSION DELAY {}ms AND CONSENSUS DURATION {}s, P = {}",
            DEFAULT_TRANSMISSION_DELAY_MS, DEFAULT_CONSENSUS_DURATION_S, p
        ));
        self.write_sep();
        let scenario = self
            .parametrizable_scenario(
                format!("EXPECTED LATENCY WITH P_CONFLICT = {}", p),
                format!("{}", DEFAULT_REPORT_FOLDER),
                5,
                DEFAULT_TRANSMISSION_DELAY_MS,
                DEFAULT_CONSENSUS_DURATION_S,
                p,
                CLIENTS,
                REPLICAS,
                FAULTY,
            )
            .await;
        self.write(String::from("+---------------------------------------------------+"));
        self.write(format!("RESULTS: "));
        self.write(format!("\t DURATION FOR 100 COMMANDS : {:#?}", scenario));
        self.write(format!("\t EXPECTED LATENCY: {} cmds/s", scenario.as_secs_f64() / 100.0));
        self.write(String::from("+---------------------------------------------------+"));
    }
}

#[cfg(test)]
mod tests {
    use tokio::time::timeout;

    use super::*;

    #[tokio::test]
    async fn test() {
        test_bug().await;
    }
    async fn test_bug() {
        let mut scenarios: Scenarios = Scenarios::new("resources/test_bug.txt");
        let t = scenarios
            .parametrizable_scenario(
                String::from("test"),
                String::from("resources/test"),
                3,
                10,
                0.1,
                0.2,
                3,
                4,
                0,
            )
            .await;
        println!("{:?}", t);
    }

    #[tokio::test]
    async fn test2() {
        let mut i = 0;
        while i < 30 {
            i += 1;
            if let Err(_) = timeout(Duration::from_secs(100), test_bug()).await {
                panic!();
            }
        }
    }

    #[tokio::test]
    async fn test3() {
        let mut i = 0;
        while i < 30 {
            i += 1;
            if let Err(_) = timeout(Duration::from_secs(100), test_bug()).await {
                panic!();
            }
        }
    }

    #[test]
    fn poisson() {
        let dist = Uniform::new(0.0, 1.0);
        let rng = &mut thread_rng();
        let mut i = 0;
        let mut count10 = 0;
        let mut count50 = 0;
        let mut count1 = 0;
        let mut count5 = 0;
        while i < 100 {
            i += 1;
            let nbr = dist.sample(rng);
            if nbr < 0.1 {
                count10 += 1;
            }
            if nbr < 0.5 {
                count50 += 1;
            }
            if nbr < 0.01 {
                count1 += 1;
            }
            if nbr < 0.05 {
                count5 += 1;
            }
        }
        println!("1%: {}, 5%: {}, 10%: {}, 50%: {}", count1, count5, count10, count50);
    }

    #[tokio::test]
    async fn test_td() {
        let mut scenarios = Scenarios::new("reports/scenarios/transmission_delay");
        scenarios.with_inscreasing_transmission_delay(10, 50, 10, 1.0, 0.1).await;
    }

    #[tokio::test]
    async fn expected_latency() {
        let mut scenarios = Scenarios::new("reports/scenarios/expected_latency2.txt");
        scenarios.expected_latency(0.01).await;
        scenarios.expected_latency(0.05).await;
        scenarios.expected_latency(0.10).await;
        scenarios.expected_latency(0.20).await;
        scenarios.expected_latency(0.50).await;
    }
    
    #[tokio::test]
    async fn expected_latency3() {
        let mut scenarios = Scenarios::new("reports/scenarios/expected_latency3.txt");
        scenarios.expected_latency(0.1).await;
    }
}
