use std::collections::BTreeSet;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use talk::unicast::{Receiver as TalkReceiver, Sender as TalkSender};
use talk::{crypto::Identity, sync::fuse::Fuse, unicast::test::UnicastSystem};
use tokio::sync::mpsc;
use tokio::sync::oneshot::{self, Sender as OneshotSender};
use uuid::Uuid;

use super::{Command, Message};

pub enum Instruction {
    Send(usize, Message),
    Execute(Command),
    Testing, // Only for testing purposes
    Broadcast(Message),
    Shutdown,
}
