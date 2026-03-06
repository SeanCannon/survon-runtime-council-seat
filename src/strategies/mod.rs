use anyhow::Result;
use async_trait::async_trait;

pub mod librarian;

#[async_trait]
pub trait Strategy: Send + Sync {
    async fn initialize(&mut self) -> Result<()>;
    async fn query(&self, question: &str) -> Result<String>;
    fn name(&self) -> &str;
    fn description(&self) -> &str;
}
