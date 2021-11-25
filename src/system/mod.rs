use self::peer::PeerId;

pub mod byzantine_system;
pub mod client;
pub mod peer;
pub mod peer_runner;
pub mod peer_system;
pub mod replica;
pub mod settings;

pub enum PeerType {
    Client,
    Replica,
}
type PeerIdentifier = (PeerType, PeerId);
