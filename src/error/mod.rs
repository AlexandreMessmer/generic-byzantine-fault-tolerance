#[derive(Debug, Clone)]
pub struct InvalidRequest;

#[derive(Debug)]
pub struct DatabaseError {
    error: String,
}
impl DatabaseError {
    pub fn new(arg: &str) -> Self {
        DatabaseError {
            error: String::from(arg),
        }
    }
}
