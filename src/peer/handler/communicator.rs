use std::{time::Duration};

use doomstack::Top;


use talk::{
    crypto::Identity,
    sync::fuse::Fuse,
    unicast::{Acknowledgement, SenderError},
};
use tokio::{
    task::JoinHandle,
    time::sleep,
};

use crate::{
    crypto::identity_table::IdentityTable,
    network::NetworkInfo,
    peer::{
        peer::PeerId,
    },
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
    T: UnicastMessage + Clone,
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

    pub fn key(&self) -> &Identity {
        &self.key
    }

    pub async fn send_message(
        &self,
        remote: Identity,
        message: T,
    ) -> Result<Acknowledgement, Top<SenderError>> {
        Self::transmit(self.network_info().transmition_delay()).await;
        self.sender.send(remote, message).await
    }

    pub async fn spawn_send(
        &self,
        remote: Identity,
        message: T,
    ) -> JoinHandle<Result<Acknowledgement, Top<SenderError>>> {
        let sender = self.sender.clone();
        let delay = self.network_info().transmition_delay();
        tokio::spawn(async move {
            Self::transmit(delay).await;
            sender.send(remote, message).await
        })
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

    async fn transmit(delay: u64) {
        sleep(Duration::from_millis(delay)).await;
    }

}

#[cfg(test)]
mod tests {

    use talk::{unicast::test::UnicastSystem, time::timeout};
    use tokio::join;

    use crate::{talk::{Message, FeedbackChannel}, network::network_info, tests::util::Utils, crypto::identity_table::IdentityTableBuilder};

    use super::*;

    async fn unicast_channel() -> (Identity, UnicastSender<Message>, UnicastReceiver<Message>) {
        let UnicastSystem {
            mut keys,
            mut senders,
            mut receivers,
        } = UnicastSystem::<Message>::setup(1).await;

        (keys.pop().unwrap(), senders.pop().unwrap(), receivers.pop().unwrap())
    }

    #[tokio::test]
    async fn spawning_test() {}

    #[tokio::test]
    async fn communication_test() {
        let network_info = NetworkInfo::new(0, 0, 0, 0, 1, 0);
        let (mut keys, mut senders, mut receivers) = Utils::mock_network(2).await;
        let (key, sender, _) = Utils::pop(&mut keys, &mut senders, &mut receivers);
        let (target, _, mut receiver) = Utils::pop(&mut keys, &mut senders, &mut receivers);
        let (rx, mut tx) = FeedbackChannel::channel();
        let communicator = Communicator::new(0, key.clone(), sender, rx, network_info.clone(), IdentityTableBuilder::new(network_info).build());
        let send = communicator.spawn_send(target.clone(), Message::Testing).await;
        
        let (id, recv, _) = receiver.receive().await;
        
        assert_eq!(id, key.clone());
        assert_eq!(recv, Message::Testing);

    
    }
}
