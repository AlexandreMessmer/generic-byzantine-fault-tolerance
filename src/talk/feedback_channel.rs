use super::*;
pub struct FeedbackChannel {}
impl FeedbackChannel {
    pub async fn channel() -> (FeedbackSender, FeedbackReceiver) {
        let UnicastSystem {
            mut keys,
            mut senders,
            mut receivers,
        } = UnicastSystem::setup(1).await.into();

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