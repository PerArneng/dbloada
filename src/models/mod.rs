pub mod project;

pub use project::{
    PROJECT_API_VERSION, PROJECT_KIND,
    Project, ProjectSpec, TableSpec, SourceSpec, ColumnSpec, ColumnIdentifier, ColumnType,
    RelationshipSpec,
};
