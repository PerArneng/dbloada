use std::path::Path;
use async_trait::async_trait;
use thiserror::Error;
use super::project_io::ProjectIOError;
use super::file_system::FileSystemError;

#[derive(Debug, Error)]
pub enum InitError {
    #[error("directory does not exist: {0}")]
    DirectoryNotFound(String),
    #[error("failed to derive project name from path: {0}")]
    InvalidDirectoryName(String),
    #[error("invalid resource name '{name}': {reason}")]
    InvalidResourceName { name: String, reason: String },
    #[error("failed to write dbloada.yaml: {0}")]
    IOError(#[from] ProjectIOError),
    #[error("file operation failed: {0}")]
    FileError(#[from] FileSystemError),
    #[error("directory is not empty: {0} (use --force to override)")]
    DirectoryNotEmpty(String),
}

#[async_trait]
pub trait Init: Send + Sync {
    async fn init(&self, path: &Path, name: Option<&str>, force: bool) -> Result<(), InitError>;
}
