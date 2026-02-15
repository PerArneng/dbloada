use std::path::{Path, PathBuf};
use async_trait::async_trait;
use crate::models::{LoadedProject, Project, Table};
use crate::traits::{ProjectIO, Load, LoadError, Logger, TableReader};
use crate::traits::table_reader;

pub const DBLOADA_PROJECT_FILENAME: &str = "dbloada.yaml";

pub fn project_file_path(dir: &Path) -> PathBuf {
    dir.join(DBLOADA_PROJECT_FILENAME)
}

pub struct LoadImpl {
    logger: Box<dyn Logger>,
    project_io: Box<dyn ProjectIO>,
    table_readers: Vec<Box<dyn TableReader>>,
}

impl LoadImpl {
    pub fn new(
        logger: Box<dyn Logger>,
        project_io: Box<dyn ProjectIO>,
        table_readers: Vec<Box<dyn TableReader>>,
    ) -> Self {
        LoadImpl {
            logger,
            project_io,
            table_readers,
        }
    }

    async fn read_tables(&self, project: &Project, project_dir: &Path) -> Result<Vec<Table>, LoadError> {
        let mut tables = Vec::new();
        for table_spec in &project.spec.tables {
            self.logger.debug(&format!("reading table '{}'", table_spec.name)).await;
            let table = table_reader::read(&self.table_readers, table_spec, project_dir).await?;
            self.logger.info(&format!(
                "loaded table '{}': {} rows, {} columns",
                table.name,
                table.num_rows(),
                table.num_columns(),
            )).await;
            tables.push(table);
        }
        Ok(tables)
    }
}

#[async_trait]
impl Load for LoadImpl {
    async fn load(&self, path: &Path) -> Result<LoadedProject, LoadError> {
        let metadata = tokio::fs::metadata(path).await;
        if metadata.is_err() || !metadata.unwrap().is_dir() {
            return Err(LoadError::DirectoryNotFound(path.display().to_string()));
        }

        let file_path = project_file_path(path);
        let file_metadata = tokio::fs::metadata(&file_path).await;
        if file_metadata.is_err() {
            return Err(LoadError::ProjectFileNotFound(file_path.display().to_string()));
        }

        self.logger.debug(&format!("loading project from: {}", file_path.display())).await;
        let project = self.project_io.load(&file_path).await?;
        self.logger.info(&format!("loaded project '{}' from: {}", project.name, file_path.display())).await;
        let tables = self.read_tables(&project, path).await?;

        Ok(LoadedProject { project, tables })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_file_path_appends_filename() {
        let path = project_file_path(Path::new("/some/dir"));
        assert_eq!(path, PathBuf::from("/some/dir/dbloada.yaml"));
    }

    #[test]
    fn project_file_path_with_trailing_slash() {
        let path = project_file_path(Path::new("/some/dir/"));
        assert_eq!(path, PathBuf::from("/some/dir/dbloada.yaml"));
    }

    #[tokio::test]
    async fn load_returns_error_for_nonexistent_directory() {
        use crate::components::test_helpers::TestLogger;
        use crate::components::project_io::YamlProjectIO;
        use crate::components::project_serialization::YamlProjectSerialization;
        use crate::components::test_helpers::InMemoryFileSystem;
        use std::sync::Arc;
        use tokio::sync::Mutex;
        use std::collections::HashMap;

        let store = Arc::new(Mutex::new(HashMap::new()));
        let file_system = Box::new(InMemoryFileSystem::new(store));
        let serialization = Box::new(YamlProjectSerialization::new(Box::new(TestLogger)));
        let project_io = Box::new(YamlProjectIO::new(
            Box::new(TestLogger),
            file_system,
            serialization,
        ));
        let loader = LoadImpl::new(Box::new(TestLogger), project_io, vec![]);

        let result = loader.load(Path::new("/nonexistent/dir")).await;
        assert!(matches!(result, Err(LoadError::DirectoryNotFound(_))));
    }

    #[tokio::test]
    async fn load_returns_project_and_tables_for_valid_project() {
        use crate::components::csv_parser::CsvParserImpl;
        use crate::components::file_system::DiskFileSystem;
        use crate::components::project_io::YamlProjectIO;
        use crate::components::project_serialization::YamlProjectSerialization;
        use crate::components::table_reader::CsvTableReader;
        use crate::components::test_helpers::TestLogger;
        use crate::models::{
            ColumnIdentifier, ColumnSpec, ColumnType, FileSourceSpec, Project, ProjectSpec, SourceSpec, TableSpec,
        };

        let tmp = tempfile::tempdir().unwrap();
        let data_dir = tmp.path().join("data");
        tokio::fs::create_dir_all(&data_dir).await.unwrap();
        tokio::fs::write(data_dir.join("cities.csv"), "Name,Country\nLondon,UK\n").await.unwrap();

        let project = Project {
            name: "test".to_string(),
            api_version: "project.dbloada.io/v1".to_string(),
            spec: ProjectSpec {
                tables: vec![TableSpec {
                    name: "city".to_string(),
                    description: String::new(),
                    has_header: true,
                    source: SourceSpec::File(FileSourceSpec {
                        filename: "data/cities.csv".to_string(),
                        character_encoding: "utf-8".to_string(),
                    }),
                    columns: vec![
                        ColumnSpec {
                            name: "name".to_string(),
                            description: String::new(),
                            column_identifier: ColumnIdentifier::Name("Name".to_string()),
                            column_type: ColumnType::String,
                        },
                        ColumnSpec {
                            name: "country".to_string(),
                            description: String::new(),
                            column_identifier: ColumnIdentifier::Name("Country".to_string()),
                            column_type: ColumnType::String,
                        },
                    ],
                    relationships: vec![],
                }],
            },
        };

        let fs_for_io = Box::new(DiskFileSystem::new(Box::new(TestLogger)));
        let serialization = Box::new(YamlProjectSerialization::new(Box::new(TestLogger)));
        let project_io = Box::new(YamlProjectIO::new(Box::new(TestLogger), fs_for_io, serialization));
        project_io
            .save(&project, &tmp.path().join(DBLOADA_PROJECT_FILENAME))
            .await
            .unwrap();

        let loader = LoadImpl::new(
            Box::new(TestLogger),
            Box::new(YamlProjectIO::new(
                Box::new(TestLogger),
                Box::new(DiskFileSystem::new(Box::new(TestLogger))),
                Box::new(YamlProjectSerialization::new(Box::new(TestLogger))),
            )),
            vec![Box::new(CsvTableReader::new(
                Box::new(TestLogger),
                Box::new(DiskFileSystem::new(Box::new(TestLogger))),
                Box::new(CsvParserImpl::new(Box::new(TestLogger))),
            ))],
        );

        let loaded = loader.load(tmp.path()).await.unwrap();
        assert_eq!(loaded.project.name, "test");
        assert_eq!(loaded.tables.len(), 1);
        assert_eq!(loaded.tables[0].name, "city");
        assert_eq!(loaded.tables[0].num_rows(), 1);
    }

    #[tokio::test]
    async fn load_returns_table_reader_error_when_table_source_is_missing() {
        use crate::components::csv_parser::CsvParserImpl;
        use crate::components::file_system::DiskFileSystem;
        use crate::components::project_io::YamlProjectIO;
        use crate::components::project_serialization::YamlProjectSerialization;
        use crate::components::table_reader::CsvTableReader;
        use crate::components::test_helpers::TestLogger;
        use crate::models::{
            ColumnIdentifier, ColumnSpec, ColumnType, FileSourceSpec, Project, ProjectSpec, SourceSpec, TableSpec,
        };

        let tmp = tempfile::tempdir().unwrap();
        let project = Project {
            name: "test".to_string(),
            api_version: "project.dbloada.io/v1".to_string(),
            spec: ProjectSpec {
                tables: vec![TableSpec {
                    name: "city".to_string(),
                    description: String::new(),
                    has_header: true,
                    source: SourceSpec::File(FileSourceSpec {
                        filename: "data/missing.csv".to_string(),
                        character_encoding: "utf-8".to_string(),
                    }),
                    columns: vec![
                        ColumnSpec {
                            name: "name".to_string(),
                            description: String::new(),
                            column_identifier: ColumnIdentifier::Name("Name".to_string()),
                            column_type: ColumnType::String,
                        },
                    ],
                    relationships: vec![],
                }],
            },
        };

        let fs_for_io = Box::new(DiskFileSystem::new(Box::new(TestLogger)));
        let serialization = Box::new(YamlProjectSerialization::new(Box::new(TestLogger)));
        let project_io = Box::new(YamlProjectIO::new(Box::new(TestLogger), fs_for_io, serialization));
        project_io
            .save(&project, &tmp.path().join(DBLOADA_PROJECT_FILENAME))
            .await
            .unwrap();

        let loader = LoadImpl::new(
            Box::new(TestLogger),
            Box::new(YamlProjectIO::new(
                Box::new(TestLogger),
                Box::new(DiskFileSystem::new(Box::new(TestLogger))),
                Box::new(YamlProjectSerialization::new(Box::new(TestLogger))),
            )),
            vec![Box::new(CsvTableReader::new(
                Box::new(TestLogger),
                Box::new(DiskFileSystem::new(Box::new(TestLogger))),
                Box::new(CsvParserImpl::new(Box::new(TestLogger))),
            ))],
        );

        let err = loader.load(tmp.path()).await.unwrap_err();
        assert!(matches!(err, LoadError::TableReaderError(_)));
    }
}
