use super::*;
impl FeedbackReceiver {
    pub async fn receive_feedback(&mut self) -> Feedback {
        let (_, feedback, _) = self.receiver.receive().await;
        feedback
    }
}
