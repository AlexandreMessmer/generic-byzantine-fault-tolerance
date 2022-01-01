









use super::{Command, Message};

pub enum Instruction {
    Send(usize, Message),
    Execute(Command),
    Testing, // Only for testing purposes
    Broadcast(Message),
    Shutdown,
}
