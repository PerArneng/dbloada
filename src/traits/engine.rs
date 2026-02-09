use std::path::Path;
use async_trait::async_trait;
use super::init::InitError;
use super::load::LoadError;
use crate::models::Project;

#[async_trait]
pub trait Engine: Send + Sync {
    async fn init(&self);
    async fn init_project_dir(&self, path: &Path, name: Option<&str>) -> Result<(), InitError>;
    async fn load_project(&self, path: &Path) -> Result<Project, LoadError>;
}
