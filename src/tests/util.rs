use futures::channel::mpsc::Receiver;
use network_info::NetworkInfo;
use talk::{unicast::test::UnicastSystem, crypto::Identity};

use crate::{network::network_info, peer::{handler::Communicator, peer::PeerId}, talk::{Message, FeedbackChannel, FeedbackReceiver}, crypto::identity_table::IdentityTableBuilder, types::*};

pub struct Utils;

impl Utils {
    pub async fn communicator(network_info: NetworkInfo, id: PeerId) -> (Communicator<Message>, UnicastReceiver<Message>, FeedbackReceiver) {
        let (key, sender, receiver) = Utils::unicast_channel().await;

        let (feedback_inlet, feedback_outlet) = FeedbackChannel::channel();
        let identity_table = IdentityTableBuilder::new(network_info.clone()).build();

        (Communicator::new(id, key, sender, feedback_inlet, network_info, identity_table), receiver, feedback_outlet)
    }

    pub async fn unicast_channel() -> (Identity, UnicastSender<Message>, UnicastReceiver<Message>) {
        let UnicastSystem {
            mut keys,
            mut senders,
            mut receivers,
        } = UnicastSystem::<Message>::setup(1).await;

        (keys.pop().unwrap(), senders.pop().unwrap(), receivers.pop().unwrap())
    }

    pub async fn mock_network(size: usize) -> (Vec<Identity>, Vec<UnicastSender<Message>>, Vec<UnicastReceiver<Message>>) {
        let UnicastSystem {
            keys, 
            senders,
            receivers
        } = UnicastSystem::<Message>::setup(size).await;
        (keys, senders, receivers)
    }

    pub fn pop(keys: &mut Vec<Identity>, senders: &mut Vec<UnicastSender<Message>>, receivers: &mut Vec<UnicastReceiver<Message>>) -> (Identity, UnicastSender<Message>, UnicastReceiver<Message>) {
        (keys.pop().unwrap(), senders.pop().unwrap(), receivers.pop().unwrap())
    }
}