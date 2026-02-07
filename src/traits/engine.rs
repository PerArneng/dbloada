use std::path::Path;
use super::init::InitError;
use super::load::LoadError;
use super::project_serialization::Project;

pub trait Engine {
    fn init(&self);
    fn init_project_dir(&self, path: &Path, name: Option<&str>) -> Result<(), InitError>;
    fn load_project(&self, path: &Path) -> Result<Project, LoadError>;
}
