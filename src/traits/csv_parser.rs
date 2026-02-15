use async_trait::async_trait;
use thiserror::Error;
use crate::models::{Table, TableSpec};

#[derive(Debug, Error)]
pub enum CsvParserError {
    #[error("failed to parse table '{table_name}': {message}")]
    ParseError { table_name: String, message: String },
}

#[async_trait]
pub trait CsvParser: Send + Sync {
    async fn parse(&self, content: &str, table: &TableSpec) -> Result<Table, CsvParserError>;
}
