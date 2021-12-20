use doomstack::Top;
use talk::{
    crypto::Identity,
    unicast::{Acknowledgement, Acknowledger, Receiver, Sender, SenderError},
};
use tokio::task::JoinHandle;

use super::*;
use crate::talk::Message;

/// Structure that defines a `Peer`, i.e. an entity that can send and receive `Message`
/// Its behavior is defined by a `PeerRunner`

impl Peer {
    /// Create a new `Peer`from the given arguments
    ///
    /// This is essentially a single unit of a `UnicastSystem` from talk crate
    pub fn new(
        id: PeerId,
        key: Identity,
        sender: Sender<Message>,
        receiver: Receiver<Message>,
    ) -> Peer {
        Peer {
            id,
            key,
            sender,
            receiver,
        }
    }

    pub fn key(&self) -> &Identity {
        &self.key
    }

    pub fn spawn_send(
        &self,
        remote: Identity,
        message: Message,
        fuse: &Fuse,
    ) -> JoinHandle<Option<Result<Acknowledgement, Top<SenderError>>>> {
        self.sender.spawn_send(remote, message, fuse)
    }

    pub async fn send(
        &self,
        remote: Identity,
        message: Message,
    ) -> Result<Acknowledgement, SenderError> {
        self.sender
            .send(remote, message)
            .await
            .map_err(|_| SenderError::SendFailed)
    }
}

#[async_trait::async_trait]
impl Communicator for Peer {
    async fn send_message(
        &self,
        remote: Identity,
        message: Message,
    ) -> Result<Acknowledgement, Top<SenderError>> {
        self.sender.send(remote, message).await
    }
    async fn receive_message(&mut self) -> (Identity, Message, Acknowledger) {
        self.receiver.receive().await
    }

    fn spawn_send_message(
        &self,
        remote: Identity,
        message: Message,
        fuse: &Fuse,
    ) -> JoinHandle<Option<Result<Acknowledgement, Top<SenderError>>>> {
        self.sender.spawn_send(remote, message, fuse)
    }
}

impl Identifiable for Peer {
    fn id(&self) -> PeerId {
        self.id
    }
}
