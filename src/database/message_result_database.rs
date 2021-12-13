use super::*;

pub struct MessageResultDatabase {
    results: MessageDatabase,
    feedback_inlet: FeedbackSender,
}

impl MessageResultDatabase {
    pub fn new(feedback_inlet: FeedbackSender) -> Self {
        MessageResultDatabase {
            results: HashMap::new(),
            feedback_inlet,
        }
    }

    pub fn add(&mut self, key: Message) -> Option<usize> {
        let value = self.results.get(&key).map_or(1, |x| x + 1);
        self.results.insert(key, value)
    }

    pub fn send_feedback(&self, feedback: Feedback) {
        self.feedback_inlet.send_feedback(feedback);
    }

    /// Consume the database to return the feedback inlet
    pub fn feedback_inlet(self) -> FeedbackSender {
        self.feedback_inlet
    }

    pub fn size(&self) -> usize {
        self.results.len()
    }

    pub fn message_number(&self, message: &Message) -> Option<usize> {
        self.results
            .get(message)
            .map(|size| size.clone())
    }
}

#[cfg(test)]
mod tests {

    use crate::talk::FeedbackChannel;

    use super::*;

    #[tokio::test]
    async fn construction_test() {
        let (tx, mut rx) = FeedbackChannel::channel().await;
        let db = MessageResultDatabase::new(tx);
        let inlet = db.feedback_inlet();
        inlet.send_feedback(Feedback::Error(String::from("Test")));
        let feedback = rx.receive_feedback();

        let res = tokio::join!(feedback).0;
        assert_eq!(res, Feedback::Error(String::from("Test")));
    }

    #[tokio::test]
    async fn add_test() {
        let (tx, _) = FeedbackChannel::channel().await;
        let mut db = MessageResultDatabase::new(tx);
        for _ in 0..10 {
            db.add(Message::Testing);
        }

        let res = db.add(Message::Testing);
        assert_eq!(res.unwrap(), 10);

        let res2 = db.add(Message::Plaintext(String::from("Hello")));
        assert!(res2 == None);
    }
}