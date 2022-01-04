use std::{
    ops::Range,
    time::{Duration, SystemTime, SystemTimeError},
};

use rand::thread_rng;
use rand_distr::{Distribution, Poisson};

pub const DEFAULT_CONSENSUS_DURATION: f64 = 10.0;
pub const DEFAULT_REPORT_FOLDER: &str = "resources";
pub const MAX_TRANSMISSION_DELAY: u64 = 1000;
#[derive(Clone, Debug)]
pub struct NetworkInfo {
    nbr_clients: usize,
    nbr_replicas: usize,
    nbr_faulty_clients: usize,
    nbr_faulty_replicas: usize,
    transmission_delay: u64,
    /// in ms
    consensus_duration: f64, // in s
    transmission_delay_distribution: Poisson<f64>, // In milliseconds
    n_ack: usize,
    report_folder: String,
    creation: SystemTime,
    write_logs: bool,
}

impl NetworkInfo {
    pub fn new(
        nbr_clients: usize,
        nbr_replicas: usize,
        nbr_faulty_clients: usize,
        nbr_faulty_replicas: usize,
        transmission_delay: u64,
        consensus_duration: f64,
        report_folder: String,
        n_ack: usize,
        write_logs: bool,
    ) -> Self {
        assert!(
            5 * nbr_faulty_replicas < nbr_replicas,
            "Network does not satisfy the resilience condition"
        );
        assert!(consensus_duration >= 0.0);
        Self {
            nbr_clients,
            nbr_replicas,
            nbr_faulty_clients,
            nbr_faulty_replicas,
            transmission_delay,
            consensus_duration,
            transmission_delay_distribution: Poisson::new(transmission_delay as f64)
                .unwrap_or(Poisson::new(1.0).unwrap()),
            n_ack,
            report_folder,
            creation: SystemTime::now(),
            write_logs,
        }
    }

    pub fn with_default_report_folder(
        nbr_clients: usize,
        nbr_replicas: usize,
        nbr_faulty_clients: usize,
        nbr_faulty_replicas: usize,
        transmission_delay: u64,
        n_ack: usize,
    ) -> Self {
        Self {
            nbr_clients,
            nbr_replicas,
            nbr_faulty_clients,
            nbr_faulty_replicas,
            transmission_delay,
            consensus_duration: DEFAULT_CONSENSUS_DURATION,
            transmission_delay_distribution: Poisson::new(transmission_delay as f64)
                .unwrap_or(Poisson::new(1.0).unwrap()),
            n_ack,
            report_folder: String::from(DEFAULT_REPORT_FOLDER),
            creation: SystemTime::now(),
            write_logs: false,
        }
    }

    pub fn default(
        nbr_clients: usize,
        nbr_replicas: usize,
        nbr_faulty_clients: usize,
        nbr_faulty_replicas: usize,
        transmission_delay: u64,
    ) -> Self {
        Self::with_default_report_folder(
            nbr_clients,
            nbr_replicas,
            nbr_faulty_clients,
            nbr_faulty_replicas,
            transmission_delay,
            nbr_replicas,
        )
    }

    pub fn default_parameters(
        nbr_clients: usize,
        nbr_replicas: usize,
        nbr_faulty_clients: usize,
        nbr_faulty_replicas: usize,
        transmission_delay: u64,
        consensus_duration: f64,
        report_folder: String,
    ) -> Self {
        assert!(nbr_replicas > 0);
        Self::new(
            nbr_clients,
            nbr_replicas,
            nbr_faulty_clients,
            nbr_faulty_replicas,
            transmission_delay,
            consensus_duration,
            report_folder,
            nbr_replicas,
            false,
        )
    }

    pub fn report_folder(&self) -> &String {
        &self.report_folder
    }
    pub fn nbr_clients(&self) -> usize {
        self.nbr_clients
    }

    pub fn nbr_replicas(&self) -> usize {
        self.nbr_replicas
    }

    pub fn size(&self) -> usize {
        self.nbr_clients + self.nbr_replicas + self.nbr_faulty_clients + self.nbr_faulty_replicas
    }

    pub fn nbr_faulty_clients(&self) -> usize {
        self.nbr_faulty_clients
    }

    pub fn nbr_faulty_replicas(&self) -> usize {
        self.nbr_faulty_replicas
    }

    pub fn n_ack(&self) -> usize {
        self.n_ack
    }

    pub fn f(&self) -> usize {
        self.nbr_faulty_replicas()
    }

    pub fn consensus_duration(&self) -> f64 {
        self.consensus_duration
    }

    pub fn write_logs(&self) -> bool {
        self.write_logs
    }
    pub fn set_write_logs(&mut self, value: bool) {
        self.write_logs = value;
    }

    pub fn compute_ranges(&self) -> (Range<usize>, Range<usize>, Range<usize>, Range<usize>) {
        let client_start: usize = 0;
        let client_end: usize = self.nbr_clients();
        let faulty_client_start: usize = client_end;
        let faulty_client_end: usize = faulty_client_start + self.nbr_faulty_clients();
        let replica_start: usize = faulty_client_end;
        let replica_end: usize = replica_start + self.nbr_replicas();
        let faulty_replica_start: usize = replica_end;
        let faulty_replica_end: usize = faulty_replica_start + self.nbr_faulty_replicas();

        (
            client_start..client_end,
            faulty_client_start..faulty_client_end,
            replica_start..replica_end,
            faulty_replica_start..faulty_replica_end,
        )
    }

    pub fn transmission_delay(&self) -> u64 {
        self.transmission_delay
    }
    pub fn sample_transmission_delay(&self) -> u64 {
        if self.transmission_delay == 0 {
            return 0;
        }
        let mut rng = thread_rng();
        let mut delay = self.transmission_delay_distribution.sample(&mut rng) as u64;
        while delay > MAX_TRANSMISSION_DELAY {
            delay = self.transmission_delay_distribution.sample(&mut rng) as u64;
        }
        delay
    }

    pub fn consensus_transmission_delay(&self) -> f64 {
        self.consensus_duration
    }
    pub fn elapsed(&self) -> Result<Duration, SystemTimeError> {
        self.creation.elapsed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_range_correclty() {
        let info = NetworkInfo::with_default_report_folder(5, 5, 2, 2, 1, 0);
        let (client_range, faulty_client_range, replica_range, faulty_replica_range) =
            info.compute_ranges();

        assert!(client_range.contains(&0));
        assert!(client_range.contains(&4));
        assert!(!client_range.contains(&5));

        assert!(faulty_client_range.contains(&5));
        assert!(faulty_client_range.contains(&6));
        assert!(!faulty_client_range.contains(&7));

        assert!(replica_range.contains(&7));
        assert!(replica_range.contains(&11));
        assert!(!replica_range.contains(&12));

        assert!(faulty_replica_range.contains(&12));
        assert!(faulty_replica_range.contains(&13));
        assert!(!faulty_replica_range.contains(&14));
    }

    #[test]
    fn poisson() {
        let distr = Poisson::new(2.0).unwrap();
        for _ in 0..10 {
            let delay = distr.sample(&mut thread_rng());
            println!("{:?}", delay);
        }
    }
}
