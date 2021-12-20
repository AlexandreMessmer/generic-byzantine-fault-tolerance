pub use talk::unicast::{
    Message as UnicastMessage, Receiver as UnicastReceiver, Sender as UnicastSender,
};
pub use tokio::sync::mpsc::{Receiver as MPSCReceiver, Sender as MPSCSender};
use uuid::Uuid;

use crate::talk::Instruction;

pub type InstructionSender = MPSCSender<Instruction>;
pub type InstructionReceiver = MPSCReceiver<Instruction>;