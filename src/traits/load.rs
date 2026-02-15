use std::path::Path;
use async_trait::async_trait;
use thiserror::Error;
use crate::models::LoadedProject;
use super::project_io::ProjectIOError;
use super::TableReaderError;

#[derive(Debug, Error)]
pub enum LoadError {
    #[error("directory not found: {0}")]
    DirectoryNotFound(String),
    #[error("project file not found: {0}")]
    ProjectFileNotFound(String),
    #[error(transparent)]
    IOError(#[from] ProjectIOError),
    #[error(transparent)]
    TableReaderError(#[from] TableReaderError),
}

#[async_trait]
pub trait Load: Send + Sync {
    async fn load(&self, path: &Path) -> Result<LoadedProject, LoadError>;
}
