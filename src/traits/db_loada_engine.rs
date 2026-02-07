use std::path::Path;
use super::init::InitError;

pub trait DbLoadaEngine {
    fn init(&self);
    fn init_project_dir(&self, path: &Path, name: Option<&str>) -> Result<(), InitError>;
}
