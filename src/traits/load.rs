use std::path::Path;
use async_trait::async_trait;
use thiserror::Error;
use super::project_serialization::Project;
use super::project_io::ProjectIOError;

#[derive(Debug, Error)]
pub enum LoadError {
    #[error("directory not found: {0}")]
    DirectoryNotFound(String),
    #[error("project file not found: {0}")]
    ProjectFileNotFound(String),
    #[error(transparent)]
    IOError(#[from] ProjectIOError),
}

#[async_trait]
pub trait Load: Send + Sync {
    async fn load(&self, path: &Path) -> Result<Project, LoadError>;
}
