use talk::crypto::Identity;

use crate::talk::message::Message;

use super::peer::PeerId;

pub trait Runner {
    fn send(&self, target: &Identity, message: Message);
    fn id(&self) -> PeerId;
}