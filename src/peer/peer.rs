use std::sync::Arc;

use doomstack::Top;
use talk::{
    crypto::Identity,
    sync::fuse::Fuse,
    unicast::{
        Acknowledgement, Message, Receiver as UnicastReceiver, Sender as UnicastSender, SenderError,
    },
};
use tokio::task::JoinHandle;

use super::{handler::Handler, runner::Runner};
use crate::{
    crypto::identity_table::IdentityTable,
    network::{
        network::Network,
        network_info::{self, NetworkInfo},
    },
    types::*,
};
pub type PeerId = usize;
pub struct Peer<T: UnicastMessage> {
    id: PeerId,
    key: Identity,
    sender: UnicastSender<T>,
    receiver: UnicastReceiver<T>,
    network_outlet: InstructionReceiver,
    network_info: NetworkInfo,
    identity_table: IdentityTable,
    handler: Box<dyn Handler<T>>,
    fuse: Fuse,
}

/// Structure that defines a `Peer`, i.e. an entity that can send and receive `Message`
/// Its behavior is defined by a `PeerRunner`

impl<T> Peer<T>
where
    T: UnicastMessage,
{
    /// Create a new `Peer`from the given arguments
    ///
    /// This is essentially a single unit of a `UnicastSystem` from talk crate
    pub fn new(
        id: PeerId,
        key: Identity,
        sender: UnicastSender<T>,
        receiver: UnicastReceiver<T>,
        network_outlet: InstructionReceiver,
        network_info: NetworkInfo,
        identity_table: IdentityTable,
        handler: Box<dyn Handler<T>>,
    ) -> Self {
        Peer {
            id,
            key,
            sender,
            receiver,
            network_outlet,
            network_info,
            identity_table,
            handler,
            fuse: Fuse::new(),
        }
    }

    pub fn key(&self) -> &Identity {
        &self.key
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

#[async_trait::async_trait]
impl<T> Runner<T> for Peer<T>
where
    T: Message,
{
    async fn run(mut self) {
        loop {
            let handler = &mut self.handler;
            tokio::select! {
                (id, message, acknowledger) = self.receiver.receive() => {
                    println!("Received something");
                    handler.handle_message(&self, id, message, acknowledger).await;
                }

                Some(instruction) = self.network_outlet.recv() => {
                    handler.handle_instruction(&self, instruction).await;
                }
            }
        }
    }
}

pub struct PeerInfo<T: UnicastMessage> {
    id: PeerId,
    key: Arc<Identity>,
    sender: Arc<UnicastSender<T>>,
    network_info: Arc<UnicastSender<T>>,
    identity_table: Arc<IdentityTable>,
}
