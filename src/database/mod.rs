pub mod client_database;
pub mod replica_database;

use std::collections::HashMap;

use crate::talk::CommandResult;
use crate::{
    error::DatabaseError,
    talk::{CommandId, Message},
};

type ResultHashMap = HashMap<CommandResult, usize>;
type MessageDatabase = HashMap<Message, usize>;
type DatabaseResult = Result<Option<usize>, DatabaseError>;
