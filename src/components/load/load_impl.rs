use std::path::{Path, PathBuf};
use crate::traits::{Project, ProjectIO, Load, LoadError, Logger};

pub const DBLOADA_PROJECT_FILENAME: &str = "dbloada.yaml";

pub fn project_file_path(dir: &Path) -> PathBuf {
    dir.join(DBLOADA_PROJECT_FILENAME)
}

pub struct LoadImpl {
    logger: Box<dyn Logger>,
    project_io: Box<dyn ProjectIO>,
}

impl LoadImpl {
    pub fn new(logger: Box<dyn Logger>, project_io: Box<dyn ProjectIO>) -> Self {
        LoadImpl { logger, project_io }
    }
}

impl Load for LoadImpl {
    fn load(&self, path: &Path) -> Result<Project, LoadError> {
        if !path.is_dir() {
            return Err(LoadError::DirectoryNotFound(path.display().to_string()));
        }

        let file_path = project_file_path(path);
        if !file_path.exists() {
            return Err(LoadError::ProjectFileNotFound(file_path.display().to_string()));
        }

        self.logger.debug(&format!("loading project from: {}", file_path.display()));
        let project = self.project_io.load(&file_path)?;
        self.logger.info(&format!("loaded project '{}' from: {}", project.name, file_path.display()));
        Ok(project)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_file_path_appends_filename() {
        let path = project_file_path(Path::new("/some/dir"));
        assert_eq!(path, PathBuf::from("/some/dir/dbloada.yaml"));
    }

    #[test]
    fn project_file_path_with_trailing_slash() {
        let path = project_file_path(Path::new("/some/dir/"));
        assert_eq!(path, PathBuf::from("/some/dir/dbloada.yaml"));
    }

    #[test]
    fn load_returns_error_for_nonexistent_directory() {
        use crate::components::test_helpers::TestLogger;
        use crate::components::project_io::YamlProjectIO;
        use crate::components::project_serialization::YamlProjectSerialization;
        use crate::components::test_helpers::InMemoryFileSystem;
        use std::rc::Rc;
        use std::cell::RefCell;
        use std::collections::HashMap;

        let store = Rc::new(RefCell::new(HashMap::new()));
        let file_system = Box::new(InMemoryFileSystem::new(store));
        let serialization = Box::new(YamlProjectSerialization::new(Box::new(TestLogger)));
        let project_io = Box::new(YamlProjectIO::new(
            Box::new(TestLogger),
            file_system,
            serialization,
        ));
        let loader = LoadImpl::new(Box::new(TestLogger), project_io);

        let result = loader.load(Path::new("/nonexistent/dir"));
        assert!(matches!(result, Err(LoadError::DirectoryNotFound(_))));
    }
}
