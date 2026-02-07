use std::path::Path;
use thiserror::Error;
use super::project_io::ProjectIOError;
use super::string_file::StringFileError;

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
    FileError(#[from] StringFileError),
}

pub trait Init {
    fn init(&self, path: &Path, name: Option<&str>) -> Result<(), InitError>;
}
