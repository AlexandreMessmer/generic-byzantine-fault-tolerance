use std::fmt::Display;

use uuid::Uuid;

use crate::{
    peer::peer::PeerId,
    talk::{Command, CommandResult},
};

use super::action::Action;

pub struct Transaction {
    id: Uuid,
    issuer: PeerId,
    action: Action,
    result: CommandResult,
    status: Status,
}

impl Transaction {
    pub fn new(
        id: Uuid,
        issuer: PeerId,
        action: Action,
        result: CommandResult,
        status: Status,
    ) -> Self {
        Transaction {
            id,
            issuer,
            action,
            result,
            status,
        }
    }

    pub fn from_command(command: &Command, result: &CommandResult) -> Self {
        Transaction::new(
            *command.id(),
            *command.issuer(),
            command.action().clone(),
            result.clone(),
            Status::Executed,
        )
    }

    pub fn rollback(&mut self) {
        self.status = Status::Rollbacked;
    }
}

impl Display for Transaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Client #{} > {} -> {} (ID: {}) : {}",
            self.issuer, self.action, self.result, self.id, self.status
        )
    }
}

pub enum Status {
    Executed,
    Rollbacked,
}

impl Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Status::Executed => "EXECUTED",
            Status::Rollbacked => "ROLLBACKED",
        };
        write!(f, "{}", str)
    }
}
