use std::path::Path;
use crate::traits::{
    DBLoadaProject, DbLoadaProjectIO, DbLoadaProjectIOError,
    DbLoadaProjectSerialization, Logger, StringFile,
};

pub struct YamlDbLoadaProjectIO {
    logger: Box<dyn Logger>,
    string_file: Box<dyn StringFile>,
    serialization: Box<dyn DbLoadaProjectSerialization>,
}

impl YamlDbLoadaProjectIO {
    pub fn new(
        logger: Box<dyn Logger>,
        string_file: Box<dyn StringFile>,
        serialization: Box<dyn DbLoadaProjectSerialization>,
    ) -> Self {
        YamlDbLoadaProjectIO {
            logger,
            string_file,
            serialization,
        }
    }
}

impl DbLoadaProjectIO for YamlDbLoadaProjectIO {
    fn load(&self, path: &Path) -> Result<DBLoadaProject, DbLoadaProjectIOError> {
        self.logger.debug(&format!("loading project from: {}", path.display()));
        let content = self.string_file.load(path)?;
        let project = self.serialization.deserialize(&content)?;
        self.logger.info(&format!("loaded project '{}' from: {}", project.name, path.display()));
        Ok(project)
    }

    fn save(&self, project: &DBLoadaProject, path: &Path) -> Result<(), DbLoadaProjectIOError> {
        self.logger.debug(&format!("saving project '{}' to: {}", project.name, path.display()));
        let content = self.serialization.serialize(project)?;
        self.string_file.save(&content, path)?;
        self.logger.info(&format!("saved project '{}' to: {}", project.name, path.display()));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::test_helpers::{InMemoryStringFile, TestLogger};
    use crate::components::db_loada_project_serialization::YamlDbLoadaProjectSerialization;
    use crate::traits::{DBLOADA_PROJECT_API_VERSION, ProjectSpec};
    use std::path::PathBuf;

    fn make_io() -> (YamlDbLoadaProjectIO, std::rc::Rc<std::cell::RefCell<std::collections::HashMap<PathBuf, String>>>) {
        let store = std::rc::Rc::new(std::cell::RefCell::new(std::collections::HashMap::new()));
        let string_file = Box::new(InMemoryStringFile::new(store.clone()));
        let serialization = Box::new(YamlDbLoadaProjectSerialization::new(Box::new(TestLogger)));
        let io = YamlDbLoadaProjectIO::new(Box::new(TestLogger), string_file, serialization);
        (io, store)
    }

    fn test_project(name: &str) -> DBLoadaProject {
        DBLoadaProject {
            name: name.to_string(),
            api_version: DBLOADA_PROJECT_API_VERSION.to_string(),
            spec: ProjectSpec { tables: vec![] },
        }
    }

    #[test]
    fn save_and_load_round_trip() {
        let (io, _store) = make_io();
        let path = PathBuf::from("/projects/dbloada.yaml");
        let project = test_project("my-project");

        io.save(&project, &path).unwrap();
        let loaded = io.load(&path).unwrap();

        assert_eq!(project, loaded);
    }

    #[test]
    fn save_and_load_with_different_names() {
        let (io, _store) = make_io();

        for name in &["alpha", "beta-project", "test-123"] {
            let path = PathBuf::from(format!("/projects/{name}/dbloada.yaml"));
            let project = test_project(name);

            io.save(&project, &path).unwrap();
            let loaded = io.load(&path).unwrap();

            assert_eq!(project, loaded, "round-trip failed for name: {name}");
        }
    }

    #[test]
    fn load_from_nonexistent_path_returns_error() {
        let (io, _store) = make_io();
        let path = PathBuf::from("/nonexistent/dbloada.yaml");

        let result = io.load(&path);

        assert!(result.is_err());
    }

    #[test]
    fn load_from_invalid_yaml_content_returns_error() {
        let (io, store) = make_io();
        let path = PathBuf::from("/projects/dbloada.yaml");
        store.borrow_mut().insert(path.clone(), "not valid yaml {{{{".to_string());

        let result = io.load(&path);

        assert!(result.is_err());
    }

    #[test]
    fn save_overwrites_existing_file() {
        let (io, _store) = make_io();
        let path = PathBuf::from("/projects/dbloada.yaml");

        let project1 = test_project("first");
        io.save(&project1, &path).unwrap();

        let project2 = test_project("second");
        io.save(&project2, &path).unwrap();

        let loaded = io.load(&path).unwrap();
        assert_eq!(loaded, project2);
    }

    #[test]
    fn load_preserves_api_version() {
        let (io, _store) = make_io();
        let path = PathBuf::from("/projects/dbloada.yaml");
        let project = test_project("test");

        io.save(&project, &path).unwrap();
        let loaded = io.load(&path).unwrap();

        assert_eq!(loaded.api_version, DBLOADA_PROJECT_API_VERSION);
    }

    #[test]
    fn multiple_projects_at_different_paths() {
        let (io, _store) = make_io();
        let path_a = PathBuf::from("/projects/a/dbloada.yaml");
        let path_b = PathBuf::from("/projects/b/dbloada.yaml");
        let project_a = test_project("project-a");
        let project_b = test_project("project-b");

        io.save(&project_a, &path_a).unwrap();
        io.save(&project_b, &path_b).unwrap();

        let loaded_a = io.load(&path_a).unwrap();
        let loaded_b = io.load(&path_b).unwrap();

        assert_eq!(loaded_a, project_a);
        assert_eq!(loaded_b, project_b);
    }
}
