use doomstack::Top;
use talk::{crypto::Identity, unicast::{Sender as UnicastSender, Receiver as UnicastReceiver, Message, Acknowledgement, SenderError}, sync::fuse::Fuse};
use tokio::task::JoinHandle;

use super::{handler::Handler, runner::Runner};
use crate::{types::*, crypto::identity_table::IdentityTable, system::network_info::{NetworkInfo, self}};
pub type PeerId = usize;
pub struct Peer<T: UnicastMessage> {
    id: PeerId,
    key: Identity,
    sender: UnicastSender<T>,
    receiver: UnicastReceiver<T>,
    network_outlet: InstructionReceiver,
    network_info: NetworkInfo,
    handler: Box<dyn Handler<T>>,
    fuse: Fuse,
}

/// Structure that defines a `Peer`, i.e. an entity that can send and receive `Message`
/// Its behavior is defined by a `PeerRunner`

impl<T> Peer<T> where T: UnicastMessage {
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
        handler: Box<dyn Handler<T>>,
        fuse: Fuse,
    ) -> Self {
        Peer {
            id,
            key,
            sender,
            receiver,
            network_outlet,
            network_info,
            handler,
            fuse
        }
    }

    pub fn key(&self) -> &Identity {
        &self.key
    }

    pub async fn send_message(&self, remote: Identity, message: T) -> Result<Acknowledgement, Top<SenderError>> {
        self.sender
            .send(remote, message)
            .await
    }

    pub fn spawn_send(&self, remote: Identity, message: T) -> JoinHandle<Option<Result<Acknowledgement, Top<SenderError>>>> {
        self.sender
            .spawn_send(remote, message, &self.fuse)
    }

    pub fn id(&self) -> &usize {
        &self.id
    }
}

#[async_trait::async_trait]
impl<T> Runner<T> for Peer<T> where T: Message {
    async fn run(&mut self) {
        loop {
            tokio::select! {
                (id, message, acknowledger) = self.receiver.receive() => {
                    println!("Received something");
                    self.handler.handle_message(id, message, acknowledger).await;
                }

                Some(instruction) = self.network_outlet.recv() => {
                    self.handler.handle_command(instruction).await;
                }
            }
        }
    }
}