use thiserror::Error;

pub const DBLOADA_PROJECT_API_VERSION: &str = "project.dbloada.io/v1";
pub const DBLOADA_PROJECT_KIND: &str = "DBLoadaProject";

#[derive(Debug, Clone, PartialEq)]
pub struct DBLoadaProject {
    pub name: String,
    pub api_version: String,
    pub spec: ProjectSpec,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProjectSpec {
    pub tables: Vec<TableSpec>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TableSpec {
    pub name: String,
    pub description: String,
    pub has_header: bool,
    pub source: SourceSpec,
    pub columns: Vec<ColumnSpec>,
    pub relationships: Vec<RelationshipSpec>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceSpec {
    pub filename: String,
    pub character_encoding: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ColumnSpec {
    pub name: String,
    pub description: String,
    pub column_identifier: ColumnIdentifier,
    pub column_type: ColumnType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ColumnIdentifier {
    Index(u64),
    Name(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ColumnType {
    String { max_length: Option<u64> },
    Int64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RelationshipSpec {
    pub name: String,
    pub description: String,
    pub source_column: String,
    pub target_table: String,
    pub target_column: String,
}

#[derive(Debug, Error)]
pub enum DbLoadaProjectSerializationError {
    #[error("failed to serialize project: {0}")]
    SerializeError(String),
    #[error("failed to deserialize project: {0}")]
    DeserializeError(String),
    #[error("unexpected kind: expected '{expected}', got '{actual}'")]
    UnexpectedKind { expected: String, actual: String },
}

pub trait DbLoadaProjectSerialization {
    fn serialize(&self, project: &DBLoadaProject) -> Result<String, DbLoadaProjectSerializationError>;
    fn deserialize(&self, content: &str) -> Result<DBLoadaProject, DbLoadaProjectSerializationError>;
}
