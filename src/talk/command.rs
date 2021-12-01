use super::{feedback::Feedback, message::Message, RequestId};
use tokio::sync::oneshot::{Receiver as OneshotReceiver, Sender as OneshotSender};
type FeedbackSender = OneshotSender<Feedback>;

/// A command is a type of message for message passing.
/// It is only used to send command to a runner
#[derive(Debug)]
pub enum Command {
    Send(usize, Message),
    Execute(Message, RequestId, FeedbackSender),
    Testing(Option<OneshotSender<Command>>), // Only for testing purposes
    AskStatus(Message, FeedbackSender),
    Answer,
}
