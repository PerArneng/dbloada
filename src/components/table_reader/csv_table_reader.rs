use std::collections::HashMap;
use std::path::Path;
use async_trait::async_trait;
use crate::models::{ColumnIdentifier, Table, TableSpec};
use crate::traits::{Logger, FileSystem};
use crate::traits::table_reader::{TableReader, TableReaderError};

pub struct CsvTableReader {
    logger: Box<dyn Logger>,
    file_system: Box<dyn FileSystem>,
}

impl CsvTableReader {
    pub fn new(logger: Box<dyn Logger>, file_system: Box<dyn FileSystem>) -> Self {
        CsvTableReader { logger, file_system }
    }
}

fn strip_csv_field(field: &str) -> String {
    let trimmed = field.trim();
    trimmed
        .strip_prefix('"')
        .and_then(|s| s.strip_suffix('"'))
        .unwrap_or(trimmed)
        .to_string()
}

fn resolve_column_indices(
    table: &TableSpec,
    header_map: &Option<HashMap<String, usize>>,
) -> Result<Vec<usize>, TableReaderError> {
    let mut indices = Vec::with_capacity(table.columns.len());
    for col in &table.columns {
        let idx = match &col.column_identifier {
            ColumnIdentifier::Index(i) => *i as usize,
            ColumnIdentifier::Name(name) => {
                let map = header_map.as_ref().ok_or_else(|| TableReaderError::ReadError {
                    table_name: table.name.clone(),
                    message: format!(
                        "column '{}' uses name identifier '{}' but has_header is false",
                        col.name, name
                    ),
                })?;
                *map.get(name).ok_or_else(|| TableReaderError::ReadError {
                    table_name: table.name.clone(),
                    message: format!(
                        "column '{}' references header '{}' which was not found in CSV headers",
                        col.name, name
                    ),
                })?
            }
        };
        indices.push(idx);
    }
    Ok(indices)
}

fn extract_row(record: &csv::StringRecord, indices: &[usize]) -> Vec<String> {
    indices
        .iter()
        .map(|&i| strip_csv_field(record.get(i).unwrap_or("")))
        .collect()
}

#[async_trait]
impl TableReader for CsvTableReader {
    fn name(&self) -> &str {
        "csv"
    }

    fn can_read(&self, table: &TableSpec) -> bool {
        table.source.filename.to_lowercase().ends_with(".csv")
    }

    async fn read_table(&self, table: &TableSpec, project_dir: &Path) -> Result<Table, TableReaderError> {
        let path = project_dir.join(&table.source.filename);
        self.logger.debug(&format!("reading CSV file: {}", path.display())).await;
        self.logger.debug(&format!("has_header: {}", table.has_header)).await;

        let content = self.file_system.load(&path).await?;

        let mut reader = csv::ReaderBuilder::new()
            .has_headers(table.has_header)
            .trim(csv::Trim::All)
            .from_reader(content.as_bytes());

        let header_map = if table.has_header {
            let headers = reader.headers().map_err(|e| TableReaderError::ReadError {
                table_name: table.name.clone(),
                message: format!("failed to parse CSV headers: {}", e),
            })?;
            let map: HashMap<String, usize> = headers
                .iter()
                .enumerate()
                .map(|(i, h)| (strip_csv_field(h), i))
                .collect();
            self.logger.debug(&format!("CSV headers: {:?}", map)).await;
            Some(map)
        } else {
            None
        };

        let indices = resolve_column_indices(table, &header_map)?;
        self.logger.debug(&format!(
            "column mapping: {:?}",
            table.columns.iter().map(|c| &c.name).zip(indices.iter()).collect::<Vec<_>>()
        )).await;

        let mut rows = Vec::new();
        for result in reader.records() {
            let record = result.map_err(|e| TableReaderError::ReadError {
                table_name: table.name.clone(),
                message: format!("failed to parse CSV record: {}", e),
            })?;
            rows.push(extract_row(&record, &indices));
        }

        let column_names: Vec<String> = table.columns.iter().map(|c| c.name.clone()).collect();

        self.logger.info(&format!(
            "read table '{}' using reader '{}': {} rows, {} columns",
            table.name,
            self.name(),
            rows.len(),
            column_names.len(),
        )).await;

        Ok(Table::new(table.name.clone(), column_names, rows))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ColumnSpec, ColumnType, SourceSpec};
    use crate::components::test_helpers::{TestLogger, InMemoryFileSystem};
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
        )
    }

    fn table_spec_with_header(name: &str, filename: &str, columns: Vec<ColumnSpec>) -> TableSpec {
        TableSpec {
            name: name.to_string(),
            description: String::new(),
            has_header: true,
            source: SourceSpec {
                filename: filename.to_string(),
                character_encoding: "utf-8".to_string(),
            },
            columns,
            relationships: vec![],
        }
    }

    fn table_spec_no_header(name: &str, filename: &str, columns: Vec<ColumnSpec>) -> TableSpec {
        TableSpec {
            name: name.to_string(),
            description: String::new(),
            has_header: false,
            source: SourceSpec {
                filename: filename.to_string(),
                character_encoding: "utf-8".to_string(),
            },
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
    fn resolve_column_indices_by_index() {
        let spec = table_spec_no_header("t", "f.csv", vec![
            col_by_index("a", 2),
            col_by_index("b", 0),
        ]);
        let indices = resolve_column_indices(&spec, &None).unwrap();
        assert_eq!(indices, vec![2, 0]);
    }

    #[test]
    fn resolve_column_indices_by_name() {
        let spec = table_spec_with_header("t", "f.csv", vec![
            col_by_name("col_b", "B"),
            col_by_name("col_a", "A"),
        ]);
        let mut map = HashMap::new();
        map.insert("A".to_string(), 0);
        map.insert("B".to_string(), 1);
        let indices = resolve_column_indices(&spec, &Some(map)).unwrap();
        assert_eq!(indices, vec![1, 0]);
    }
}
