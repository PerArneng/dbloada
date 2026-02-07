use std::path::Path;
use crate::traits::{DBLoadaProject, DbLoadaEngine, Init, InitError, Load, LoadError, Logger};

pub struct DbLoadaEngineImpl {
    logger: Box<dyn Logger>,
    init: Box<dyn Init>,
    load: Box<dyn Load>,
}

impl DbLoadaEngineImpl {
    pub fn new(logger: Box<dyn Logger>, init: Box<dyn Init>, load: Box<dyn Load>) -> Self {
        DbLoadaEngineImpl { logger, init, load }
    }
}

impl DbLoadaEngine for DbLoadaEngineImpl {
    fn init(&self) {
        self.logger.info("hello");
    }

    fn init_project_dir(&self, path: &Path, name: Option<&str>) -> Result<(), InitError> {
        self.init.init(path, name)
    }

    fn load_project(&self, path: &Path) -> Result<DBLoadaProject, LoadError> {
        self.load.load(path)
    }
}
