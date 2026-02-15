pub const PROJECT_API_VERSION: &str = "project.dbloada.io/v1";
pub const PROJECT_KIND: &str = "DBLoadaProject";

#[derive(Debug, Clone, PartialEq)]
pub struct Project {
    pub name: String,
    pub api_version: String,
    pub spec: ProjectSpec,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProjectSpec {
    pub tables: Vec<TableSpec>,
}

#[derive(Debug)]
pub struct LoadedProject {
    pub project: Project,
    pub tables: Vec<super::table::Table>,
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
pub enum SourceSpec {
    File(FileSourceSpec),
    Cmd(CmdSourceSpec),
}

#[derive(Debug, Clone, PartialEq)]
pub struct FileSourceSpec {
    pub filename: String,
    pub character_encoding: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CmdSourceSpec {
    pub command: String,
    pub args: Vec<String>,
    pub stdout: bool,
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
    String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RelationshipSpec {
    pub name: String,
    pub description: String,
    pub source_column: String,
    pub target_table: String,
    pub target_column: String,
}
