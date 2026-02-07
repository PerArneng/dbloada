use std::path::Path;
use crate::traits::{Project, Engine, Init, InitError, Load, LoadError, Logger};

pub struct EngineImpl {
    logger: Box<dyn Logger>,
    init: Box<dyn Init>,
    load: Box<dyn Load>,
}

impl EngineImpl {
    pub fn new(logger: Box<dyn Logger>, init: Box<dyn Init>, load: Box<dyn Load>) -> Self {
        EngineImpl { logger, init, load }
    }
}

impl Engine for EngineImpl {
    fn init(&self) {
        self.logger.info("hello");
    }

    fn init_project_dir(&self, path: &Path, name: Option<&str>) -> Result<(), InitError> {
        self.init.init(path, name)
    }

    fn load_project(&self, path: &Path) -> Result<Project, LoadError> {
        self.load.load(path)
    }
}
