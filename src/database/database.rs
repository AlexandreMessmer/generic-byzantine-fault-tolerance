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
    ) -> Result<(), DatabaseError> {
        self.requests
            .add_request(request, feedback_sender)
            .map_err(|_| DatabaseError::new("Cannot add twice the same request"))
    }

    /// Update a request by adding the given `Message` in the database.
    /// The request must be in the database.
    pub fn update_request(
        &mut self,
        request_id: RequestId,
        message_result: Message,
    ) -> Result<Option<usize>, DatabaseError> {
        self.requests
            .add(request_id, message_result)
            .map_err(|_| DatabaseError::new("Request doesn't exist"))
    }

    pub fn complete_request(&mut self, request_id: &RequestId) -> Result<FeedbackSender, DatabaseError> {
        self.requests
            .remove(request_id)
            .ok_or(DatabaseError::new("The request doesn't exist"))
    }

    pub fn contains_request(&self, request_id: &RequestId) -> bool {
        self.requests
            .contains(request_id)
    }

    pub fn is_request_completed(&self, request_id: &RequestId, message: &Message, bound: usize) -> Result<bool, DatabaseError> {
        self.requests
            .request_info(request_id, message)
            .map(|size| size >= bound)
            .ok_or(DatabaseError::new("The request is either inexistant or unintialized"))
    }
}

