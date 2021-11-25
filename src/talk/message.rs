use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum Message {
    Plaintext(String),
    ACK(Uuid, ),
    CHK(Uuid),
}
