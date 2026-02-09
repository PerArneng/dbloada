use std::path::{Path, PathBuf};
use async_trait::async_trait;
use crate::models::Project;
use crate::traits::{ProjectIO, Load, LoadError, Logger};

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

#[async_trait]
impl Load for LoadImpl {
    async fn load(&self, path: &Path) -> Result<Project, LoadError> {
        let metadata = tokio::fs::metadata(path).await;
        if metadata.is_err() || !metadata.unwrap().is_dir() {
            return Err(LoadError::DirectoryNotFound(path.display().to_string()));
        }

        let file_path = project_file_path(path);
        let file_metadata = tokio::fs::metadata(&file_path).await;
        if file_metadata.is_err() {
            return Err(LoadError::ProjectFileNotFound(file_path.display().to_string()));
        }

        self.logger.debug(&format!("loading project from: {}", file_path.display())).await;
        let project = self.project_io.load(&file_path).await?;
        self.logger.info(&format!("loaded project '{}' from: {}", project.name, file_path.display())).await;
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

    #[tokio::test]
    async fn load_returns_error_for_nonexistent_directory() {
        use crate::components::test_helpers::TestLogger;
        use crate::components::project_io::YamlProjectIO;
        use crate::components::project_serialization::YamlProjectSerialization;
        use crate::components::test_helpers::InMemoryFileSystem;
        use std::sync::Arc;
        use tokio::sync::Mutex;
        use std::collections::HashMap;

        let store = Arc::new(Mutex::new(HashMap::new()));
        let file_system = Box::new(InMemoryFileSystem::new(store));
        let serialization = Box::new(YamlProjectSerialization::new(Box::new(TestLogger)));
        let project_io = Box::new(YamlProjectIO::new(
            Box::new(TestLogger),
            file_system,
            serialization,
        ));
        let loader = LoadImpl::new(Box::new(TestLogger), project_io);

        let result = loader.load(Path::new("/nonexistent/dir")).await;
        assert!(matches!(result, Err(LoadError::DirectoryNotFound(_))));
    }
}
