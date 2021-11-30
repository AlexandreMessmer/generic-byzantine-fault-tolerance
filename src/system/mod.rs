use std::collections::HashMap;

use crate::talk::{message::Message, RequestId};

use self::peer::PeerId;

pub mod byzantine_system;
pub mod client;
pub mod peer;
pub mod peer_runner;
pub mod peer_system;
pub mod replica;
pub mod settings;
pub mod runner;

pub enum PeerType {
    Client,
    Replica,
}

type RequestDatabase = HashMap<RequestId, MessageDatabase>;
type MessageDatabase = HashMap<Message, usize>;
pub struct Database {
    received: MessageDatabase,
    request: RequestDatabase,
}
impl Database {
    pub fn new() -> Self {
        Database {
            received: HashMap::new(),
            request: HashMap::new(),
        }
    }
}

use talk::unicast::{Receiver as TalkReceiver, Sender as TalkSender};

type PeerIdentifier = (PeerType, PeerId);
type Sender = TalkSender<Message>;
type Receiver = TalkReceiver<Message>;

