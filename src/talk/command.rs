use tokio::sync::oneshot::{Sender as OneshotSender, Receiver as OneshotReceiver};
use super::{RequestId, feedback::Feedback, message::Message};
type FeedbackSender = OneshotSender<Feedback>;

/// A command is a type of message for message passing.
/// It is only used to send command to a runner
#[derive(Debug)]
pub enum Command {
    Send(usize, Message),
    SendAndWait(usize, Message, FeedbackSender),
    Execute(Message, RequestId, FeedbackSender),
    Testing(Option<OneshotSender<Command>>), // Only for testing purposes
    Answer,
}
