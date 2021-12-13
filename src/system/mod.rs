

use crate::{
    settings::RunnerSettings,
    talk::{
        Instruction, Message,
        RequestId,
    },
};

pub mod byzantine_system;
pub mod peer_system;
pub mod runner;
pub use runner::*;

use talk::{
    crypto::Identity,
    sync::fuse::Fuse,
    unicast::{Receiver as TalkReceiver, Sender as TalkSender},
};
use tokio::sync::oneshot::Sender as OneShotSender;

// Definition of some senders and receivers
type InstructionSender = MPSCSender<Instruction>;
type InstructionReceiver = MPSCReceiver<Instruction>;
type Sender = TalkSender<Message>;
type Receiver = TalkReceiver<Message>;

use tokio::sync::mpsc::Receiver as MPSCReceiver;
use tokio::sync::mpsc::Sender as MPSCSender;

/// Define a system of peers. Command can be given to peer through the MPSC channel.
///
/// Each peer runs a `PeerRunner` that handles incoming request. We can pass information to each peer through
/// message handling (see https://tokio.rs/tokio/tutorial/channels).
///
/// The index of each peer runner inlet is the id of the peer, from 0 to size (excluded)
pub struct PeerSystem {
    size: usize,
    runner_inlets: Vec<InstructionSender>,
    fuse: Fuse,
}

#[cfg(test)]
mod tests {
    use std::{char, collections::HashMap};

    #[test]
    pub fn entry_test() {
        let mut map: HashMap<u32, char> = HashMap::new();
        map.insert(0, 'a');
        map.entry(0).and_modify(|c| {
            c.make_ascii_uppercase();
        });
        let zero = 0;
        println!("{:?}", map.get(&zero));
    }
}
