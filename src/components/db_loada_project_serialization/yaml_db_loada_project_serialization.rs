use serde::{Deserialize, Serialize};
use crate::traits::{
    DBLoadaProject, DbLoadaProjectSerialization, DbLoadaProjectSerializationError,
    DBLOADA_PROJECT_KIND, Logger,
};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DbLoadaProjectYaml {
    api_version: String,
    kind: String,
    metadata: MetadataYaml,
    #[serde(default)]
    spec: serde_yaml::Value,
}

#[derive(Serialize, Deserialize)]
struct MetadataYaml {
    name: String,
}

pub fn serialize_to_yaml(project: &DBLoadaProject) -> Result<String, DbLoadaProjectSerializationError> {
    let yaml_model = DbLoadaProjectYaml {
        api_version: project.api_version.clone(),
        kind: DBLOADA_PROJECT_KIND.to_string(),
        metadata: MetadataYaml {
            name: project.name.clone(),
        },
        spec: serde_yaml::Value::Mapping(serde_yaml::Mapping::new()),
    };
    serde_yaml::to_string(&yaml_model)
        .map_err(|e| DbLoadaProjectSerializationError::SerializeError(e.to_string()))
}

pub fn deserialize_from_yaml(content: &str) -> Result<DBLoadaProject, DbLoadaProjectSerializationError> {
    let yaml_model: DbLoadaProjectYaml = serde_yaml::from_str(content)
        .map_err(|e| DbLoadaProjectSerializationError::DeserializeError(e.to_string()))?;

    if yaml_model.kind != DBLOADA_PROJECT_KIND {
        return Err(DbLoadaProjectSerializationError::UnexpectedKind {
            expected: DBLOADA_PROJECT_KIND.to_string(),
            actual: yaml_model.kind,
        });
    }

    Ok(DBLoadaProject {
        name: yaml_model.metadata.name,
        api_version: yaml_model.api_version,
    })
}

pub struct YamlDbLoadaProjectSerialization {
    logger: Box<dyn Logger>,
}

impl YamlDbLoadaProjectSerialization {
    pub fn new(logger: Box<dyn Logger>) -> Self {
        YamlDbLoadaProjectSerialization { logger }
    }
}

impl DbLoadaProjectSerialization for YamlDbLoadaProjectSerialization {
    fn serialize(&self, project: &DBLoadaProject) -> Result<String, DbLoadaProjectSerializationError> {
        self.logger.debug(&format!("serializing project: {}", project.name));
        let result = serialize_to_yaml(project)?;
        self.logger.info(&format!("serialized project: {}", project.name));
        Ok(result)
    }

    fn deserialize(&self, content: &str) -> Result<DBLoadaProject, DbLoadaProjectSerializationError> {
        self.logger.debug("deserializing project");
        let project = deserialize_from_yaml(content)?;
        self.logger.info(&format!("deserialized project: {}", project.name));
        Ok(project)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::init::init_impl::build_project_yaml;
    use crate::traits::DBLOADA_PROJECT_API_VERSION;

    #[test]
    fn serialize_to_yaml_produces_valid_yaml() {
        let project = DBLoadaProject {
            name: "test-project".to_string(),
            api_version: DBLOADA_PROJECT_API_VERSION.to_string(),
        };
        let yaml = serialize_to_yaml(&project).unwrap();
        assert!(yaml.contains("apiVersion"));
        assert!(yaml.contains("kind"));
        assert!(yaml.contains("metadata"));
        assert!(yaml.contains("name"));
        assert!(yaml.contains("spec"));
    }

    #[test]
    fn serialize_to_yaml_uses_correct_api_version() {
        let project = DBLoadaProject {
            name: "test".to_string(),
            api_version: DBLOADA_PROJECT_API_VERSION.to_string(),
        };
        let yaml = serialize_to_yaml(&project).unwrap();
        assert!(yaml.contains(DBLOADA_PROJECT_API_VERSION));
    }

    #[test]
    fn serialize_to_yaml_sets_kind_to_dbloada_project() {
        let project = DBLoadaProject {
            name: "test".to_string(),
            api_version: DBLOADA_PROJECT_API_VERSION.to_string(),
        };
        let yaml = serialize_to_yaml(&project).unwrap();
        assert!(yaml.contains("kind: DBLoadaProject"));
    }

    #[test]
    fn serialize_to_yaml_embeds_project_name_in_metadata() {
        let project = DBLoadaProject {
            name: "my-project".to_string(),
            api_version: DBLOADA_PROJECT_API_VERSION.to_string(),
        };
        let yaml = serialize_to_yaml(&project).unwrap();
        assert!(yaml.contains("name: my-project"));
    }

    #[test]
    fn serialize_to_yaml_with_hyphenated_name() {
        let project = DBLoadaProject {
            name: "my-cool-project".to_string(),
            api_version: DBLOADA_PROJECT_API_VERSION.to_string(),
        };
        let yaml = serialize_to_yaml(&project).unwrap();
        assert!(yaml.contains("name: my-cool-project"));
    }

    #[test]
    fn serialize_to_yaml_with_single_char_name() {
        let project = DBLoadaProject {
            name: "a".to_string(),
            api_version: DBLOADA_PROJECT_API_VERSION.to_string(),
        };
        let yaml = serialize_to_yaml(&project).unwrap();
        assert!(yaml.contains("name: a"));
    }

    #[test]
    fn serialize_to_yaml_with_long_name() {
        let name = "a".repeat(63);
        let project = DBLoadaProject {
            name: name.clone(),
            api_version: DBLOADA_PROJECT_API_VERSION.to_string(),
        };
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
        let project = DBLoadaProject {
            name: "test-project".to_string(),
            api_version: DBLOADA_PROJECT_API_VERSION.to_string(),
        };
        let yaml = serialize_to_yaml(&project).unwrap();
        let deserialized = deserialize_from_yaml(&yaml).unwrap();
        assert_eq!(project, deserialized);
    }

    #[test]
    fn round_trip_with_various_names() {
        for name in &["a", "my-project", "test-123-project"] {
            let project = DBLoadaProject {
                name: name.to_string(),
                api_version: DBLOADA_PROJECT_API_VERSION.to_string(),
            };
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
            Err(DbLoadaProjectSerializationError::UnexpectedKind { .. })
        ));
    }

    #[test]
    fn deserialize_rejects_empty_kind() {
        let yaml = "apiVersion: project.dbloada.io/v1\nkind: ''\nmetadata:\n  name: test\nspec: {}\n";
        let result = deserialize_from_yaml(yaml);
        assert!(matches!(
            result,
            Err(DbLoadaProjectSerializationError::UnexpectedKind { .. })
        ));
    }

    #[test]
    fn deserialize_rejects_invalid_yaml() {
        let yaml = "not: valid: yaml: {{{{";
        let result = deserialize_from_yaml(yaml);
        assert!(matches!(
            result,
            Err(DbLoadaProjectSerializationError::DeserializeError(_))
        ));
    }

    #[test]
    fn deserialize_rejects_empty_string() {
        let result = deserialize_from_yaml("");
        assert!(matches!(
            result,
            Err(DbLoadaProjectSerializationError::DeserializeError(_))
        ));
    }

    #[test]
    fn deserialize_rejects_missing_kind_field() {
        let yaml = "apiVersion: project.dbloada.io/v1\nmetadata:\n  name: test\nspec: {}\n";
        let result = deserialize_from_yaml(yaml);
        assert!(matches!(
            result,
            Err(DbLoadaProjectSerializationError::DeserializeError(_))
        ));
    }

    #[test]
    fn deserialize_rejects_missing_metadata() {
        let yaml = "apiVersion: project.dbloada.io/v1\nkind: DBLoadaProject\nspec: {}\n";
        let result = deserialize_from_yaml(yaml);
        assert!(matches!(
            result,
            Err(DbLoadaProjectSerializationError::DeserializeError(_))
        ));
    }

    #[test]
    fn deserialize_rejects_missing_name_in_metadata() {
        let yaml = "apiVersion: project.dbloada.io/v1\nkind: DBLoadaProject\nmetadata: {}\nspec: {}\n";
        let result = deserialize_from_yaml(yaml);
        assert!(matches!(
            result,
            Err(DbLoadaProjectSerializationError::DeserializeError(_))
        ));
    }

    #[test]
    fn deserialize_rejects_missing_api_version() {
        let yaml = "kind: DBLoadaProject\nmetadata:\n  name: test\nspec: {}\n";
        let result = deserialize_from_yaml(yaml);
        assert!(matches!(
            result,
            Err(DbLoadaProjectSerializationError::DeserializeError(_))
        ));
    }

    #[test]
    fn deserialize_handles_extra_fields_gracefully() {
        let yaml = "apiVersion: project.dbloada.io/v1\nkind: DBLoadaProject\nmetadata:\n  name: test\n  labels:\n    app: test\nspec: {}\nextra: field\n";
        let result = deserialize_from_yaml(yaml);
        assert!(result.is_ok());
    }

    #[test]
    fn deserialize_handles_nonempty_spec() {
        let yaml = "apiVersion: project.dbloada.io/v1\nkind: DBLoadaProject\nmetadata:\n  name: test\nspec:\n  database:\n    host: localhost\n";
        let result = deserialize_from_yaml(yaml);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().name, "test");
    }

    #[test]
    fn deserialize_handles_yaml_with_leading_document_marker() {
        let yaml = "---\napiVersion: project.dbloada.io/v1\nkind: DBLoadaProject\nmetadata:\n  name: test\nspec: {}\n";
        let result = deserialize_from_yaml(yaml);
        assert!(result.is_ok());
    }

    #[test]
    fn deserialize_from_build_project_yaml_output() {
        let yaml = build_project_yaml("my-project").unwrap();
        let project = deserialize_from_yaml(&yaml).unwrap();
        assert_eq!(project.name, "my-project");
        assert_eq!(project.api_version, DBLOADA_PROJECT_API_VERSION);
    }
}
