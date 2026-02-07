use std::fs;
use std::path::Path;
use crate::traits::{Logger, FileSystem, FileSystemError};

pub struct DiskFileSystem {
    logger: Box<dyn Logger>,
}

impl DiskFileSystem {
    pub fn new(logger: Box<dyn Logger>) -> Self {
        DiskFileSystem { logger }
    }
}

impl FileSystem for DiskFileSystem {
    fn save(&self, content: &str, path: &Path) -> Result<(), FileSystemError> {
        self.logger.debug(&format!("writing file: {}", path.display()));
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| FileSystemError::DirCreateError {
                path: parent.to_path_buf(),
                source: e,
            })?;
        }
        fs::write(path, content).map_err(|e| FileSystemError::WriteError {
            path: path.to_path_buf(),
            source: e,
        })?;
        self.logger.info(&format!("wrote file: {}", path.display()));
        Ok(())
    }

    fn load(&self, path: &Path) -> Result<String, FileSystemError> {
        self.logger.debug(&format!("reading file: {}", path.display()));
        let content = fs::read_to_string(path).map_err(|e| FileSystemError::ReadError {
            path: path.to_path_buf(),
            source: e,
        })?;
        self.logger.info(&format!("read file: {}", path.display()));
        Ok(content)
    }

    fn ensure_dir(&self, path: &Path) -> Result<(), FileSystemError> {
        self.logger.debug(&format!("ensuring directory: {}", path.display()));
        fs::create_dir_all(path).map_err(|e| FileSystemError::DirCreateError {
            path: path.to_path_buf(),
            source: e,
        })?;
        self.logger.info(&format!("ensured directory: {}", path.display()));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::test_helpers::TestLogger;
    use std::path::PathBuf;

    #[test]
    fn save_and_load_round_trip() {
        let logger = Box::new(TestLogger);
        let file_system = DiskFileSystem::new(logger);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.txt");
        let content = "hello world\nline two";

        file_system.save(content, &path).unwrap();
        let loaded = file_system.load(&path).unwrap();

        assert_eq!(loaded, content);
    }

    #[test]
    fn load_nonexistent_file_returns_read_error() {
        let logger = Box::new(TestLogger);
        let file_system = DiskFileSystem::new(logger);
        let path = PathBuf::from("/nonexistent/path/file.txt");

        let result = file_system.load(&path);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, FileSystemError::ReadError { .. }));
    }

    #[test]
    fn save_to_invalid_path_returns_dir_create_error() {
        let logger = Box::new(TestLogger);
        let file_system = DiskFileSystem::new(logger);
        let path = PathBuf::from("/nonexistent/directory/file.txt");

        let result = file_system.save("content", &path);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, FileSystemError::DirCreateError { .. }));
    }

    #[test]
    fn save_creates_parent_directories() {
        let logger = Box::new(TestLogger);
        let file_system = DiskFileSystem::new(logger);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("sub").join("dir").join("test.txt");

        file_system.save("nested content", &path).unwrap();
        let loaded = file_system.load(&path).unwrap();

        assert_eq!(loaded, "nested content");
    }

    #[test]
    fn ensure_dir_creates_directory() {
        let logger = Box::new(TestLogger);
        let file_system = DiskFileSystem::new(logger);
        let dir = tempfile::tempdir().unwrap();
        let new_dir = dir.path().join("new_subdir");

        file_system.ensure_dir(&new_dir).unwrap();

        assert!(new_dir.is_dir());
    }

    #[test]
    fn ensure_dir_invalid_path_returns_error() {
        let logger = Box::new(TestLogger);
        let file_system = DiskFileSystem::new(logger);
        let path = PathBuf::from("/nonexistent/root/dir");

        let result = file_system.ensure_dir(&path);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, FileSystemError::DirCreateError { .. }));
    }
}
