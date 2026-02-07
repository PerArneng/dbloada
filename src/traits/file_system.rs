use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FileSystemError {
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

pub trait FileSystem {
    fn save(&self, content: &str, path: &std::path::Path) -> Result<(), FileSystemError>;
    fn load(&self, path: &std::path::Path) -> Result<String, FileSystemError>;
    fn ensure_dir(&self, path: &std::path::Path) -> Result<(), FileSystemError>;
}
