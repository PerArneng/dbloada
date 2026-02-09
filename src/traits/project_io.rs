use std::path::Path;
use async_trait::async_trait;
use thiserror::Error;
use super::file_system::FileSystemError;
use crate::models::Project;
use super::project_serialization::ProjectSerializationError;

#[derive(Debug, Error)]
pub enum ProjectIOError {
    #[error(transparent)]
    FileError(#[from] FileSystemError),
    #[error(transparent)]
    SerializationError(#[from] ProjectSerializationError),
}

#[async_trait]
pub trait ProjectIO: Send + Sync {
    async fn load(&self, path: &Path) -> Result<Project, ProjectIOError>;
    async fn save(&self, project: &Project, path: &Path) -> Result<(), ProjectIOError>;
}
