use serde::{Deserialize, Serialize};

use crate::peer::{peer::PeerId, Peer};

use super::CommandResult;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
pub enum Feedback {
    Error(PeerId, String),
    Acknowledgement(PeerId),
    Result(PeerId, CommandResult),
    ShutdownComplete(PeerId),
}

impl Feedback {
    pub fn from(&self) -> PeerId {
        match self {
            Feedback::Error(id, _) => *id,
            Feedback::Acknowledgement(id) => *id,
            Feedback::Result(id, _) => *id,
            Feedback::ShutdownComplete(id) => *id,
        }
    }
}
