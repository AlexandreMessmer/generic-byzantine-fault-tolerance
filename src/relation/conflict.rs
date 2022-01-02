use crate::{banking::action::Action, talk::Command};

use super::Relation;

/// Define the conflicting relation between banking operations.
/// To simplify the model, we define it as follows (command with different issuer doesn't not conflict):
/// Any action require a registration. Thus, Register conflicts with everything.
/// Deposit to an account commute. Thus, two deposits doesn't not conflict.
/// Withdrawal and any other operation conflict. Since it can fail, and requires a decrement operation, it must be executed
/// in order.
pub struct ConflictingRelation;

impl Relation<Command> for ConflictingRelation {
    fn is_related(x: &Command, y: &Command) -> bool {
        if x.issuer().eq(y.issuer()) {
            return match (x.action(), y.action()) {
                (Action::Register, Action::Register) => false,
                (Action::Register, _) => true,
                (_, Action::Register) => true,
                (Action::Withdraw(_), _) => true,
                (_, Action::Withdraw(_)) => true,
                _ => false,
            };
        }
        false
    }
}
