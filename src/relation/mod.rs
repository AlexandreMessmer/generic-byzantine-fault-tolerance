pub mod conflict;

pub trait Relation<T> {
    fn is_related(x: &T, y: &T) -> bool;
}
