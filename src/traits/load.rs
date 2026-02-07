use std::path::Path;
use thiserror::Error;
use super::db_loada_project_serialization::DBLoadaProject;
use super::db_loada_project_io::DbLoadaProjectIOError;

#[derive(Debug, Error)]
pub enum LoadError {
    #[error("directory not found: {0}")]
    DirectoryNotFound(String),
    #[error("project file not found: {0}")]
    ProjectFileNotFound(String),
    #[error(transparent)]
    IOError(#[from] DbLoadaProjectIOError),
}

pub trait Load {
    fn load(&self, path: &Path) -> Result<DBLoadaProject, LoadError>;
}
