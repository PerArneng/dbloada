use std::path::Path;
use crate::traits::{DbLoadaEngine, Init, InitError, Logger};

pub struct DbLoadaEngineImpl {
    logger: Box<dyn Logger>,
    init: Box<dyn Init>,
}

impl DbLoadaEngineImpl {
    pub fn new(logger: Box<dyn Logger>, init: Box<dyn Init>) -> Self {
        DbLoadaEngineImpl { logger, init }
    }
}

impl DbLoadaEngine for DbLoadaEngineImpl {
    fn init(&self) {
        self.logger.info("hello");
    }

    fn init_project_dir(&self, path: &Path, name: Option<&str>) -> Result<(), InitError> {
        self.init.init(path, name)
    }
}
