use async_trait::async_trait;

#[async_trait]
pub trait Logger: Send + Sync {
    async fn error(&self, msg: &str);
    async fn warn(&self, msg: &str);
    async fn info(&self, msg: &str);
    async fn debug(&self, msg: &str);
    async fn trace(&self, msg: &str);
}
