pub use talk::unicast::{
    Message as UnicastMessage, Receiver as UnicastReceiver, Sender as UnicastSender,
};
pub use tokio::sync::{
    broadcast::{Receiver as BroadcastReceiver, Sender as BroadcastSender},
    mpsc::{Receiver as MPSCReceiver, Sender as MPSCSender},
};

use crate::talk::Instruction;

pub type InstructionSender = MPSCSender<Instruction>;
pub type InstructionReceiver = MPSCReceiver<Instruction>;
