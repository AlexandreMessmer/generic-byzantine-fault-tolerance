use futures::Future;

pub struct Timeout {}

impl Timeout {
    pub fn until_success<T, U>(f: T)
    where
        T: Future<Output = U>,
    {
    }
}
