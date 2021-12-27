use doomstack::Top;
use talk::{
    crypto::Identity,
    sync::fuse::Fuse,
    unicast::{Acknowledgement, SenderError},
};
use tokio::task::JoinHandle;

use crate::{
    crypto::identity_table::IdentityTable,
    network::NetworkInfo,
    peer::peer::PeerId,
    talk::{Feedback, FeedbackSender},
    types::*,
};

pub struct Communicator<T>
where
    T: UnicastMessage,
{
    id: PeerId,
    key: Identity,
    sender: UnicastSender<T>,
    feedback_inlet: FeedbackSender,
    network_info: NetworkInfo,
    identity_table: IdentityTable,
    fuse: Fuse,
}

impl<T> Communicator<T>
where
    T: UnicastMessage,
{
    pub fn new(
        id: PeerId,
        key: Identity,
        sender: UnicastSender<T>,
        feedback_inlet: FeedbackSender,
        network_info: NetworkInfo,
        identity_table: IdentityTable,
    ) -> Self {
        Communicator {
            id,
            key,
            sender,
            feedback_inlet,
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

    // Sends the feedback on the current thread
    pub async fn send_feedback(
        &self,
        feedback: Feedback,
    ) -> Result<(), tokio::sync::mpsc::error::SendError<Feedback>> {
        self.feedback_inlet.send(feedback).await
    }

    /// Spawns a tokio Task to send the feedback
    pub fn spawn_send_feedback(
        &self,
        feedback: Feedback,
    ) -> JoinHandle<Result<(), tokio::sync::mpsc::error::SendError<Feedback>>> {
        let sender = self.feedback_inlet.clone();
        tokio::spawn(async move { sender.send(feedback).await })
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
