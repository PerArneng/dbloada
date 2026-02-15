use std::path::Path;
use async_trait::async_trait;
use super::init::InitError;
use super::load::LoadError;
use crate::models::LoadedProject;

#[async_trait]
pub trait Engine: Send + Sync {
    async fn init(&self);
    async fn init_project_dir(&self, path: &Path, name: Option<&str>, force: bool) -> Result<(), InitError>;
    async fn load_project(&self, path: &Path) -> Result<LoadedProject, LoadError>;
}
