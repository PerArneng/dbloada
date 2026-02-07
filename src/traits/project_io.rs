use std::path::Path;
use thiserror::Error;
use super::file_system::FileSystemError;
use super::project_serialization::{Project, ProjectSerializationError};

#[derive(Debug, Error)]
pub enum ProjectIOError {
    #[error(transparent)]
    FileError(#[from] FileSystemError),
    #[error(transparent)]
    SerializationError(#[from] ProjectSerializationError),
}

pub trait ProjectIO {
    fn load(&self, path: &Path) -> Result<Project, ProjectIOError>;
    fn save(&self, project: &Project, path: &Path) -> Result<(), ProjectIOError>;
}
