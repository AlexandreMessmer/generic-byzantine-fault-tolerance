use super::{Command};

pub enum Instruction {
    Execute(Command),
    Testing, // Only for testing purposes
    Shutdown,
}
