use std::path::Path;
use async_trait::async_trait;
use crate::models::{
    Project, ProjectSpec, TableSpec, SourceSpec, FileSourceSpec, CmdSourceSpec,
    ColumnSpec, ColumnIdentifier, ColumnType,
    RelationshipSpec, PROJECT_API_VERSION,
};
use crate::traits::{ProjectIO, Init, InitError, Logger, FileSystem};

pub fn sanitize_resource_name(raw: &str) -> String {
    let s: String = raw
        .to_lowercase()
        .chars()
        .map(|c| if c == ' ' || c == '_' { '-' } else { c })
        .filter(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == '-')
        .collect();

    let s = s.trim_matches('-').to_string();

    // collapse consecutive hyphens
    let mut result = String::with_capacity(s.len());
    let mut prev_hyphen = false;
    for c in s.chars() {
        if c == '-' {
            if !prev_hyphen {
                result.push(c);
            }
            prev_hyphen = true;
        } else {
            result.push(c);
            prev_hyphen = false;
        }
    }

    // truncate to 63 chars, trim trailing hyphens
    if result.len() > 63 {
        result.truncate(63);
    }
    result.trim_end_matches('-').to_string()
}

pub fn validate_resource_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("name must not be empty".to_string());
    }
    if name.len() > 63 {
        return Err(format!(
            "name must be no more than 63 characters, got {}",
            name.len()
        ));
    }
    if !name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
        return Err(
            "name must contain only lowercase alphanumeric characters or '-'".to_string(),
        );
    }
    let first = name.chars().next().unwrap();
    if !first.is_ascii_alphanumeric() {
        return Err("name must start with an alphanumeric character".to_string());
    }
    let last = name.chars().last().unwrap();
    if !last.is_ascii_alphanumeric() {
        return Err("name must end with an alphanumeric character".to_string());
    }
    Ok(())
}

pub fn example_project(name: &str) -> Project {
    Project {
        name: name.to_string(),
        api_version: PROJECT_API_VERSION.to_string(),
        spec: ProjectSpec {
            tables: vec![
                TableSpec {
                    name: "country".to_string(),
                    description: "Countries where cities and by extension offices are located in".to_string(),
                    has_header: false,
                    source: SourceSpec::File(FileSourceSpec {
                        filename: "data/countries.csv".to_string(),
                        character_encoding: "utf-8".to_string(),
                    }),
                    columns: vec![
                        ColumnSpec {
                            name: "name".to_string(),
                            description: "The official name of the country".to_string(),
                            column_identifier: ColumnIdentifier::Index(0),
                            column_type: ColumnType::String,
                        },
                    ],
                    relationships: vec![],
                },
                TableSpec {
                    name: "city".to_string(),
                    description: "Cities located within a country".to_string(),
                    has_header: true,
                    source: SourceSpec::File(FileSourceSpec {
                        filename: "data/cities.csv".to_string(),
                        character_encoding: "utf-8".to_string(),
                    }),
                    columns: vec![
                        ColumnSpec {
                            name: "name".to_string(),
                            description: "The official name of the city".to_string(),
                            column_identifier: ColumnIdentifier::Name("Name".to_string()),
                            column_type: ColumnType::String,
                        },
                        ColumnSpec {
                            name: "country".to_string(),
                            description: "The country where the city is located in".to_string(),
                            column_identifier: ColumnIdentifier::Name("Country".to_string()),
                            column_type: ColumnType::String,
                        },
                    ],
                    relationships: vec![
                        RelationshipSpec {
                            name: "located_in_country".to_string(),
                            description: "The country where the city is located in".to_string(),
                            source_column: "country".to_string(),
                            target_table: "country".to_string(),
                            target_column: "name".to_string(),
                        },
                    ],
                },
                TableSpec {
                    name: "office".to_string(),
                    description: "The physical building where people in this company work".to_string(),
                    has_header: true,
                    source: SourceSpec::File(FileSourceSpec {
                        filename: "data/offices.csv".to_string(),
                        character_encoding: "utf-8".to_string(),
                    }),
                    columns: vec![
                        ColumnSpec {
                            name: "building_name".to_string(),
                            description: "The name of the building".to_string(),
                            column_identifier: ColumnIdentifier::Name("Building Name".to_string()),
                            column_type: ColumnType::String,
                        },
                        ColumnSpec {
                            name: "location".to_string(),
                            description: "The city where the office is located".to_string(),
                            column_identifier: ColumnIdentifier::Name("Location".to_string()),
                            column_type: ColumnType::String,
                        },
                    ],
                    relationships: vec![
                        RelationshipSpec {
                            name: "located_in".to_string(),
                            description: "The city where the office is located in".to_string(),
                            source_column: "location".to_string(),
                            target_table: "city".to_string(),
                            target_column: "name".to_string(),
                        },
                    ],
                },
                TableSpec {
                    name: "employee".to_string(),
                    description: "Employees generated by a script".to_string(),
                    has_header: true,
                    source: SourceSpec::Cmd(CmdSourceSpec {
                        command: "bash".to_string(),
                        args: vec!["scripts/generate-employees.sh".to_string()],
                        stdout: true,
                        character_encoding: "utf-8".to_string(),
                    }),
                    columns: vec![
                        ColumnSpec {
                            name: "name".to_string(),
                            description: "The employee name".to_string(),
                            column_identifier: ColumnIdentifier::Name("Name".to_string()),
                            column_type: ColumnType::String,
                        },
                        ColumnSpec {
                            name: "office".to_string(),
                            description: "The office where the employee works".to_string(),
                            column_identifier: ColumnIdentifier::Name("Office".to_string()),
                            column_type: ColumnType::String,
                        },
                    ],
                    relationships: vec![
                        RelationshipSpec {
                            name: "works_in".to_string(),
                            description: "The office where the employee works".to_string(),
                            source_column: "office".to_string(),
                            target_table: "office".to_string(),
                            target_column: "building_name".to_string(),
                        },
                    ],
                },
                TableSpec {
                    name: "department".to_string(),
                    description: "Departments generated by a script writing to a temp file".to_string(),
                    has_header: true,
                    source: SourceSpec::Cmd(CmdSourceSpec {
                        command: "bash".to_string(),
                        args: vec![
                            "scripts/generate-departments.sh".to_string(),
                            "$TEMP_CSV_PATH".to_string(),
                        ],
                        stdout: false,
                        character_encoding: "utf-8".to_string(),
                    }),
                    columns: vec![
                        ColumnSpec {
                            name: "name".to_string(),
                            description: "The department name".to_string(),
                            column_identifier: ColumnIdentifier::Name("Name".to_string()),
                            column_type: ColumnType::String,
                        },
                        ColumnSpec {
                            name: "head".to_string(),
                            description: "The head of the department".to_string(),
                            column_identifier: ColumnIdentifier::Name("Head".to_string()),
                            column_type: ColumnType::String,
                        },
                    ],
                    relationships: vec![
                        RelationshipSpec {
                            name: "headed_by".to_string(),
                            description: "The employee who heads this department".to_string(),
                            source_column: "head".to_string(),
                            target_table: "employee".to_string(),
                            target_column: "name".to_string(),
                        },
                    ],
                },
            ],
        },
    }
}

pub fn example_data_files() -> Vec<(&'static str, &'static str)> {
    vec![
        ("data/countries.csv", "\"United Kingdom\"\n\"Germany\"\n"),
        ("data/cities.csv", "\"Name\", \"Country\"\n\"London\", \"United Kingdom\"\n\"Berlin\", \"Germany\"\n"),
        ("data/offices.csv", "\"Building Name\", \"Location\"\n\"Star Tower\", \"London\"\n\"Mercator II\", \"Berlin\"\n"),
        ("scripts/generate-employees.sh", "#!/usr/bin/env bash\necho 'Name,Office'\necho 'Alice,Star Tower'\necho 'Bob,Mercator II'\n"),
        ("scripts/generate-departments.sh", "#!/usr/bin/env bash\nOUTPUT_FILE=\"$1\"\necho \"Writing departments to $OUTPUT_FILE\"\ncat > \"$OUTPUT_FILE\" <<CSV\nName,Head\nEngineering,Alice\nMarketing,Bob\nCSV\n"),
    ]
}

pub fn example_script_files() -> Vec<&'static str> {
    vec![
        "scripts/generate-employees.sh",
        "scripts/generate-departments.sh",
    ]
}

pub fn example_directories() -> Vec<&'static str> {
    vec!["data", "scripts"]
}

async fn is_directory_empty(path: &Path) -> Result<bool, InitError> {
    let mut entries = tokio::fs::read_dir(path).await.map_err(|e| {
        InitError::DirectoryNotFound(format!("{}: {}", path.display(), e))
    })?;
    Ok(entries.next_entry().await.map_err(|e| {
        InitError::DirectoryNotFound(format!("{}: {}", path.display(), e))
    })?.is_none())
}

pub struct InitImpl {
    logger: Box<dyn Logger>,
    project_io: Box<dyn ProjectIO>,
    file_system: Box<dyn FileSystem>,
}

impl InitImpl {
    pub fn new(logger: Box<dyn Logger>, project_io: Box<dyn ProjectIO>, file_system: Box<dyn FileSystem>) -> Self {
        InitImpl { logger, project_io, file_system }
    }

    fn resolve_name(path: &Path, name: Option<&str>) -> Result<String, InitError> {
        match name {
            Some(n) => {
                validate_resource_name(n).map_err(|reason| InitError::InvalidResourceName {
                    name: n.to_string(),
                    reason,
                })?;
                Ok(n.to_string())
            }
            None => {
                let absolute = path.canonicalize().map_err(|_| {
                    InitError::InvalidDirectoryName(path.display().to_string())
                })?;
                let dir_name = absolute
                    .file_name()
                    .and_then(|n| n.to_str())
                    .ok_or_else(|| InitError::InvalidDirectoryName(path.display().to_string()))?;

                let sanitized = sanitize_resource_name(dir_name);
                validate_resource_name(&sanitized).map_err(|reason| {
                    InitError::InvalidResourceName {
                        name: sanitized.clone(),
                        reason,
                    }
                })?;
                Ok(sanitized)
            }
        }
    }
}

#[async_trait]
impl Init for InitImpl {
    async fn init(&self, path: &Path, name: Option<&str>, force: bool) -> Result<(), InitError> {
        let metadata = tokio::fs::metadata(path).await;
        if metadata.is_err() || !metadata.unwrap().is_dir() {
            return Err(InitError::DirectoryNotFound(path.display().to_string()));
        }

        if !force {
            let is_empty = is_directory_empty(path).await?;
            if !is_empty {
                return Err(InitError::DirectoryNotEmpty(path.display().to_string()));
            }
        }

        let project_name = Self::resolve_name(path, name)?;

        for dir in example_directories() {
            let dir_path = path.join(dir);
            self.file_system.ensure_dir(&dir_path).await?;
            self.logger.info(&format!("created directory: {}", dir_path.display())).await;
        }

        for (relative_path, content) in example_data_files() {
            let file_path = path.join(relative_path);
            self.file_system.save(content, &file_path).await?;
            self.logger.info(&format!("created {}", file_path.display())).await;
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            for script in example_script_files() {
                let script_path = path.join(script);
                if let Ok(metadata) = tokio::fs::metadata(&script_path).await {
                    let mut perms = metadata.permissions();
                    perms.set_mode(0o755);
                    let _ = tokio::fs::set_permissions(&script_path, perms).await;
                }
            }
        }

        let project = example_project(&project_name);

        let file_path = path.join("dbloada.yaml");
        self.project_io.save(&project, &file_path).await?;

        self.logger.info(&format!("created {}", file_path.display())).await;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example_project_has_five_tables() {
        let project = example_project("test");
        assert_eq!(project.spec.tables.len(), 5);
    }

    #[test]
    fn example_project_table_names() {
        let project = example_project("test");
        let names: Vec<&str> = project.spec.tables.iter().map(|t| t.name.as_str()).collect();
        assert_eq!(names, vec!["country", "city", "office", "employee", "department"]);
    }

    #[test]
    fn example_project_uses_given_name() {
        let project = example_project("my-project");
        assert_eq!(project.name, "my-project");
    }

    #[test]
    fn example_project_has_correct_api_version() {
        let project = example_project("test");
        assert_eq!(project.api_version, PROJECT_API_VERSION);
    }

    #[test]
    fn example_project_city_has_relationship_to_country() {
        let project = example_project("test");
        let city = &project.spec.tables[1];
        assert_eq!(city.relationships.len(), 1);
        assert_eq!(city.relationships[0].target_table, "country");
    }

    #[test]
    fn example_project_office_has_relationship_to_city() {
        let project = example_project("test");
        let office = &project.spec.tables[2];
        assert_eq!(office.relationships.len(), 1);
        assert_eq!(office.relationships[0].target_table, "city");
    }

    #[test]
    fn example_project_employee_has_cmd_source() {
        let project = example_project("test");
        let employee = &project.spec.tables[3];
        match &employee.source {
            SourceSpec::Cmd(cs) => {
                assert_eq!(cs.command, "bash");
                assert!(cs.stdout);
            }
            _ => panic!("expected Cmd source for employee"),
        }
    }

    #[test]
    fn example_project_department_has_cmd_source_with_temp_file() {
        let project = example_project("test");
        let department = &project.spec.tables[4];
        match &department.source {
            SourceSpec::Cmd(cs) => {
                assert_eq!(cs.command, "bash");
                assert!(!cs.stdout);
                assert!(cs.args.contains(&"$TEMP_CSV_PATH".to_string()));
            }
            _ => panic!("expected Cmd source for department"),
        }
    }

    #[test]
    fn example_data_files_has_five_entries() {
        let files = example_data_files();
        assert_eq!(files.len(), 5);
    }

    #[test]
    fn example_data_files_paths_match_file_sources() {
        let project = example_project("test");
        let files = example_data_files();
        let file_paths: Vec<&str> = files.iter().map(|(p, _)| *p).collect();
        for table in &project.spec.tables {
            match &table.source {
                SourceSpec::File(fs) => {
                    assert!(
                        file_paths.contains(&fs.filename.as_str()),
                        "source filename '{}' not found in example data files",
                        fs.filename
                    );
                }
                SourceSpec::Cmd(cs) => {
                    // For cmd sources, check the script is in the data files
                    let script_path = format!("scripts/{}", cs.args[0].split('/').last().unwrap());
                    assert!(
                        file_paths.iter().any(|p| p.ends_with(cs.args[0].split('/').last().unwrap())),
                        "script '{}' not found in example data files",
                        script_path
                    );
                }
            }
        }
    }

    #[test]
    fn example_directories_contains_data_and_scripts() {
        let dirs = example_directories();
        assert_eq!(dirs, vec!["data", "scripts"]);
    }

    #[test]
    fn example_script_files_has_two_entries() {
        let scripts = example_script_files();
        assert_eq!(scripts.len(), 2);
    }

    #[tokio::test]
    async fn is_directory_empty_returns_true_for_empty_dir() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(is_directory_empty(tmp.path()).await.unwrap());
    }

    #[tokio::test]
    async fn is_directory_empty_returns_false_when_file_exists() {
        let tmp = tempfile::tempdir().unwrap();
        tokio::fs::write(tmp.path().join("something.txt"), "hello").await.unwrap();
        assert!(!is_directory_empty(tmp.path()).await.unwrap());
    }

    #[tokio::test]
    async fn init_rejects_non_empty_directory_without_force() {
        use crate::components::test_helpers::{mock_logger, mock_project_io, mock_file_system};

        let tmp = tempfile::tempdir().unwrap();
        tokio::fs::write(tmp.path().join("existing.txt"), "data").await.unwrap();

        let init = InitImpl::new(mock_logger(), mock_project_io(), mock_file_system());
        let result = init.init(tmp.path(), Some("test-proj"), false).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, InitError::DirectoryNotEmpty(_)));
    }

    #[tokio::test]
    async fn init_allows_non_empty_directory_with_force() {
        use crate::components::test_helpers::{mock_logger, mock_project_io, mock_file_system};

        let tmp = tempfile::tempdir().unwrap();
        tokio::fs::write(tmp.path().join("existing.txt"), "data").await.unwrap();

        let init = InitImpl::new(mock_logger(), mock_project_io(), mock_file_system());
        let result = init.init(tmp.path(), Some("test-proj"), true).await;

        assert!(result.is_ok());
    }
}
