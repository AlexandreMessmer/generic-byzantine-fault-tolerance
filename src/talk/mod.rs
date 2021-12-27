use std::sync::Arc;

use serde::{Deserialize, Serialize};
use talk::unicast::{Receiver as TalkReceiver, Sender as TalkSender};
use talk::{crypto::Identity, sync::fuse::Fuse, unicast::test::UnicastSystem};
use tokio::sync::mpsc;
use tokio::sync::oneshot::{self, Sender as OneshotSender};
use uuid::Uuid;

pub mod request;
pub use request::Request;

pub type RequestId = Uuid;
pub enum Instruction {
    Send(usize, Message),
    Execute(Message, RequestId),
    Testing, // Only for testing purposes
    Broadcast(Message),
    Shutdown,
}
impl Instruction {
    pub fn execute(message: Message) -> Instruction {
        let id = Uuid::new_v4();
        Instruction::Execute(message, id)
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
impl FeedbackChannel {
    pub fn channel() -> (FeedbackSender, FeedbackReceiver) {
        mpsc::channel(32)
    }
}

pub type FeedbackSender = mpsc::Sender<Feedback>;

pub type FeedbackReceiver = mpsc::Receiver<Feedback>;
