use thiserror::Error;

pub const DBLOADA_PROJECT_API_VERSION: &str = "project.dbloada.io/v1";
pub const DBLOADA_PROJECT_KIND: &str = "DBLoadaProject";

#[derive(Debug, Clone, PartialEq)]
pub struct DBLoadaProject {
    pub name: String,
    pub api_version: String,
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
