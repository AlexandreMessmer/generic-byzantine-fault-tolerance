use doomstack::Top;
use talk::{
    crypto::Identity,
    sync::fuse::Fuse,
    unicast::{Acknowledgement, SenderError},
};
use tokio::task::JoinHandle;

use crate::{
    crypto::identity_table::IdentityTable, network::NetworkInfo, peer::peer::PeerId, types::*,
};

pub struct PeerHandler<T>
where
    T: UnicastMessage,
{
    id: PeerId,
    key: Identity,
    sender: UnicastSender<T>,
    network_info: NetworkInfo,
    identity_table: IdentityTable,
    fuse: Fuse,
}

impl<T> PeerHandler<T>
where
    T: UnicastMessage,
{
    pub fn new(
        id: PeerId,
        key: Identity,
        sender: UnicastSender<T>,
        network_info: NetworkInfo,
        identity_table: IdentityTable,
    ) -> Self {
        PeerHandler {
            id,
            key,
            sender,
            network_info,
            identity_table,
            fuse: Fuse::new(),
        }
    }

    pub async fn send_message(
        &self,
        remote: Identity,
        message: T,
    ) -> Result<Acknowledgement, Top<SenderError>> {
        self.sender.send(remote, message).await
    }

    pub fn spawn_send(
        &self,
        remote: Identity,
        message: T,
    ) -> JoinHandle<Option<Result<Acknowledgement, Top<SenderError>>>> {
        self.sender.spawn_send(remote, message, &self.fuse)
    }

    pub fn id(&self) -> &usize {
        &self.id
    }

    pub fn identity_table(&self) -> &IdentityTable {
        &self.identity_table
    }

    pub fn network_info(&self) -> &NetworkInfo {
        &self.network_info
    }
}
