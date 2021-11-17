use talk::{crypto::{Identity}, unicast::{Receiver, Sender}};
use crate::system::{Message};

/// Structure that defines a `Peer`, i.e. an entity that can send and receive `Message` 
/// Its behavior is defined by a `PeerRunner`

pub type PeerId = usize;
pub struct Peer{
    pub id: PeerId,
    pub key: Identity,
    pub sender: Sender<Message>,
    pub receiver: Receiver<Message>,
}

impl Peer{

    /// Create a new `Peer`from the given arguments
    /// 
    /// This is essentially a single unit of a `UnicastSystem` from talk crate
    pub fn new(id: PeerId, key: Identity, sender: Sender<Message>, receiver: Receiver<Message>) -> Peer{
        Peer{
            id,
            key,
            sender, 
            receiver,
        }
    }

    pub fn key(&self) -> Identity {
        self.key.clone()
    }
}
