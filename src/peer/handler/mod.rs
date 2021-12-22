use crate::{
    crypto::identity_table::IdentityTable,
    network::{NetworkInfo, NetworkPeer},
    peer::Peer,
    talk::Message,
};
use talk::{crypto::Identity, unicast::Acknowledger};
pub mod client_handler;
pub mod faulty_client_handler;
pub mod faulty_replica_handler;
pub mod peer_handler;
pub mod replica_handler;

pub use client_handler::ClientHandler;
pub use faulty_client_handler::FaultyClientHandler;
pub use faulty_replica_handler::FaultyReplicaHandler;
pub use peer_handler::PeerHandler;
pub use replica_handler::ReplicaHandler;
use tokio::runtime::Handle;

use crate::{talk::Instruction, types::*};

use super::peer::PeerId;

#[async_trait::async_trait]
pub trait Handler<T>: Sync + Send
where
    T: UnicastMessage,
{
    async fn handle_message(&mut self, id: Identity, message: T, ack: Acknowledger);
    async fn handle_instruction(&mut self, instruction: Instruction);

    fn id(&self) -> &PeerId;
    fn network_info(&self) -> &NetworkInfo;
}

pub struct HandlerBuilder {}
impl HandlerBuilder {
    fn get_corresponding_handler(
        peer_type: NetworkPeer,
        peer_handler: PeerHandler<Message>,
    ) -> Box<dyn Handler<Message>> {
        match peer_type {
            NetworkPeer::Client => Box::new(ClientHandler::new(peer_handler)),
            NetworkPeer::FaultyClient => Box::new(FaultyClientHandler::new(peer_handler)),
            NetworkPeer::Replica => Box::new(ReplicaHandler::new(peer_handler)),
            NetworkPeer::FaultyReplica => Box::new(FaultyReplicaHandler::new(peer_handler)),
        }
    }

    pub fn handler(
        peer_type: NetworkPeer,
        id: PeerId,
        key: Identity,
        sender: UnicastSender<Message>,
        network_info: NetworkInfo,
        identity_table: IdentityTable,
    ) -> Box<dyn Handler<Message>> {
        let peer_handler = PeerHandler::new(id, key, sender, network_info, identity_table);
        Self::get_corresponding_handler(peer_type, peer_handler)
    }
}
