pub mod peer;
pub mod peer_runner;
pub mod peer_system;

#[derive(Debug)]
pub enum Command {
    Send(usize, Message),
}

use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum Message {
    Plaintext(String),
}
