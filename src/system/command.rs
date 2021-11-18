use tokio::sync::oneshot;

use super::message::Message;

#[derive(Debug)]
pub enum Command {
    Send(usize, Message),
    SendAndWait(usize, Message, oneshot::Sender<Message>),
}