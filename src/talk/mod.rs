use std::rc::Rc;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use talk::unicast::{Receiver as TalkReceiver, Sender as TalkSender};
use talk::{crypto::Identity, sync::fuse::Fuse, unicast::test::UnicastSystem};
use tokio::sync::oneshot::Sender as OneshotSender;
use uuid::Uuid;

pub mod feedback_channel;
pub mod feedback_receiver;
pub mod feedback_sender;
pub type RequestId = Uuid;
pub type Instruction = (Command, FeedbackSender);
pub enum Command {
    Send(usize, Message),
    Execute(Message, RequestId),
    Testing(Option<OneshotSender<Command>>), // Only for testing purposes
    AskStatus(Message, FeedbackSender),
    Answer,
}
impl Command {
    pub fn execute(message: Message) -> Command {
        let id = Uuid::new_v4();
        Command::Execute(message, id)
    }
}

pub type RoundNumber = usize;
/// Peers exchange Message.
/// This is defined to work for talk unicast systems.
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
pub enum Message {
    Plaintext(String),
    Testing, // Only for debugging/testing purposes
    ACK(RequestId, Arc<Message>, MessageResult, RoundNumber),
    CHK(RequestId, Arc<Message>, MessageResult, RoundNumber),
    Broadcast(Arc<Message>),
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
pub enum Feedback {
    Error(String),
    Acknowledgement,
    Result(MessageResult),
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
pub struct MessageResult {}

#[derive(Debug)]
pub struct FeedbackChannel {}

pub struct FeedbackSender {
    id: Identity,
    sender: TalkSender<Feedback>,
    fuse: Fuse,
}

pub struct FeedbackReceiver {
    receiver: TalkReceiver<Feedback>,
}
