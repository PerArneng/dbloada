use std::path::Path;
use async_trait::async_trait;
use thiserror::Error;
use crate::models::{Table, TableSpec};
use super::file_system::FileSystemError;
use super::csv_parser::CsvParserError;

#[derive(Debug, Error)]
pub enum TableReaderError {
    #[error("no reader found for table '{0}'")]
    NoReaderFound(String),
    #[error("failed to read table '{table_name}': {message}")]
    ReadError { table_name: String, message: String },
    #[error(transparent)]
    FileSystemError(#[from] FileSystemError),
    #[error(transparent)]
    CsvParserError(#[from] CsvParserError),
}

#[async_trait]
pub trait TableReader: Send + Sync {
    fn name(&self) -> &str;
    fn can_read(&self, table: &TableSpec) -> bool;
    async fn read_table(&self, table: &TableSpec, project_dir: &Path) -> Result<Table, TableReaderError>;
}

pub async fn read(
    readers: &[Box<dyn TableReader>],
    table: &TableSpec,
    project_dir: &Path,
) -> Result<Table, TableReaderError> {
    for reader in readers {
        if reader.can_read(table) {
            return reader.read_table(table, project_dir).await;
        }
    }
    Err(TableReaderError::NoReaderFound(table.name.clone()))
}
