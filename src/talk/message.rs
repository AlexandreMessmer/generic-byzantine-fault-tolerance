use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::database::replica_database::Set;

use super::{Command, CommandId, CommandResult, Phase, RoundNumber};

/// Peers exchange Message.
/// This is defined to work for talk unicast systems.
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Message {
    Testing, // Only for debugging/testing purposes
    Command(Command),
    CommandAcknowledgement(Command, RoundNumber, CommandResult, Phase),
    ReplicaBroadcast(RoundNumber, Set, Phase),
}
