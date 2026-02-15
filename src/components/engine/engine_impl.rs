use std::path::Path;
use async_trait::async_trait;
use crate::models::LoadedProject;
use crate::traits::{Engine, Init, InitError, Load, LoadError, Logger};

pub struct EngineImpl {
    logger: Box<dyn Logger>,
    init: Box<dyn Init>,
    load: Box<dyn Load>,
}

impl EngineImpl {
    pub fn new(
        logger: Box<dyn Logger>,
        init: Box<dyn Init>,
        load: Box<dyn Load>,
    ) -> Self {
        EngineImpl { logger, init, load }
    }
}

#[async_trait]
impl Engine for EngineImpl {
    async fn init(&self) {
        self.logger.info("hello").await;
    }

    async fn init_project_dir(&self, path: &Path, name: Option<&str>, force: bool) -> Result<(), InitError> {
        self.init.init(path, name, force).await
    }

    async fn load_project(&self, path: &Path) -> Result<LoadedProject, LoadError> {
        self.load.load(path).await
    }
}

#[cfg(test)]
mod tests {
    use crate::component_assembler::ComponentAssembler;

    #[tokio::test]
    async fn init_then_load_project_from_temp_directory() {
        let tmp = tempfile::tempdir().unwrap();
        let assembler = ComponentAssembler::new();
        let engine = assembler.engine();

        engine
            .init_project_dir(tmp.path(), Some("real-world-test"), false)
            .await
            .unwrap();

        let loaded = engine.load_project(tmp.path()).await.unwrap();
        assert_eq!(loaded.project.name, "real-world-test");
        assert_eq!(loaded.project.spec.tables.len(), 5);
        assert_eq!(loaded.tables.len(), 5);
    }
}
