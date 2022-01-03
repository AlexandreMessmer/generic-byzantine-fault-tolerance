use super::{Command, Message};

pub enum Instruction {
    Execute(Command),
    Testing, // Only for testing purposes
    Shutdown,
}
