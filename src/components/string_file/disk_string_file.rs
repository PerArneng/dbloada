use std::fs;
use std::path::Path;
use crate::traits::{Logger, StringFile, StringFileError};

pub struct DiskStringFile {
    logger: Box<dyn Logger>,
}

impl DiskStringFile {
    pub fn new(logger: Box<dyn Logger>) -> Self {
        DiskStringFile { logger }
    }
}

impl StringFile for DiskStringFile {
    fn save(&self, content: &str, path: &Path) -> Result<(), StringFileError> {
        self.logger.debug(&format!("writing file: {}", path.display()));
        fs::write(path, content).map_err(|e| StringFileError::WriteError {
            path: path.to_path_buf(),
            source: e,
        })?;
        self.logger.info(&format!("wrote file: {}", path.display()));
        Ok(())
    }

    fn load(&self, path: &Path) -> Result<String, StringFileError> {
        self.logger.debug(&format!("reading file: {}", path.display()));
        let content = fs::read_to_string(path).map_err(|e| StringFileError::ReadError {
            path: path.to_path_buf(),
            source: e,
        })?;
        self.logger.info(&format!("read file: {}", path.display()));
        Ok(content)
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
        let string_file = DiskStringFile::new(logger);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.txt");
        let content = "hello world\nline two";

        string_file.save(content, &path).unwrap();
        let loaded = string_file.load(&path).unwrap();

        assert_eq!(loaded, content);
    }

    #[test]
    fn load_nonexistent_file_returns_read_error() {
        let logger = Box::new(TestLogger);
        let string_file = DiskStringFile::new(logger);
        let path = PathBuf::from("/nonexistent/path/file.txt");

        let result = string_file.load(&path);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, StringFileError::ReadError { .. }));
    }

    #[test]
    fn save_to_invalid_path_returns_write_error() {
        let logger = Box::new(TestLogger);
        let string_file = DiskStringFile::new(logger);
        let path = PathBuf::from("/nonexistent/directory/file.txt");

        let result = string_file.save("content", &path);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, StringFileError::WriteError { .. }));
    }
}
