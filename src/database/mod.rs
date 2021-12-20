pub mod client_database;
pub mod message_result_database;
pub mod request_database;

pub use self::message_result_database::MessageResultDatabase;
pub use self::request_database::RequestDatabase;

use std::collections::HashMap;

use crate::{
    error::{DatabaseError, InvalidRequest},
    talk::{Feedback, FeedbackSender, Message, MessageResult, RequestId},
};

type ResultHashMap = HashMap<MessageResult, usize>;
type MessageDatabase = HashMap<Message, usize>;
type DatabaseResult = Result<Option<usize>, DatabaseError>;
