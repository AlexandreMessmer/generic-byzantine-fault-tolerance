use serde::{Deserialize, Serialize};
use talk::crypto::Identity;
use uuid::Uuid;

use crate::relation::{conflict::ConflictingRelation, Relation};

use super::CommandId;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Command {
    id: CommandId,
}

impl Command {
    pub fn new() -> Self {
        let id = Uuid::new_v4();
        Command { id }
    }

    pub fn id(&self) -> &CommandId {
        &self.id
    }

    pub fn issuer(&self) -> &Identity {
        todo!()
    }

    pub fn generate_id() -> Uuid {
        Uuid::new_v4()
    }

    pub fn execute(&self) -> CommandResult {
        CommandResult {}
    }

    pub fn conflict(&self, other: &Command) -> bool {
        ConflictingRelation::is_related(&self, other)
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CommandResult {}
