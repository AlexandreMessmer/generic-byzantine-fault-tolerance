use super::*;
impl FeedbackSender {
    pub fn send_feedback(&self, message: Feedback) {
        self.sender.spawn_send(self.id.clone(), message, &self.fuse);
    }
}
