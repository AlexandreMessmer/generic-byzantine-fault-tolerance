use talk::{
    crypto::Identity,
    unicast::{Receiver, Sender},
};

use crate::talk::Message;

use crate::system::*;

/// Structure that defines a `Peer`, i.e. an entity that can send and receive `Message`
/// Its behavior is defined by a `PeerRunner`

impl Peer {
    /// Create a new `Peer`from the given arguments
    ///
    /// This is essentially a single unit of a `UnicastSystem` from talk crate
    pub fn new(
        id: PeerId,
        key: Identity,
        sender: Sender<Message>,
        receiver: Receiver<Message>,
    ) -> Peer {
        Peer {
            id,
            key,
            sender,
            receiver,
        }
    }
}
