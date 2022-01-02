use serde::{Deserialize, Serialize};

use tokio::sync::mpsc;

use uuid::Uuid;

pub mod command;
pub mod command_result;
pub mod feedback;
pub mod instruction;
pub mod message;

pub use command::Command;
pub use command_result::CommandResult;
pub use feedback::Feedback;
pub use instruction::Instruction;
pub use message::Message;

pub type CommandId = Uuid;
pub type RoundNumber = usize;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Phase {
    ACK,
    CHK,
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
