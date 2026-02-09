use async_trait::async_trait;
use thiserror::Error;
use crate::models::{Project};

#[derive(Debug, Error)]
pub enum ProjectSerializationError {
    #[error("failed to serialize project: {0}")]
    SerializeError(String),
    #[error("failed to deserialize project: {0}")]
    DeserializeError(String),
    #[error("unexpected kind: expected '{expected}', got '{actual}'")]
    UnexpectedKind { expected: String, actual: String },
}

#[async_trait]
pub trait ProjectSerialization: Send + Sync {
    async fn serialize(&self, project: &Project) -> Result<String, ProjectSerializationError>;
    async fn deserialize(&self, content: &str) -> Result<Project, ProjectSerializationError>;
}
