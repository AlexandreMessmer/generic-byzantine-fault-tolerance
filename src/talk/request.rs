use uuid::Uuid;

use super::RequestId;

pub struct Request {
    id: RequestId,
}

impl Request {
    pub fn new() -> Self {
        let id = Uuid::new_v4();
        Request { id }
    }

    pub fn id(&self) -> &RequestId {
        &self.id
    }
}
