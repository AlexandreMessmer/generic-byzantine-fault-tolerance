

/// Defines the API of a Runner
/// Note that it should also implements an async method run()
#[async_trait::async_trait]
pub trait Runner {
    async fn run(mut self);
}
