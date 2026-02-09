use std::path::Path;
use async_trait::async_trait;
use crate::traits::{Logger, FileSystem, FileSystemError};

pub struct DiskFileSystem {
    logger: Box<dyn Logger>,
}

impl DiskFileSystem {
    pub fn new(logger: Box<dyn Logger>) -> Self {
        DiskFileSystem { logger }
    }
}

#[async_trait]
impl FileSystem for DiskFileSystem {
    async fn save(&self, content: &str, path: &Path) -> Result<(), FileSystemError> {
        self.logger.debug(&format!("writing file: {}", path.display())).await;
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await.map_err(|e| FileSystemError::DirCreateError {
                path: parent.to_path_buf(),
                source: e,
            })?;
        }
        tokio::fs::write(path, content).await.map_err(|e| FileSystemError::WriteError {
            path: path.to_path_buf(),
            source: e,
        })?;
        self.logger.info(&format!("wrote file: {}", path.display())).await;
        Ok(())
    }

    async fn load(&self, path: &Path) -> Result<String, FileSystemError> {
        self.logger.debug(&format!("reading file: {}", path.display())).await;
        let content = tokio::fs::read_to_string(path).await.map_err(|e| FileSystemError::ReadError {
            path: path.to_path_buf(),
            source: e,
        })?;
        self.logger.info(&format!("read file: {}", path.display())).await;
        Ok(content)
    }

    async fn ensure_dir(&self, path: &Path) -> Result<(), FileSystemError> {
        self.logger.debug(&format!("ensuring directory: {}", path.display())).await;
        tokio::fs::create_dir_all(path).await.map_err(|e| FileSystemError::DirCreateError {
            path: path.to_path_buf(),
            source: e,
        })?;
        self.logger.info(&format!("ensured directory: {}", path.display())).await;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::test_helpers::TestLogger;
    use std::path::PathBuf;

    #[tokio::test]
    async fn save_and_load_round_trip() {
        let logger = Box::new(TestLogger);
        let file_system = DiskFileSystem::new(logger);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.txt");
        let content = "hello world\nline two";

        file_system.save(content, &path).await.unwrap();
        let loaded = file_system.load(&path).await.unwrap();

        assert_eq!(loaded, content);
    }

    #[tokio::test]
    async fn load_nonexistent_file_returns_read_error() {
        let logger = Box::new(TestLogger);
        let file_system = DiskFileSystem::new(logger);
        let path = PathBuf::from("/nonexistent/path/file.txt");

        let result = file_system.load(&path).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, FileSystemError::ReadError { .. }));
    }

    #[tokio::test]
    async fn save_to_invalid_path_returns_dir_create_error() {
        let logger = Box::new(TestLogger);
        let file_system = DiskFileSystem::new(logger);
        let path = PathBuf::from("/nonexistent/directory/file.txt");

        let result = file_system.save("content", &path).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, FileSystemError::DirCreateError { .. }));
    }

    #[tokio::test]
    async fn save_creates_parent_directories() {
        let logger = Box::new(TestLogger);
        let file_system = DiskFileSystem::new(logger);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("sub").join("dir").join("test.txt");

        file_system.save("nested content", &path).await.unwrap();
        let loaded = file_system.load(&path).await.unwrap();

        assert_eq!(loaded, "nested content");
    }

    #[tokio::test]
    async fn ensure_dir_creates_directory() {
        let logger = Box::new(TestLogger);
        let file_system = DiskFileSystem::new(logger);
        let dir = tempfile::tempdir().unwrap();
        let new_dir = dir.path().join("new_subdir");

        file_system.ensure_dir(&new_dir).await.unwrap();

        assert!(new_dir.is_dir());
    }

    #[tokio::test]
    async fn ensure_dir_invalid_path_returns_error() {
        let logger = Box::new(TestLogger);
        let file_system = DiskFileSystem::new(logger);
        let path = PathBuf::from("/nonexistent/root/dir");

        let result = file_system.ensure_dir(&path).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, FileSystemError::DirCreateError { .. }));
    }
}
