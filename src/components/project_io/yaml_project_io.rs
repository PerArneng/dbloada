use std::path::Path;
use async_trait::async_trait;
use crate::traits::{
    Project, ProjectIO, ProjectIOError,
    ProjectSerialization, Logger, FileSystem,
};

pub struct YamlProjectIO {
    logger: Box<dyn Logger>,
    file_system: Box<dyn FileSystem>,
    serialization: Box<dyn ProjectSerialization>,
}

impl YamlProjectIO {
    pub fn new(
        logger: Box<dyn Logger>,
        file_system: Box<dyn FileSystem>,
        serialization: Box<dyn ProjectSerialization>,
    ) -> Self {
        YamlProjectIO {
            logger,
            file_system,
            serialization,
        }
    }
}

#[async_trait]
impl ProjectIO for YamlProjectIO {
    async fn load(&self, path: &Path) -> Result<Project, ProjectIOError> {
        self.logger.debug(&format!("loading project from: {}", path.display())).await;
        let content = self.file_system.load(path).await?;
        let project = self.serialization.deserialize(&content).await?;
        self.logger.info(&format!("loaded project '{}' from: {}", project.name, path.display())).await;
        Ok(project)
    }

    async fn save(&self, project: &Project, path: &Path) -> Result<(), ProjectIOError> {
        self.logger.debug(&format!("saving project '{}' to: {}", project.name, path.display())).await;
        let content = self.serialization.serialize(project).await?;
        self.file_system.save(&content, path).await?;
        self.logger.info(&format!("saved project '{}' to: {}", project.name, path.display())).await;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::test_helpers::{InMemoryFileSystem, TestLogger};
    use crate::components::project_serialization::YamlProjectSerialization;
    use crate::traits::{PROJECT_API_VERSION, ProjectSpec};
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    fn make_io() -> (YamlProjectIO, Arc<Mutex<HashMap<PathBuf, String>>>) {
        let store = Arc::new(Mutex::new(HashMap::new()));
        let file_system = Box::new(InMemoryFileSystem::new(store.clone()));
        let serialization = Box::new(YamlProjectSerialization::new(Box::new(TestLogger)));
        let io = YamlProjectIO::new(Box::new(TestLogger), file_system, serialization);
        (io, store)
    }

    fn test_project(name: &str) -> Project {
        Project {
            name: name.to_string(),
            api_version: PROJECT_API_VERSION.to_string(),
            spec: ProjectSpec { tables: vec![] },
        }
    }

    #[tokio::test]
    async fn save_and_load_round_trip() {
        let (io, _store) = make_io();
        let path = PathBuf::from("/projects/dbloada.yaml");
        let project = test_project("my-project");

        io.save(&project, &path).await.unwrap();
        let loaded = io.load(&path).await.unwrap();

        assert_eq!(project, loaded);
    }

    #[tokio::test]
    async fn save_and_load_with_different_names() {
        let (io, _store) = make_io();

        for name in &["alpha", "beta-project", "test-123"] {
            let path = PathBuf::from(format!("/projects/{name}/dbloada.yaml"));
            let project = test_project(name);

            io.save(&project, &path).await.unwrap();
            let loaded = io.load(&path).await.unwrap();

            assert_eq!(project, loaded, "round-trip failed for name: {name}");
        }
    }

    #[tokio::test]
    async fn load_from_nonexistent_path_returns_error() {
        let (io, _store) = make_io();
        let path = PathBuf::from("/nonexistent/dbloada.yaml");

        let result = io.load(&path).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn load_from_invalid_yaml_content_returns_error() {
        let (io, store) = make_io();
        let path = PathBuf::from("/projects/dbloada.yaml");
        store.lock().await.insert(path.clone(), "not valid yaml {{{{".to_string());

        let result = io.load(&path).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn save_overwrites_existing_file() {
        let (io, _store) = make_io();
        let path = PathBuf::from("/projects/dbloada.yaml");

        let project1 = test_project("first");
        io.save(&project1, &path).await.unwrap();

        let project2 = test_project("second");
        io.save(&project2, &path).await.unwrap();

        let loaded = io.load(&path).await.unwrap();
        assert_eq!(loaded, project2);
    }

    #[tokio::test]
    async fn load_preserves_api_version() {
        let (io, _store) = make_io();
        let path = PathBuf::from("/projects/dbloada.yaml");
        let project = test_project("test");

        io.save(&project, &path).await.unwrap();
        let loaded = io.load(&path).await.unwrap();

        assert_eq!(loaded.api_version, PROJECT_API_VERSION);
    }

    #[tokio::test]
    async fn multiple_projects_at_different_paths() {
        let (io, _store) = make_io();
        let path_a = PathBuf::from("/projects/a/dbloada.yaml");
        let path_b = PathBuf::from("/projects/b/dbloada.yaml");
        let project_a = test_project("project-a");
        let project_b = test_project("project-b");

        io.save(&project_a, &path_a).await.unwrap();
        io.save(&project_b, &path_b).await.unwrap();

        let loaded_a = io.load(&path_a).await.unwrap();
        let loaded_b = io.load(&path_b).await.unwrap();

        assert_eq!(loaded_a, project_a);
        assert_eq!(loaded_b, project_b);
    }
}
