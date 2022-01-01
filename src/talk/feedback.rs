use serde::{Deserialize, Serialize};

use super::CommandResult;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
pub enum Feedback {
    Error(String),
    Acknowledgement,
    Result(CommandResult),
}
