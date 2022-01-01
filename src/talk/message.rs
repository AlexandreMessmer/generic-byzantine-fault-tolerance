
use std::sync::Arc;

use serde::{Deserialize, Serialize};






use crate::database::replica_database::Set;

use super::{Command, CommandId, CommandResult, Phase, RoundNumber};

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
