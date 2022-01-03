#[async_trait::async_trait]
pub trait Shutdownable {
    async fn shutdown(&mut self);
}
