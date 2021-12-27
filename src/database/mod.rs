pub mod client_database;

use std::collections::HashMap;

use crate::{
    error::{DatabaseError, InvalidRequest},
    talk::{Feedback, FeedbackSender, Message, MessageResult, RequestId},
};

type ResultHashMap = HashMap<MessageResult, usize>;
type MessageDatabase = HashMap<Message, usize>;
type DatabaseResult = Result<Option<usize>, DatabaseError>;
