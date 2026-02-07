use std::path::Path;
use super::init::InitError;
use super::load::LoadError;
use super::db_loada_project_serialization::DBLoadaProject;

pub trait DbLoadaEngine {
    fn init(&self);
    fn init_project_dir(&self, path: &Path, name: Option<&str>) -> Result<(), InitError>;
    fn load_project(&self, path: &Path) -> Result<DBLoadaProject, LoadError>;
}
