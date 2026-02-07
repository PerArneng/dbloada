use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StringFileError {
    #[error("failed to read file: {path}")]
    ReadError {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("failed to write file: {path}")]
    WriteError {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("failed to create directory: {path}")]
    DirCreateError {
        path: PathBuf,
        source: std::io::Error,
    },
}

pub trait StringFile {
    fn save(&self, content: &str, path: &std::path::Path) -> Result<(), StringFileError>;
    fn load(&self, path: &std::path::Path) -> Result<String, StringFileError>;
    fn ensure_dir(&self, path: &std::path::Path) -> Result<(), StringFileError>;
}
