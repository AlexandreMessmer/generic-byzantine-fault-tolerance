use super::*;

impl FeedbackChannel {
    pub async fn channel() -> (FeedbackSender, FeedbackReceiver) {
        let UnicastSystem {
            mut keys,
            mut senders,
            mut receivers,
        } = UnicastSystem::<Feedback>::setup(1).await.into();
        let key = keys.pop().unwrap();
        let sender = senders.pop().unwrap();
        let receiver = receivers.pop().unwrap();
        let fb_sender = FeedbackSender {
            id: key,
            sender,
            fuse: Fuse::new(),
        };
        let fb_receiver = FeedbackReceiver { receiver };
        (fb_sender, fb_receiver)
    }
}

#[cfg(test)]
pub mod tests {

    use super::*;
    use crate::talk::FeedbackChannel;
    #[tokio::test]
    async fn channel_feedback() {
        let (tx, mut rx) = FeedbackChannel::channel().await;
        let sent = Feedback::Error(String::from("Test"));
        let s1 = sent.clone();
        tx.send_feedback(s1);
        let feedback = rx.receive_feedback().await;
        assert_eq!(feedback, sent);
    }
}
