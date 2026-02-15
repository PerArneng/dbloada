use std::path::Path;
use async_trait::async_trait;
use crate::models::{SourceSpec, TableSpec};
use crate::traits::{Logger, FileSystem, CsvParser};
use crate::traits::table_reader::{TableReader, TableReaderError};
use crate::models::Table;

pub struct CsvTableReader {
    logger: Box<dyn Logger>,
    file_system: Box<dyn FileSystem>,
    csv_parser: Box<dyn CsvParser>,
}

impl CsvTableReader {
    pub fn new(
        logger: Box<dyn Logger>,
        file_system: Box<dyn FileSystem>,
        csv_parser: Box<dyn CsvParser>,
    ) -> Self {
        CsvTableReader { logger, file_system, csv_parser }
    }
}

fn decode_bytes(bytes: &[u8], encoding_label: &str) -> Result<String, String> {
    let encoding = encoding_rs::Encoding::for_label(encoding_label.as_bytes())
        .ok_or_else(|| format!("unsupported encoding: '{}'", encoding_label))?;
    let (cow, _, had_errors) = encoding.decode(bytes);
    if had_errors {
        return Err(format!("encoding errors while decoding as '{}'", encoding_label));
    }
    Ok(cow.into_owned())
}

#[async_trait]
impl TableReader for CsvTableReader {
    fn name(&self) -> &str {
        "csv"
    }

    fn can_read(&self, table: &TableSpec) -> bool {
        match &table.source {
            SourceSpec::File(fs) => fs.filename.to_lowercase().ends_with(".csv"),
            SourceSpec::Cmd(_) => false,
        }
    }

    async fn read_table(&self, table: &TableSpec, project_dir: &Path) -> Result<Table, TableReaderError> {
        let file_source = match &table.source {
            SourceSpec::File(fs) => fs,
            SourceSpec::Cmd(_) => {
                return Err(TableReaderError::ReadError {
                    table_name: table.name.clone(),
                    message: "CsvTableReader does not support command sources".to_string(),
                });
            }
        };

        let path = project_dir.join(&file_source.filename);
        self.logger.debug(&format!("reading CSV file: {}", path.display())).await;
        self.logger.debug(&format!("has_header: {}", table.has_header)).await;

        let encoding_lower = file_source.character_encoding.to_lowercase();
        let content = if encoding_lower == "utf-8" || encoding_lower == "utf8" {
            self.file_system.load(&path).await?
        } else {
            let bytes = self.file_system.load_bytes(&path).await?;
            decode_bytes(&bytes, &file_source.character_encoding).map_err(|msg| {
                TableReaderError::ReadError {
                    table_name: table.name.clone(),
                    message: msg,
                }
            })?
        };

        let result = self.csv_parser.parse(&content, table).await?;

        self.logger.info(&format!(
            "read table '{}' using reader '{}': {} rows, {} columns",
            table.name,
            self.name(),
            result.num_rows(),
            result.num_columns(),
        )).await;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ColumnSpec, ColumnIdentifier, ColumnType, FileSourceSpec};
    use crate::components::test_helpers::{TestLogger, InMemoryFileSystem};
    use crate::components::csv_parser::CsvParserImpl;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    fn make_reader(files: Vec<(&str, &str)>) -> CsvTableReader {
        let mut map = std::collections::HashMap::new();
        for (path, content) in files {
            map.insert(std::path::PathBuf::from(path), content.to_string());
        }
        let store = Arc::new(Mutex::new(map));
        CsvTableReader::new(
            Box::new(TestLogger),
            Box::new(InMemoryFileSystem::new(store)),
            Box::new(CsvParserImpl::new(Box::new(TestLogger))),
        )
    }

    fn file_source(filename: &str) -> SourceSpec {
        SourceSpec::File(FileSourceSpec {
            filename: filename.to_string(),
            character_encoding: "utf-8".to_string(),
        })
    }

    fn table_spec_with_header(name: &str, filename: &str, columns: Vec<ColumnSpec>) -> TableSpec {
        TableSpec {
            name: name.to_string(),
            description: String::new(),
            has_header: true,
            source: file_source(filename),
            columns,
            relationships: vec![],
        }
    }

    fn table_spec_no_header(name: &str, filename: &str, columns: Vec<ColumnSpec>) -> TableSpec {
        TableSpec {
            name: name.to_string(),
            description: String::new(),
            has_header: false,
            source: file_source(filename),
            columns,
            relationships: vec![],
        }
    }

    fn col_by_name(name: &str, header: &str) -> ColumnSpec {
        ColumnSpec {
            name: name.to_string(),
            description: String::new(),
            column_identifier: ColumnIdentifier::Name(header.to_string()),
            column_type: ColumnType::String,
        }
    }

    fn col_by_index(name: &str, index: u64) -> ColumnSpec {
        ColumnSpec {
            name: name.to_string(),
            description: String::new(),
            column_identifier: ColumnIdentifier::Index(index),
            column_type: ColumnType::String,
        }
    }

    #[test]
    fn can_read_csv_extension() {
        let reader = make_reader(vec![]);
        let spec = table_spec_with_header("t", "data/file.csv", vec![]);
        assert!(reader.can_read(&spec));
    }

    #[test]
    fn can_read_csv_case_insensitive() {
        let reader = make_reader(vec![]);
        let spec = table_spec_with_header("t", "data/file.CSV", vec![]);
        assert!(reader.can_read(&spec));
    }

    #[test]
    fn cannot_read_non_csv() {
        let reader = make_reader(vec![]);
        let spec = table_spec_with_header("t", "data/file.json", vec![]);
        assert!(!reader.can_read(&spec));
    }

    #[test]
    fn cannot_read_cmd_source() {
        let reader = make_reader(vec![]);
        let spec = TableSpec {
            name: "t".to_string(),
            description: String::new(),
            has_header: true,
            source: SourceSpec::Cmd(crate::models::CmdSourceSpec {
                command: "bash".to_string(),
                args: vec![],
                stdout: true,
                character_encoding: "utf-8".to_string(),
            }),
            columns: vec![],
            relationships: vec![],
        };
        assert!(!reader.can_read(&spec));
    }

    #[tokio::test]
    async fn read_table_with_headers_by_name() {
        let reader = make_reader(vec![
            ("/project/data/cities.csv", "Name,Country\nLondon,UK\nBerlin,Germany\n"),
        ]);
        let spec = table_spec_with_header("city", "data/cities.csv", vec![
            col_by_name("name", "Name"),
            col_by_name("country", "Country"),
        ]);
        let table = reader.read_table(&spec, Path::new("/project")).await.unwrap();
        assert_eq!(table.name, "city");
        assert_eq!(table.num_rows(), 2);
        assert_eq!(table.num_columns(), 2);
        assert_eq!(table.cell(0, 0), Some("London"));
        assert_eq!(table.cell(0, 1), Some("UK"));
        assert_eq!(table.cell(1, 0), Some("Berlin"));
        assert_eq!(table.cell(1, 1), Some("Germany"));
    }

    #[tokio::test]
    async fn read_table_without_headers_by_index() {
        let reader = make_reader(vec![
            ("/project/data/countries.csv", "\"United Kingdom\"\n\"Germany\"\n"),
        ]);
        let spec = table_spec_no_header("country", "data/countries.csv", vec![
            col_by_index("name", 0),
        ]);
        let table = reader.read_table(&spec, Path::new("/project")).await.unwrap();
        assert_eq!(table.num_rows(), 2);
        assert_eq!(table.cell(0, 0), Some("United Kingdom"));
        assert_eq!(table.cell(1, 0), Some("Germany"));
    }

    #[tokio::test]
    async fn read_table_reorders_columns() {
        let reader = make_reader(vec![
            ("/project/data/test.csv", "A,B,C\n1,2,3\n"),
        ]);
        let spec = table_spec_with_header("t", "data/test.csv", vec![
            col_by_name("col_c", "C"),
            col_by_name("col_a", "A"),
        ]);
        let table = reader.read_table(&spec, Path::new("/project")).await.unwrap();
        assert_eq!(table.headers(), &["col_c", "col_a"]);
        assert_eq!(table.cell(0, 0), Some("3"));
        assert_eq!(table.cell(0, 1), Some("1"));
    }

    #[tokio::test]
    async fn read_table_name_without_header_errors() {
        let reader = make_reader(vec![
            ("/project/data/test.csv", "a,b\n1,2\n"),
        ]);
        let spec = table_spec_no_header("t", "data/test.csv", vec![
            col_by_name("col", "a"),
        ]);
        let result = reader.read_table(&spec, Path::new("/project")).await;
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("has_header is false"), "error was: {}", err);
    }

    #[tokio::test]
    async fn read_table_missing_header_errors() {
        let reader = make_reader(vec![
            ("/project/data/test.csv", "A,B\n1,2\n"),
        ]);
        let spec = table_spec_with_header("t", "data/test.csv", vec![
            col_by_name("col", "NonExistent"),
        ]);
        let result = reader.read_table(&spec, Path::new("/project")).await;
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("not found in CSV headers"), "error was: {}", err);
    }

    #[tokio::test]
    async fn read_table_file_not_found_errors() {
        let reader = make_reader(vec![]);
        let spec = table_spec_with_header("t", "data/missing.csv", vec![]);
        let result = reader.read_table(&spec, Path::new("/project")).await;
        assert!(result.is_err());
    }

    #[test]
    fn decode_bytes_utf8() {
        let result = decode_bytes(b"hello", "utf-8").unwrap();
        assert_eq!(result, "hello");
    }

    #[test]
    fn decode_bytes_unknown_encoding_errors() {
        let result = decode_bytes(b"hello", "unknown-encoding");
        assert!(result.is_err());
    }
}
