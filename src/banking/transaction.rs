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

    pub fn id(&self) -> &Uuid {
        &self.id
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
            "Client #{:02.2} > {:<.16} -> {:<.16} (ID: {:<.8}) : {}",
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

#[cfg(test)]
mod tests {
    use crate::{
        banking::action::Action,
        talk::{Command, CommandResult},
    };

    use super::Transaction;

    #[test]
    fn print_test() {
        let _ = Transaction::from_command(
            &Command::new(11, Action::Deposit(10)),
            &CommandResult::Failure(format!("13456789A3456789567890")),
        );
    }
}
