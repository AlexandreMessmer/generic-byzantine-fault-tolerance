use std::fmt::Display;
use std::hash::Hash;

use serde::{Deserialize, Serialize};

use crate::banking::banking::Money;

type Data = Money;
#[derive(Clone, Serialize, Deserialize, Debug, PartialOrd, Ord)]
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
        write!(f, "{:<64.64}", str)
    }
}

impl Eq for CommandResult {}

impl PartialEq for CommandResult {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Success(l0), Self::Success(r0)) => l0 == r0,
            (Self::Failure(_), Self::Failure(_)) => true,
            _ => false,
        }
    }
}

impl Hash for CommandResult {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            CommandResult::Success(_) => core::mem::discriminant(self).hash(state),
            _ => core::mem::discriminant(&ShadownFailure::Fail).hash(state),
        }
    }
}

enum ShadownFailure {
    Fail,
}
