use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use crate::traits::{Logger, StringFile, StringFileError};

pub struct TestLogger;

impl Logger for TestLogger {
    fn error(&self, _msg: &str) {}
    fn warn(&self, _msg: &str) {}
    fn info(&self, _msg: &str) {}
    fn debug(&self, _msg: &str) {}
    fn trace(&self, _msg: &str) {}
}

pub struct InMemoryStringFile {
    store: Rc<RefCell<HashMap<PathBuf, String>>>,
}

impl InMemoryStringFile {
    pub fn new(store: Rc<RefCell<HashMap<PathBuf, String>>>) -> Self {
        InMemoryStringFile { store }
    }
}

impl StringFile for InMemoryStringFile {
    fn save(&self, content: &str, path: &Path) -> Result<(), StringFileError> {
        self.store.borrow_mut().insert(path.to_path_buf(), content.to_string());
        Ok(())
    }

    fn load(&self, path: &Path) -> Result<String, StringFileError> {
        self.store
            .borrow()
            .get(path)
            .cloned()
            .ok_or_else(|| StringFileError::ReadError {
                path: path.to_path_buf(),
                source: std::io::Error::new(std::io::ErrorKind::NotFound, "not found in memory store"),
            })
    }

    fn ensure_dir(&self, _path: &Path) -> Result<(), StringFileError> {
        Ok(())
    }
}
