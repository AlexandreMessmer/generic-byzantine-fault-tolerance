use std::collections::HashMap;

use super::*;

pub struct ClientDatabase {
    requests: HashMap<RequestId, HashMap<Message, usize>>,
}
impl ClientDatabase {
    pub fn new() -> Self {
        ClientDatabase {
            requests: HashMap::new(),
        }
    }

    /// Add a request to the database.
    /// The `RequestId` must be unique. It fails if a request with the same id already exists.
    pub fn add_request(&mut self, request: RequestId) -> Result<(), DatabaseError> {
        if self.requests.contains_key(&request) {
            return Err(DatabaseError::new("Cannot insert twice the same request"));
        }
        self.requests.insert(request, HashMap::new());
        Ok(())
    }

    /// Update a request by adding the given `Message` in the database.
    /// The request must be in the database.
    pub fn update_request(
        &mut self,
        request_id: &RequestId,
        message: Message,
    ) -> Result<usize, DatabaseError> {
        let nbr = self
            .requests
            .get_mut(request_id)
            .map(|request_db| {
                request_db
                    .entry(message)
                    .and_modify(|content| *content += 1)
                    .or_insert(1)
            })
            .map(|nbr| nbr.clone());
        nbr.ok_or(DatabaseError::from(format!(
            "Cannot find the request #{}",
            *request_id
        )))
    }

    pub fn contains_request(&self, request: &RequestId) -> bool {
        self.requests.contains_key(request)
    }

    pub fn complete_request(&mut self, request_id: &RequestId) -> Result<(), DatabaseError> {
        self.requests
            .remove(request_id)
            .map(|_| ())
            .ok_or(DatabaseError::new("The request doesn't exist"))
    }

    pub fn is_request_completed(
        &self,
        request_id: &RequestId,
        message: &Message,
        bound: usize,
    ) -> Result<bool, DatabaseError> {
        self.requests
            .get(request_id)
            .map(|request_db| request_db.get(message))
            .flatten()
            .map(|nbr| nbr.eq(&bound))
            .ok_or(DatabaseError::new(&format!(
                "Cannot retrieve the request {}",
                *request_id
            )))
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use crate::talk::Request;

    use super::*;

    #[test]
    fn add_request_works() {
        let mut db = ClientDatabase::new();
        let request = Request::new();
        assert_eq!(db.contains_request(request.id()), false);
        db.add_request(request.id().clone()).unwrap();
        assert_eq!(db.contains_request(request.id()), true);
        let res = db.add_request(request.id().clone());
        assert_eq!(res.is_err(), true);
    }

    #[test]
    fn request_id_is_resistant() {
        let mut db = ClientDatabase::new();
        for _ in 0..10000 {
            db.add_request(Uuid::new_v4()).unwrap();
        }
        print!("ID: {}", Uuid::new_v4());
    }
}
