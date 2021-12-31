use std::collections::BTreeSet;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use talk::unicast::{Receiver as TalkReceiver, Sender as TalkSender};
use talk::{crypto::Identity, sync::fuse::Fuse, unicast::test::UnicastSystem};
use tokio::sync::mpsc;
use tokio::sync::oneshot::{self, Sender as OneshotSender};
use uuid::Uuid;

pub mod command;
pub mod transaction;
pub use command::Command;
pub use command::CommandResult;

use crate::database::replica_database::Set;

pub type CommandId = Uuid;
pub enum Instruction {
    Send(usize, Message),
    Execute(Command),
    Testing, // Only for testing purposes
    Broadcast(Message),
    Shutdown,
}

pub type RoundNumber = usize;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Phase {
    ACK,
    CHK,
}
/// Peers exchange Message.
/// This is defined to work for talk unicast systems.
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Message {
    Plaintext(String),
    Testing, // Only for debugging/testing purposes
    ACK(CommandId, Arc<Message>, CommandResult, RoundNumber),
    CHK(CommandId, Arc<Message>, CommandResult, RoundNumber),
    Command(Command),
    CommandAcknowledgement(Command, RoundNumber, CommandResult, Phase),
    ReplicaBroadcast(RoundNumber, Set, Phase),
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
pub enum Feedback {
    Error(String),
    Acknowledgement,
    Result(CommandResult),
}

#[derive(Debug)]
pub struct FeedbackChannel {}
impl FeedbackChannel {
    pub fn channel() -> (FeedbackSender, FeedbackReceiver) {
        mpsc::channel(32)
    }
}

pub type FeedbackSender = mpsc::Sender<Feedback>;

pub type FeedbackReceiver = mpsc::Receiver<Feedback>;
