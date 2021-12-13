use std::collections::hash_map::Entry;

use uuid::Uuid;

use super::*;
pub struct RequestDatabase {
    message_results: HashMap<RequestId, MessageResultDatabase>,
}

impl RequestDatabase {
    pub fn new() -> Self {
        RequestDatabase {
            message_results: HashMap::<Uuid, MessageResultDatabase>::new(),
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
        message: Message,
    ) -> Result<Option<usize>, InvalidRequest> {
        let mut res: Option<usize> = None;
        if let Entry::Occupied(_) = self.message_results
            .entry(request)
            .and_modify(|db| {
                res = db.add(message);
        }) {
            return Ok(res);
        }
        Err(InvalidRequest)
    }

    pub fn remove(&mut self, request: &RequestId) -> Option<FeedbackSender> {
        self.message_results
            .remove(request)
            .map(|v | v.feedback_inlet())
    }

    pub fn contains(&self, request_id: &RequestId) -> bool {
        self.message_results
            .contains_key(request_id)
    }

    /// Returns the number of identical message received for a given request ID.
    /// None is return if either the request or the message is not present.
    pub fn request_info(&self, request_id: &RequestId, message: &Message) -> Option<usize> {
        self.message_results
            .get(request_id)
            .map(|db| db.message_number(message))
            .flatten()
    }
}

#[cfg(test)]
mod tests {

    use crate::talk::FeedbackChannel;

    use super::*;

    #[tokio::test]
    async fn global_test(){
        let (tx1, _) = FeedbackChannel::channel().await;
        let (tx2, _) = FeedbackChannel::channel().await;

        let mut request_db = RequestDatabase::new();
        let id = new_request_id();
        if let Err(_) = request_db.add_request(id, tx1){
            panic!();
        }
        if let Ok(_) = request_db.add_request(id, tx2) {
            panic!();
        }

        let res1 = request_db.add(id, Message::Testing).unwrap();
        assert_eq!(res1, None);
        let res2 = request_db.add(id, Message::Testing).unwrap();
        assert_eq!(res2.unwrap(), 1);

        if let Ok(_) = request_db.add(new_request_id(), Message::Testing) {
            panic!();
        }

    }

    fn new_request_id() -> RequestId {
        uuid::Uuid::new_v4()
    }
}