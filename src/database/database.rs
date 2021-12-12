use std::collections::{hash_map::Entry, HashMap};

use super::*;
impl Database {
    pub fn new() -> Self {
        Database {
            received: HashMap::new(),
            requests: RequestDatabase::new(),
        }
    }

    /// Add a request to the database.
    /// The `RequestId` must be unique. It fails if a request with the same id already exists.
    pub fn add_request(
        &mut self,
        request: RequestId,
        feedback_sender: FeedbackSender,
    ) -> DatabaseResult {
        self.requests
            .add_request(request, feedback_sender)
            .map_err(|_| DatabaseError::new("Cannot add twice the same request"))
    }

    /// Update a request by adding the given `Message` in the databases.
    /// The request must be in the database.
    pub fn update_request(
        &mut self,
        request_id: RequestId,
        message_result: Message,
    ) -> DatabaseResult {
        self.requests
            .add(request_id, message_result)
            .map_err(|_| DatabaseError::new("Request doesn't exist"))
    }
}

impl RequestDatabase {
    pub fn new() -> Self {
        RequestDatabase {
            message_results: HashMap::new(),
        }
    }

    pub fn add_request(
        &mut self,
        request: RequestId,
        feedback_sender: FeedbackSender,
    ) -> Result<(), InvalidRequest> {
        let res = self
            .message_results
            .insert(request, MessageResultDatabase::new(feedback_sender));
        if let None = res {
            return Ok(());
        }
        Err(InvalidRequest)
    }

    pub fn add(
        &mut self,
        request: RequestId,
        message_result: Message,
    ) -> Result<(), InvalidRequest> {
        if let Entry::Occupied(_) = self.message_results.entry(request).and_modify(|db| {
            db.add(message_result);
        }) {
            return Ok(());
        }
        Err(InvalidRequest)
    }
}

impl MessageResultDatabase {
    pub fn new(feedback_inlet: FeedbackSender) -> Self {
        MessageResultDatabase {
            results: HashMap::new(),
            feedback_inlet,
        }
    }

    pub fn add(&mut self, key: Message) -> Option<usize> {
        let value = self.results.get(&key).map_or(1, |x| x + 1);
        self.results.insert(key, value)
    }

    pub fn send_feedback(&self, feedback: Feedback) {
        self.feedback_inlet.send_feedback(feedback);
    }
}
