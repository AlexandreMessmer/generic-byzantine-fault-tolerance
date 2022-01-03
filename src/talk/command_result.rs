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
        let str = match self {
            CommandResult::Success(data) => match data {
                Some(amount) => format!("SUCCESS <DATA: {}>", *amount),
                None => format!("SUCCESS"),
            },
            CommandResult::Failure(reason) => format!("FAILURE <{}>", *reason),
        };
        write!(f, "{:<32.32}", str)
    }
}
