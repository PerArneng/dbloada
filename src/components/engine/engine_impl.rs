use std::path::Path;
use async_trait::async_trait;
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

#[async_trait]
impl Engine for EngineImpl {
    async fn init(&self) {
        self.logger.info("hello").await;
    }

    async fn init_project_dir(&self, path: &Path, name: Option<&str>) -> Result<(), InitError> {
        self.init.init(path, name).await
    }

    async fn load_project(&self, path: &Path) -> Result<Project, LoadError> {
        self.load.load(path).await
    }
}
