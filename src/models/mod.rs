pub mod project;
pub mod table;

pub use project::{
    PROJECT_API_VERSION, PROJECT_KIND,
    Project, ProjectSpec, TableSpec, SourceSpec, ColumnSpec, ColumnIdentifier, ColumnType,
    RelationshipSpec,
};
pub use table::{Table, table_to_string};
