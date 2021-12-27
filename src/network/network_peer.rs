use std::ops::Range;

#[derive(Debug, Clone, Copy)]
pub enum NetworkPeer {
    Client,
    FaultyClient,
    Replica,
    FaultyReplica,
}

impl NetworkPeer {
    pub fn get_corresponding_type(
        id: &usize,
        client_range: &Range<usize>,
        faulty_client_range: &Range<usize>,
        replica_range: &Range<usize>,
        faulty_replica_range: &Range<usize>,
    ) -> Option<Self> {
        if client_range.contains(id) {
            return Some(NetworkPeer::Client);
        } else if faulty_client_range.contains(id) {
            return Some(NetworkPeer::FaultyClient);
        } else if replica_range.contains(id) {
            return Some(NetworkPeer::Replica);
        } else if faulty_replica_range.contains(id) {
            return Some(NetworkPeer::FaultyReplica);
        }

        None
    }
}
