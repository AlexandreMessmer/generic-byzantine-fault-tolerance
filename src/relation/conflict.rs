use crate::talk::Command;

use super::Relation;

pub struct ConflictingRelation;

impl Relation<Command> for ConflictingRelation {
    fn is_related(x: &Command, y: &Command) -> bool {
        todo!()
    }
}
