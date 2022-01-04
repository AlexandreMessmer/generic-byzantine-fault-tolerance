use std::collections::BTreeSet;

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
    pub fn is_conflicting(set1: &BTreeSet<Command>, set2: &BTreeSet<Command>) -> bool {
        let mut set2 = set2.clone().into_iter();
        for elem1 in set1.iter() {
            if set2.any(|elem2| Self::is_related(&elem1, &elem2) && elem1.ne(&elem2)) {
                return true;
            }
        }
        false
    }
}

impl Relation<Command> for ConflictingRelation {
    fn is_related(x: &Command, y: &Command) -> bool {
        if x.eq(y) {
            return false;
        }
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
