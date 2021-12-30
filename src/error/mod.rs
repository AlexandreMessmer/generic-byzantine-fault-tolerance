use crate::talk::FeedbackSender;

#[derive(Debug, Clone)]
pub struct InvalidRequest;

pub struct InvalidRequestWithFeedback {
    pub feedback_sender: FeedbackSender,
}
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

    pub fn error_message(&self) -> String {
        self.error.clone()
    }
}

impl From<String> for DatabaseError {
    fn from(str: String) -> Self {
        DatabaseError { error: str }
    }
}

pub struct ShutdownError;
