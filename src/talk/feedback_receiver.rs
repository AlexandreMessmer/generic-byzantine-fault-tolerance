use super::*;
pub struct FeedbackReceiver {
    receiver: TalkReceiver<Feedback>,
}
impl FeedbackReceiver {
    pub async fn receive_feedback(&mut self) -> Feedback {
        let (_, feedback, _) = self.receiver.receive().await;
        feedback
    }
}