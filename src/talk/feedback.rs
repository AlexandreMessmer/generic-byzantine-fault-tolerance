use serde::{Deserialize, Serialize};

use super::message::Message;

///
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
pub enum Feedback{
    Error(String),
    Acknowledgement,
    Res(Result),
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
pub struct Result{
    res: u64,
}