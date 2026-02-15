use std::path::Path;
use async_trait::async_trait;
use crate::models::{Project, Table};
use crate::traits::{Engine, Init, InitError, Load, LoadError, Logger, TableReader, TableReaderError};
use crate::traits::table_reader;

pub struct EngineImpl {
    logger: Box<dyn Logger>,
    init: Box<dyn Init>,
    load: Box<dyn Load>,
    table_readers: Vec<Box<dyn TableReader>>,
}

impl EngineImpl {
    pub fn new(
        logger: Box<dyn Logger>,
        init: Box<dyn Init>,
        load: Box<dyn Load>,
        table_readers: Vec<Box<dyn TableReader>>,
    ) -> Self {
        EngineImpl { logger, init, load, table_readers }
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

    async fn read_tables(&self, project: &Project, project_dir: &Path) -> Result<Vec<Table>, TableReaderError> {
        let mut tables = Vec::new();
        for table_spec in &project.spec.tables {
            self.logger.debug(&format!("reading table '{}'", table_spec.name)).await;
            let table = table_reader::read(&self.table_readers, table_spec, project_dir).await?;
            self.logger.info(&format!(
                "loaded table '{}': {} rows, {} columns",
                table.name,
                table.num_rows(),
                table.num_columns(),
            )).await;
            tables.push(table);
        }
        Ok(tables)
    }
}
