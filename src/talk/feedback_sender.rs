use super::*;

impl FeedbackSender {
    pub fn send_feedback(&self, message: Feedback, fuse: &Fuse) {
        self.sender.spawn_send(self.id.clone(), message, fuse);
    }
}