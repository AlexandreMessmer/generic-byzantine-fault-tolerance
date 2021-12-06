use futures::{Future, future};
use tokio::runtime::Runtime;

use super::*;

impl FeedbackChannel {
    pub fn channel() -> (FeedbackSender, FeedbackReceiver) {
        let singleton = UnicastSystem::setup(1);
        let singleton = futures::executor::block_on(singleton);
        let UnicastSystem {
            mut keys,
            mut senders,
            mut receivers,
        } = singleton;
        let sender = FeedbackSender {
            id: keys.remove(0),
            sender: senders.remove(0),
        };

        let receiver = FeedbackReceiver {
            receiver: receivers.remove(0),
        };

        (sender, receiver)
    }
}