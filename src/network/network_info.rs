use std::{ops::Range, time::Duration};

#[derive(Clone)]
pub struct NetworkInfo {
    nbr_clients: usize,
    nbr_replicas: usize,
    nbr_faulty_clients: usize,
    nbr_faulty_replicas: usize,
    transmition_delay: Duration,
    n_ack: usize,
}

impl NetworkInfo {
    pub fn new(
        nbr_clients: usize,
        nbr_replicas: usize,
        nbr_faulty_clients: usize,
        nbr_faulty_replicas: usize,
        transmition_delay: Duration,
        n_ack: usize,
    ) -> Self {
        Self {
            nbr_clients,
            nbr_replicas,
            nbr_faulty_clients,
            nbr_faulty_replicas,
            transmition_delay,
            n_ack,
        }
    }

    pub fn nbr_clients(&self) -> usize {
        self.nbr_clients
    }

    pub fn nbr_replicas(&self) -> usize {
        self.nbr_replicas
    }

    pub fn size(&self) -> usize {
        self.nbr_clients + self.nbr_replicas
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

    pub fn transmition_delay(&self) -> &Duration {
        &self.transmition_delay
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn compute_range_correclty() {
        let info = NetworkInfo::new(5, 5, 2, 2, Duration::from_secs(1), 0);
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
}
