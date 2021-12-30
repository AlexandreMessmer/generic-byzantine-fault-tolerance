pub mod client_database;
pub mod replica_database;

use std::collections::HashMap;

use crate::talk::CommandResult;
use crate::{
    error::{DatabaseError, InvalidRequest},
    talk::{CommandId, Feedback, FeedbackSender, Message},
};

type ResultHashMap = HashMap<CommandResult, usize>;
type MessageDatabase = HashMap<Message, usize>;
type DatabaseResult = Result<Option<usize>, DatabaseError>;
