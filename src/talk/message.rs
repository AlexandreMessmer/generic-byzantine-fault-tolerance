use serde::{Deserialize, Serialize};

use super::{RequestId, feedback::Feedback};

/// Peers exchange Message.
/// This is defined to work for talk unicast systems.
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
pub enum Message {
    Plaintext(String),
    Testing, // Only for debugging/testing purposes
    ACK(RequestId, MessageResult),
    CHK(RequestId),
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
pub struct MessageResult {

}