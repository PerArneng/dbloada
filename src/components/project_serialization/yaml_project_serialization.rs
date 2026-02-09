use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use crate::models::{
    Project, PROJECT_KIND,
    ProjectSpec, TableSpec, SourceSpec, ColumnSpec, ColumnIdentifier, ColumnType, RelationshipSpec,
};
use crate::traits::{ProjectSerialization, ProjectSerializationError, Logger};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProjectYaml {
    api_version: String,
    kind: String,
    metadata: MetadataYaml,
    #[serde(default)]
    spec: Option<ProjectSpecYaml>,
}

#[derive(Serialize, Deserialize)]
struct MetadataYaml {
    name: String,
}

#[derive(Serialize, Deserialize)]
struct ProjectSpecYaml {
    #[serde(default)]
    tables: Vec<TableSpecYaml>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TableSpecYaml {
    name: String,
    description: String,
    has_header: bool,
    source: SourceSpecYaml,
    columns: Vec<ColumnSpecYaml>,
    #[serde(default)]
    relationships: Vec<RelationshipSpecYaml>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SourceSpecYaml {
    filename: String,
    character_encoding: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ColumnSpecYaml {
    name: String,
    description: String,
    column_identifier: ColumnIdentifierYaml,
    #[serde(rename = "type")]
    column_type: String,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum ColumnIdentifierYaml {
    Index(u64),
    Name(String),
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RelationshipSpecYaml {
    name: String,
    description: String,
    source_column: String,
    target_table: String,
    target_column: String,
}

pub fn parse_column_type(s: &str) -> Result<ColumnType, String> {
    let trimmed = s.trim();
    if trimmed == "int64" {
        return Ok(ColumnType::Int64);
    }
    if trimmed == "string" {
        return Ok(ColumnType::String { max_length: None });
    }
    if trimmed.starts_with("string(") && trimmed.ends_with(')') {
        let inner = &trimmed[7..trimmed.len() - 1];
        let max_length: u64 = inner
            .parse()
            .map_err(|_| format!("invalid max_length in type '{trimmed}'"))?;
        return Ok(ColumnType::String {
            max_length: Some(max_length),
        });
    }
    Err(format!("unknown column type: '{trimmed}'"))
}

pub fn column_type_to_string(ct: &ColumnType) -> String {
    match ct {
        ColumnType::String { max_length: None } => "string".to_string(),
        ColumnType::String {
            max_length: Some(len),
        } => format!("string({len})"),
        ColumnType::Int64 => "int64".to_string(),
    }
}

fn spec_to_yaml(spec: &ProjectSpec) -> ProjectSpecYaml {
    ProjectSpecYaml {
        tables: spec.tables.iter().map(table_to_yaml).collect(),
    }
}

fn table_to_yaml(table: &TableSpec) -> TableSpecYaml {
    TableSpecYaml {
        name: table.name.clone(),
        description: table.description.clone(),
        has_header: table.has_header,
        source: SourceSpecYaml {
            filename: table.source.filename.clone(),
            character_encoding: table.source.character_encoding.clone(),
        },
        columns: table.columns.iter().map(column_to_yaml).collect(),
        relationships: table.relationships.iter().map(relationship_to_yaml).collect(),
    }
}

fn column_to_yaml(col: &ColumnSpec) -> ColumnSpecYaml {
    ColumnSpecYaml {
        name: col.name.clone(),
        description: col.description.clone(),
        column_identifier: match &col.column_identifier {
            ColumnIdentifier::Index(i) => ColumnIdentifierYaml::Index(*i),
            ColumnIdentifier::Name(n) => ColumnIdentifierYaml::Name(n.clone()),
        },
        column_type: column_type_to_string(&col.column_type),
    }
}

fn relationship_to_yaml(rel: &RelationshipSpec) -> RelationshipSpecYaml {
    RelationshipSpecYaml {
        name: rel.name.clone(),
        description: rel.description.clone(),
        source_column: rel.source_column.clone(),
        target_table: rel.target_table.clone(),
        target_column: rel.target_column.clone(),
    }
}

fn spec_from_yaml(yaml: Option<ProjectSpecYaml>) -> Result<ProjectSpec, ProjectSerializationError> {
    match yaml {
        None => Ok(ProjectSpec { tables: vec![] }),
        Some(spec_yaml) => {
            let tables = spec_yaml
                .tables
                .into_iter()
                .map(table_from_yaml)
                .collect::<Result<Vec<_>, _>>()?;
            Ok(ProjectSpec { tables })
        }
    }
}

fn table_from_yaml(yaml: TableSpecYaml) -> Result<TableSpec, ProjectSerializationError> {
    let columns = yaml
        .columns
        .into_iter()
        .map(column_from_yaml)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(TableSpec {
        name: yaml.name,
        description: yaml.description,
        has_header: yaml.has_header,
        source: SourceSpec {
            filename: yaml.source.filename,
            character_encoding: yaml.source.character_encoding,
        },
        columns,
        relationships: yaml
            .relationships
            .into_iter()
            .map(|r| RelationshipSpec {
                name: r.name,
                description: r.description,
                source_column: r.source_column,
                target_table: r.target_table,
                target_column: r.target_column,
            })
            .collect(),
    })
}

fn column_from_yaml(yaml: ColumnSpecYaml) -> Result<ColumnSpec, ProjectSerializationError> {
    let column_type = parse_column_type(&yaml.column_type)
        .map_err(|e| ProjectSerializationError::DeserializeError(e))?;
    let column_identifier = match yaml.column_identifier {
        ColumnIdentifierYaml::Index(i) => ColumnIdentifier::Index(i),
        ColumnIdentifierYaml::Name(n) => ColumnIdentifier::Name(n),
    };
    Ok(ColumnSpec {
        name: yaml.name,
        description: yaml.description,
        column_identifier,
        column_type,
    })
}

pub fn serialize_to_yaml(project: &Project) -> Result<String, ProjectSerializationError> {
    let yaml_model = ProjectYaml {
        api_version: project.api_version.clone(),
        kind: PROJECT_KIND.to_string(),
        metadata: MetadataYaml {
            name: project.name.clone(),
        },
        spec: if project.spec.tables.is_empty() {
            None
        } else {
            Some(spec_to_yaml(&project.spec))
        },
    };
    serde_yaml::to_string(&yaml_model)
        .map_err(|e| ProjectSerializationError::SerializeError(e.to_string()))
}

pub fn deserialize_from_yaml(content: &str) -> Result<Project, ProjectSerializationError> {
    let yaml_model: ProjectYaml = serde_yaml::from_str(content)
        .map_err(|e| ProjectSerializationError::DeserializeError(e.to_string()))?;

    if yaml_model.kind != PROJECT_KIND {
        return Err(ProjectSerializationError::UnexpectedKind {
            expected: PROJECT_KIND.to_string(),
            actual: yaml_model.kind,
        });
    }

    let spec = spec_from_yaml(yaml_model.spec)?;

    Ok(Project {
        name: yaml_model.metadata.name,
        api_version: yaml_model.api_version,
        spec,
    })
}

pub struct YamlProjectSerialization {
    logger: Box<dyn Logger>,
}

impl YamlProjectSerialization {
    pub fn new(logger: Box<dyn Logger>) -> Self {
        YamlProjectSerialization { logger }
    }
}

#[async_trait]
impl ProjectSerialization for YamlProjectSerialization {
    async fn serialize(&self, project: &Project) -> Result<String, ProjectSerializationError> {
        self.logger.debug(&format!("serializing project: {}", project.name)).await;
        let result = serialize_to_yaml(project)?;
        self.logger.info(&format!("serialized project: {}", project.name)).await;
        Ok(result)
    }

    async fn deserialize(&self, content: &str) -> Result<Project, ProjectSerializationError> {
        self.logger.debug("deserializing project").await;
        let project = deserialize_from_yaml(content)?;
        self.logger.info(&format!("deserialized project: {}", project.name)).await;
        Ok(project)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::PROJECT_API_VERSION;

    fn empty_spec_project(name: &str) -> Project {
        Project {
            name: name.to_string(),
            api_version: PROJECT_API_VERSION.to_string(),
            spec: ProjectSpec { tables: vec![] },
        }
    }

    #[test]
    fn serialize_to_yaml_produces_valid_yaml() {
        let project = empty_spec_project("test-project");
        let yaml = serialize_to_yaml(&project).unwrap();
        assert!(yaml.contains("apiVersion"));
        assert!(yaml.contains("kind"));
        assert!(yaml.contains("metadata"));
        assert!(yaml.contains("name"));
    }

    #[test]
    fn serialize_to_yaml_uses_correct_api_version() {
        let project = empty_spec_project("test");
        let yaml = serialize_to_yaml(&project).unwrap();
        assert!(yaml.contains(PROJECT_API_VERSION));
    }

    #[test]
    fn serialize_to_yaml_sets_kind_to_dbloada_project() {
        let project = empty_spec_project("test");
        let yaml = serialize_to_yaml(&project).unwrap();
        assert!(yaml.contains("kind: DBLoadaProject"));
    }

    #[test]
    fn serialize_to_yaml_embeds_project_name_in_metadata() {
        let project = empty_spec_project("my-project");
        let yaml = serialize_to_yaml(&project).unwrap();
        assert!(yaml.contains("name: my-project"));
    }

    #[test]
    fn serialize_to_yaml_with_hyphenated_name() {
        let project = empty_spec_project("my-cool-project");
        let yaml = serialize_to_yaml(&project).unwrap();
        assert!(yaml.contains("name: my-cool-project"));
    }

    #[test]
    fn serialize_to_yaml_with_single_char_name() {
        let project = empty_spec_project("a");
        let yaml = serialize_to_yaml(&project).unwrap();
        assert!(yaml.contains("name: a"));
    }

    #[test]
    fn serialize_to_yaml_with_long_name() {
        let name = "a".repeat(63);
        let project = empty_spec_project(&name);
        let yaml = serialize_to_yaml(&project).unwrap();
        assert!(yaml.contains(&format!("name: {name}")));
    }

    #[test]
    fn deserialize_from_yaml_parses_valid_yaml() {
        let yaml = "apiVersion: project.dbloada.io/v1\nkind: DBLoadaProject\nmetadata:\n  name: test\nspec: {}\n";
        let result = deserialize_from_yaml(yaml);
        assert!(result.is_ok());
    }

    #[test]
    fn deserialize_from_yaml_extracts_correct_name() {
        let yaml = "apiVersion: project.dbloada.io/v1\nkind: DBLoadaProject\nmetadata:\n  name: my-project\nspec: {}\n";
        let project = deserialize_from_yaml(yaml).unwrap();
        assert_eq!(project.name, "my-project");
    }

    #[test]
    fn deserialize_from_yaml_extracts_correct_api_version() {
        let yaml = "apiVersion: project.dbloada.io/v1\nkind: DBLoadaProject\nmetadata:\n  name: test\nspec: {}\n";
        let project = deserialize_from_yaml(yaml).unwrap();
        assert_eq!(project.api_version, "project.dbloada.io/v1");
    }

    #[test]
    fn round_trip_preserves_data() {
        let project = empty_spec_project("test-project");
        let yaml = serialize_to_yaml(&project).unwrap();
        let deserialized = deserialize_from_yaml(&yaml).unwrap();
        assert_eq!(project, deserialized);
    }

    #[test]
    fn round_trip_with_various_names() {
        for name in &["a", "my-project", "test-123-project"] {
            let project = empty_spec_project(name);
            let yaml = serialize_to_yaml(&project).unwrap();
            let deserialized = deserialize_from_yaml(&yaml).unwrap();
            assert_eq!(project, deserialized, "round-trip failed for name: {name}");
        }
    }

    #[test]
    fn deserialize_rejects_wrong_kind() {
        let yaml = "apiVersion: project.dbloada.io/v1\nkind: WrongKind\nmetadata:\n  name: test\nspec: {}\n";
        let result = deserialize_from_yaml(yaml);
        assert!(matches!(
            result,
            Err(ProjectSerializationError::UnexpectedKind { .. })
        ));
    }

    #[test]
    fn deserialize_rejects_empty_kind() {
        let yaml = "apiVersion: project.dbloada.io/v1\nkind: ''\nmetadata:\n  name: test\nspec: {}\n";
        let result = deserialize_from_yaml(yaml);
        assert!(matches!(
            result,
            Err(ProjectSerializationError::UnexpectedKind { .. })
        ));
    }

    #[test]
    fn deserialize_rejects_invalid_yaml() {
        let yaml = "not: valid: yaml: {{{{";
        let result = deserialize_from_yaml(yaml);
        assert!(matches!(
            result,
            Err(ProjectSerializationError::DeserializeError(_))
        ));
    }

    #[test]
    fn deserialize_rejects_empty_string() {
        let result = deserialize_from_yaml("");
        assert!(matches!(
            result,
            Err(ProjectSerializationError::DeserializeError(_))
        ));
    }

    #[test]
    fn deserialize_rejects_missing_kind_field() {
        let yaml = "apiVersion: project.dbloada.io/v1\nmetadata:\n  name: test\nspec: {}\n";
        let result = deserialize_from_yaml(yaml);
        assert!(matches!(
            result,
            Err(ProjectSerializationError::DeserializeError(_))
        ));
    }

    #[test]
    fn deserialize_rejects_missing_metadata() {
        let yaml = "apiVersion: project.dbloada.io/v1\nkind: DBLoadaProject\nspec: {}\n";
        let result = deserialize_from_yaml(yaml);
        assert!(matches!(
            result,
            Err(ProjectSerializationError::DeserializeError(_))
        ));
    }

    #[test]
    fn deserialize_rejects_missing_name_in_metadata() {
        let yaml = "apiVersion: project.dbloada.io/v1\nkind: DBLoadaProject\nmetadata: {}\nspec: {}\n";
        let result = deserialize_from_yaml(yaml);
        assert!(matches!(
            result,
            Err(ProjectSerializationError::DeserializeError(_))
        ));
    }

    #[test]
    fn deserialize_rejects_missing_api_version() {
        let yaml = "kind: DBLoadaProject\nmetadata:\n  name: test\nspec: {}\n";
        let result = deserialize_from_yaml(yaml);
        assert!(matches!(
            result,
            Err(ProjectSerializationError::DeserializeError(_))
        ));
    }

    #[test]
    fn deserialize_handles_extra_fields_gracefully() {
        let yaml = "apiVersion: project.dbloada.io/v1\nkind: DBLoadaProject\nmetadata:\n  name: test\n  labels:\n    app: test\nspec: {}\nextra: field\n";
        let result = deserialize_from_yaml(yaml);
        assert!(result.is_ok());
    }

    #[test]
    fn deserialize_handles_yaml_with_leading_document_marker() {
        let yaml = "---\napiVersion: project.dbloada.io/v1\nkind: DBLoadaProject\nmetadata:\n  name: test\nspec: {}\n";
        let result = deserialize_from_yaml(yaml);
        assert!(result.is_ok());
    }

    #[test]
    fn deserialize_handles_missing_spec() {
        let yaml = "apiVersion: project.dbloada.io/v1\nkind: DBLoadaProject\nmetadata:\n  name: test\n";
        let project = deserialize_from_yaml(yaml).unwrap();
        assert_eq!(project.spec.tables.len(), 0);
    }

    #[test]
    fn parse_column_type_string() {
        assert_eq!(
            parse_column_type("string"),
            Ok(ColumnType::String { max_length: None })
        );
    }

    #[test]
    fn parse_column_type_string_with_max_length() {
        assert_eq!(
            parse_column_type("string(50)"),
            Ok(ColumnType::String {
                max_length: Some(50)
            })
        );
    }

    #[test]
    fn parse_column_type_int64() {
        assert_eq!(parse_column_type("int64"), Ok(ColumnType::Int64));
    }

    #[test]
    fn parse_column_type_unknown_returns_error() {
        assert!(parse_column_type("boolean").is_err());
    }

    #[test]
    fn column_type_to_string_roundtrip() {
        let types = vec![
            ColumnType::String { max_length: None },
            ColumnType::String {
                max_length: Some(100),
            },
            ColumnType::Int64,
        ];
        for ct in types {
            let s = column_type_to_string(&ct);
            let parsed = parse_column_type(&s).unwrap();
            assert_eq!(ct, parsed);
        }
    }

    #[test]
    fn deserialize_full_spec_yaml() {
        let yaml = r#"
apiVersion: project.dbloada.io/v1
kind: DBLoadaProject
metadata:
  name: testdata
spec:
  tables:
    - name: country
      description: Countries
      hasHeader: false
      source:
        filename: data/countries.csv
        characterEncoding: utf-8
      columns:
        - name: name
          description: The name
          columnIdentifier: 0
          type: string
    - name: city
      description: Cities
      hasHeader: true
      source:
        filename: data/cities.csv
        characterEncoding: utf-8
      columns:
        - name: name
          description: City name
          columnIdentifier: "Name"
          type: string
        - name: country
          description: Country
          columnIdentifier: "Country"
          type: string(50)
      relationships:
        - name: located_in_country
          description: Country where city is
          sourceColumn: country
          targetTable: country
          targetColumn: name
"#;
        let project = deserialize_from_yaml(yaml).unwrap();
        assert_eq!(project.name, "testdata");
        assert_eq!(project.spec.tables.len(), 2);

        let country = &project.spec.tables[0];
        assert_eq!(country.name, "country");
        assert!(!country.has_header);
        assert_eq!(country.source.filename, "data/countries.csv");
        assert_eq!(country.columns.len(), 1);
        assert_eq!(country.columns[0].column_identifier, ColumnIdentifier::Index(0));
        assert_eq!(country.columns[0].column_type, ColumnType::String { max_length: None });
        assert!(country.relationships.is_empty());

        let city = &project.spec.tables[1];
        assert_eq!(city.name, "city");
        assert!(city.has_header);
        assert_eq!(city.columns.len(), 2);
        assert_eq!(city.columns[0].column_identifier, ColumnIdentifier::Name("Name".to_string()));
        assert_eq!(
            city.columns[1].column_type,
            ColumnType::String {
                max_length: Some(50)
            }
        );
        assert_eq!(city.relationships.len(), 1);
        assert_eq!(city.relationships[0].name, "located_in_country");
        assert_eq!(city.relationships[0].target_table, "country");
    }

    #[test]
    fn round_trip_with_full_spec() {
        let project = Project {
            name: "test".to_string(),
            api_version: PROJECT_API_VERSION.to_string(),
            spec: ProjectSpec {
                tables: vec![TableSpec {
                    name: "users".to_string(),
                    description: "User table".to_string(),
                    has_header: true,
                    source: SourceSpec {
                        filename: "data/users.csv".to_string(),
                        character_encoding: "utf-8".to_string(),
                    },
                    columns: vec![ColumnSpec {
                        name: "id".to_string(),
                        description: "User ID".to_string(),
                        column_identifier: ColumnIdentifier::Index(0),
                        column_type: ColumnType::Int64,
                    }],
                    relationships: vec![],
                }],
            },
        };
        let yaml = serialize_to_yaml(&project).unwrap();
        let deserialized = deserialize_from_yaml(&yaml).unwrap();
        assert_eq!(project, deserialized);
    }
}
