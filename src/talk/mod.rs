use serde::{Serialize, Deserialize};
use talk::{crypto::Identity, unicast::{test::UnicastSystem, }, sync::fuse::Fuse};
use tokio::{sync::oneshot::Sender as OneshotSender};
use talk::unicast::{Sender as TalkSender, Receiver as TalkReceiver};
use uuid::Uuid;

pub mod feedback_channel;
pub mod feedback_sender;
pub mod feedback_receiver;
pub type RequestId = Uuid;
pub type Instruction = (Command, FeedbackSender);
pub enum Command {
    Send(usize, Message),
    Execute(Message, RequestId, FeedbackSender),
    Testing(Option<OneshotSender<Command>>), // Only for testing purposes
    AskStatus(Message, FeedbackSender),
    Answer,
}

/// Peers exchange Message.
/// This is defined to work for talk unicast systems.
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
pub enum Message {
    Plaintext(String),
    Testing, // Only for debugging/testing purposes
    ACK(RequestId, MessageResult),
    CHK(RequestId),
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
pub enum Feedback{
    Error(String),
    Acknowledgement,
    Res(Option<MessageResult>),
}


#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
pub struct MessageResult {}

pub struct FeedbackReceiver {
    receiver: TalkReceiver<Feedback>,
}

pub struct FeedbackSender {
    id: Identity,
    sender: TalkSender<Feedback>,
}

pub struct FeedbackChannel {}
