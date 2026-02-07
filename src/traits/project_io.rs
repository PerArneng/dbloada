use std::path::Path;
use thiserror::Error;
use super::string_file::StringFileError;
use super::project_serialization::{Project, ProjectSerializationError};

#[derive(Debug, Error)]
pub enum ProjectIOError {
    #[error(transparent)]
    FileError(#[from] StringFileError),
    #[error(transparent)]
    SerializationError(#[from] ProjectSerializationError),
}

pub trait ProjectIO {
    fn load(&self, path: &Path) -> Result<Project, ProjectIOError>;
    fn save(&self, project: &Project, path: &Path) -> Result<(), ProjectIOError>;
}
