use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;
use async_trait::async_trait;
use crate::traits::{Logger, FileSystem, FileSystemError};

pub struct TestLogger;

#[async_trait]
impl Logger for TestLogger {
    async fn error(&self, _msg: &str) {}
    async fn warn(&self, _msg: &str) {}
    async fn info(&self, _msg: &str) {}
    async fn debug(&self, _msg: &str) {}
    async fn trace(&self, _msg: &str) {}
}

pub struct InMemoryFileSystem {
    store: Arc<Mutex<HashMap<PathBuf, String>>>,
}

impl InMemoryFileSystem {
    pub fn new(store: Arc<Mutex<HashMap<PathBuf, String>>>) -> Self {
        InMemoryFileSystem { store }
    }
}

#[async_trait]
impl FileSystem for InMemoryFileSystem {
    async fn save(&self, content: &str, path: &Path) -> Result<(), FileSystemError> {
        self.store.lock().await.insert(path.to_path_buf(), content.to_string());
        Ok(())
    }

    async fn load(&self, path: &Path) -> Result<String, FileSystemError> {
        self.store
            .lock()
            .await
            .get(path)
            .cloned()
            .ok_or_else(|| FileSystemError::ReadError {
                path: path.to_path_buf(),
                source: std::io::Error::new(std::io::ErrorKind::NotFound, "not found in memory store"),
            })
    }

    async fn ensure_dir(&self, _path: &Path) -> Result<(), FileSystemError> {
        Ok(())
    }
}
