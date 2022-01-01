

use serde::{Deserialize, Serialize};

use uuid::Uuid;

use crate::{
    banking::action::Action,
    peer::peer::PeerId,
    relation::{conflict::ConflictingRelation, Relation},
};

use super::CommandId;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Command {
    id: CommandId,
    issuer: PeerId,
    action: Action,
}

impl Command {
    pub fn new(issuer: PeerId, action: Action) -> Self {
        let id = Uuid::new_v4();
        Command { id, issuer, action }
    }

    pub fn issue() -> Self {
        todo!()
    }

    pub fn id(&self) -> &CommandId {
        &self.id
    }

    pub fn issuer(&self) -> &PeerId {
        todo!()
    }

    pub fn action(&self) -> &Action {
        &self.action
    }
    pub fn generate_id() -> Uuid {
        Uuid::new_v4()
    }

    pub fn conflict(&self, other: &Command) -> bool {
        ConflictingRelation::is_related(&self, other)
    }
}
