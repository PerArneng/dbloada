use std::path::Path;
use crate::traits::{
    Project, ProjectIO, Init, InitError, Logger, FileSystem,
    ProjectSpec, TableSpec, SourceSpec, ColumnSpec, ColumnIdentifier, ColumnType,
    RelationshipSpec, PROJECT_API_VERSION,
};

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
                    source: SourceSpec {
                        filename: "data/countries.csv".to_string(),
                        character_encoding: "utf-8".to_string(),
                    },
                    columns: vec![
                        ColumnSpec {
                            name: "name".to_string(),
                            description: "The official name of the country".to_string(),
                            column_identifier: ColumnIdentifier::Index(0),
                            column_type: ColumnType::String { max_length: None },
                        },
                    ],
                    relationships: vec![],
                },
                TableSpec {
                    name: "city".to_string(),
                    description: "Cities located within a country".to_string(),
                    has_header: true,
                    source: SourceSpec {
                        filename: "data/cities.csv".to_string(),
                        character_encoding: "utf-8".to_string(),
                    },
                    columns: vec![
                        ColumnSpec {
                            name: "name".to_string(),
                            description: "The official name of the city".to_string(),
                            column_identifier: ColumnIdentifier::Name("Name".to_string()),
                            column_type: ColumnType::String { max_length: None },
                        },
                        ColumnSpec {
                            name: "country".to_string(),
                            description: "The country where the city is located in".to_string(),
                            column_identifier: ColumnIdentifier::Name("Country".to_string()),
                            column_type: ColumnType::String { max_length: None },
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
                    source: SourceSpec {
                        filename: "data/offices.csv".to_string(),
                        character_encoding: "utf-8".to_string(),
                    },
                    columns: vec![
                        ColumnSpec {
                            name: "building_name".to_string(),
                            description: "The name of the building".to_string(),
                            column_identifier: ColumnIdentifier::Name("Building Name".to_string()),
                            column_type: ColumnType::String { max_length: None },
                        },
                        ColumnSpec {
                            name: "location".to_string(),
                            description: "The city where the office is located".to_string(),
                            column_identifier: ColumnIdentifier::Name("Location".to_string()),
                            column_type: ColumnType::String { max_length: None },
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
            ],
        },
    }
}

pub fn example_data_files() -> Vec<(&'static str, &'static str)> {
    vec![
        ("data/countries.csv", "\"United Kingdom\"\n\"Germany\"\n"),
        ("data/cities.csv", "\"Name\", \"Country\"\n\"London\", \"United Kingdom\"\n\"Berlin\", \"Germany\"\n"),
        ("data/offices.csv", "\"Building Name\", \"Location\"\n\"Star Tower\", \"London\"\n\"Mercator II\", \"Berlin\"\n"),
    ]
}

pub fn example_directories() -> Vec<&'static str> {
    vec!["data", "scripts"]
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

impl Init for InitImpl {
    fn init(&self, path: &Path, name: Option<&str>) -> Result<(), InitError> {
        if !path.is_dir() {
            return Err(InitError::DirectoryNotFound(path.display().to_string()));
        }

        let project_name = Self::resolve_name(path, name)?;

        for dir in example_directories() {
            let dir_path = path.join(dir);
            self.file_system.ensure_dir(&dir_path)?;
            self.logger.info(&format!("created directory: {}", dir_path.display()));
        }

        for (relative_path, content) in example_data_files() {
            let file_path = path.join(relative_path);
            self.file_system.save(content, &file_path)?;
            self.logger.info(&format!("created {}", file_path.display()));
        }

        let project = example_project(&project_name);

        let file_path = path.join("dbloada.yaml");
        self.project_io.save(&project, &file_path)?;

        self.logger.info(&format!("created {}", file_path.display()));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example_project_has_three_tables() {
        let project = example_project("test");
        assert_eq!(project.spec.tables.len(), 3);
    }

    #[test]
    fn example_project_table_names() {
        let project = example_project("test");
        let names: Vec<&str> = project.spec.tables.iter().map(|t| t.name.as_str()).collect();
        assert_eq!(names, vec!["country", "city", "office"]);
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
    fn example_data_files_has_three_entries() {
        let files = example_data_files();
        assert_eq!(files.len(), 3);
    }

    #[test]
    fn example_data_files_paths_match_sources() {
        let project = example_project("test");
        let files = example_data_files();
        let file_paths: Vec<&str> = files.iter().map(|(p, _)| *p).collect();
        for table in &project.spec.tables {
            assert!(
                file_paths.contains(&table.source.filename.as_str()),
                "source filename '{}' not found in example data files",
                table.source.filename
            );
        }
    }

    #[test]
    fn example_directories_contains_data_and_scripts() {
        let dirs = example_directories();
        assert_eq!(dirs, vec!["data", "scripts"]);
    }
}
