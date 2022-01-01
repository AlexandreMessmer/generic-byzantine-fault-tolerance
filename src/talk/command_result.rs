use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::banking::banking::Money;

type Data = Money;
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum CommandResult {
    Success(Option<Data>),
    Failure(String),
}

impl Display for CommandResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandResult::Success(data) => match data {
                Some(amount) => write!(f, "SUCCESS < DATA: {}>", *amount),
                None => write!(f, "SUCCESS"),
            },
            CommandResult::Failure(reason) => write!(f, "FAILURE <{}>", *reason),
        }
    }
}
