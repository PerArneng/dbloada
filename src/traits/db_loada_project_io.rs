use std::path::Path;
use thiserror::Error;
use super::string_file::StringFileError;
use super::db_loada_project_serialization::{DBLoadaProject, DbLoadaProjectSerializationError};

#[derive(Debug, Error)]
pub enum DbLoadaProjectIOError {
    #[error(transparent)]
    FileError(#[from] StringFileError),
    #[error(transparent)]
    SerializationError(#[from] DbLoadaProjectSerializationError),
}

pub trait DbLoadaProjectIO {
    fn load(&self, path: &Path) -> Result<DBLoadaProject, DbLoadaProjectIOError>;
    fn save(&self, project: &DBLoadaProject, path: &Path) -> Result<(), DbLoadaProjectIOError>;
}
