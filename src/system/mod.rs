pub mod peer;
pub mod peer_runner;
pub mod peer_system;
pub enum Command{
    Send(usize, Message)
}

use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub enum Message{
    Plaintext(String)
}