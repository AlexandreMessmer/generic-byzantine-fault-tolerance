use std::collections::BTreeSet;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use talk::unicast::{Receiver as TalkReceiver, Sender as TalkSender};
use talk::{crypto::Identity, sync::fuse::Fuse, unicast::test::UnicastSystem};
use tokio::sync::mpsc;
use tokio::sync::oneshot::{self, Sender as OneshotSender};
use uuid::Uuid;

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
