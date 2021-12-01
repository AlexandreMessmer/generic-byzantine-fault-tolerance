use serde::{Deserialize, Serialize};

use super::message::Message;

///
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
pub enum Feedback {
    Error(String),
    Acknowledgement,
    Res(Option<Result>),
}

type ResultContent = usize;
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
pub struct Result {
    res: ResultContent,
}

impl Result {
    pub fn new(value: ResultContent) -> Self {
        Result { res: value }
    }

    pub fn get(&self) -> ResultContent {
        self.res
    }
}
