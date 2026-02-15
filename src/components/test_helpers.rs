use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;
use async_trait::async_trait;
use crate::models::Project;
use crate::traits::{Logger, FileSystem, FileSystemError, ProjectIO, ProjectIOError};

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

    async fn load_bytes(&self, path: &Path) -> Result<Vec<u8>, FileSystemError> {
        let content = self.load(path).await?;
        Ok(content.into_bytes())
    }

    async fn ensure_dir(&self, _path: &Path) -> Result<(), FileSystemError> {
        Ok(())
    }
}

pub struct InMemoryProjectIO;

#[async_trait]
impl ProjectIO for InMemoryProjectIO {
    async fn load(&self, _path: &Path) -> Result<Project, ProjectIOError> {
        unimplemented!("not needed in test")
    }

    async fn save(&self, _project: &Project, _path: &Path) -> Result<(), ProjectIOError> {
        Ok(())
    }
}

pub fn mock_logger() -> Box<dyn Logger> {
    Box::new(TestLogger)
}

pub fn mock_project_io() -> Box<dyn ProjectIO> {
    Box::new(InMemoryProjectIO)
}

pub fn mock_file_system() -> Box<dyn FileSystem> {
    Box::new(InMemoryFileSystem::new(Arc::new(Mutex::new(HashMap::new()))))
}
