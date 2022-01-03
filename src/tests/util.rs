use network_info::NetworkInfo;
use talk::{crypto::Identity, unicast::test::UnicastSystem};

use crate::{
    crypto::identity_table::IdentityTableBuilder,
    network::network_info,
    peer::{handler::Communicator, peer::PeerId},
    talk::{FeedbackChannel, FeedbackReceiver, Message},
    types::*,
};

pub struct Utils;

impl Utils {
    pub async fn communicator(
        network_info: NetworkInfo,
        id: PeerId,
    ) -> (
        Communicator<Message>,
        UnicastReceiver<Message>,
        FeedbackReceiver,
    ) {
        let (key, sender, receiver) = Utils::unicast_channel().await;

        let (feedback_inlet, feedback_outlet) = FeedbackChannel::channel();
        let identity_table = IdentityTableBuilder::new(network_info.clone()).build();

        (
            Communicator::new(
                id,
                key,
                sender,
                feedback_inlet,
                network_info,
                identity_table,
            ),
            receiver,
            feedback_outlet,
        )
    }

    pub async fn unicast_channel() -> (Identity, UnicastSender<Message>, UnicastReceiver<Message>) {
        let UnicastSystem {
            mut keys,
            mut senders,
            mut receivers,
        } = UnicastSystem::<Message>::setup(1).await;

        (
            keys.pop().unwrap(),
            senders.pop().unwrap(),
            receivers.pop().unwrap(),
        )
    }

    pub async fn mock_network(
        size: usize,
    ) -> (
        Vec<Identity>,
        Vec<UnicastSender<Message>>,
        Vec<UnicastReceiver<Message>>,
    ) {
        let UnicastSystem {
            keys,
            senders,
            receivers,
        } = UnicastSystem::<Message>::setup(size).await;
        (keys, senders, receivers)
    }

    pub fn pop(
        keys: &mut Vec<Identity>,
        senders: &mut Vec<UnicastSender<Message>>,
        receivers: &mut Vec<UnicastReceiver<Message>>,
    ) -> (Identity, UnicastSender<Message>, UnicastReceiver<Message>) {
        (
            keys.pop().unwrap(),
            senders.pop().unwrap(),
            receivers.pop().unwrap(),
        )
    }

    pub fn pop_from_network(
        system: &mut UnicastSystem<Message>,
    ) -> (Identity, UnicastSender<Message>, UnicastReceiver<Message>) {
        (
            system.keys.pop().unwrap(),
            system.senders.pop().unwrap(),
            system.receivers.pop().unwrap(),
        )
    }
}
