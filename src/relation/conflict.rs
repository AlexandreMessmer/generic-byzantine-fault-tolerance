use crate::{banking::action::Action, talk::Command};

use super::Relation;

/// Define the conflicting relation between banking operations.
/// To simplify the model, we define it as follows (command with different issuer doesn't not conflict):
/// Any action require a registration. Thus, Register conflicts with everything.
/// Deposit to an account commute. Thus, two deposits doesn't not conflict.
/// Withdrawal and any other operation conflict. Since it can fail, and requires a decrement operation, it must be executed
/// in order.
pub struct ConflictingRelation;

impl ConflictingRelation {
    /// x conflict with y if x ~ y (but not necessarily y ~ x)
    fn conflict_with(x: &Action, y: &Action) -> bool {
        match x {
            Action::Register => match y {
                Action::Register => false,
                _ => true,
            },
            Action::Get => false,
            Action::Deposit(_) => false,
            Action::Withdraw(_) => true,
        }
    }
}
impl Relation<Command> for ConflictingRelation {
    fn is_related(x: &Command, y: &Command) -> bool {
        if x.issuer().eq(y.issuer()) {
            let x_action = x.action();
            let y_action = y.action();
            return ConflictingRelation::conflict_with(x_action, y_action)
                || ConflictingRelation::conflict_with(y_action, x_action);
        }
        false
    }
}
