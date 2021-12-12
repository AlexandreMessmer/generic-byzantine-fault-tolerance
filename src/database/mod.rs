pub mod database;

use std::collections::HashMap;

use crate::{
    error::{DatabaseError, InvalidRequest},
    talk::{Feedback, FeedbackSender, Message, MessageResult, RequestId},
};

type ResultHashMap = HashMap<MessageResult, usize>;
type MessageDatabase = HashMap<Message, usize>;
type DatabaseResult = Result<(), DatabaseError>;
pub struct Database {
    received: MessageDatabase,
    requests: RequestDatabase,
}

struct RequestDatabase {
    message_results: HashMap<RequestId, MessageResultDatabase>,
}

struct MessageResultDatabase {
    results: MessageDatabase,
    feedback_inlet: FeedbackSender,
}
