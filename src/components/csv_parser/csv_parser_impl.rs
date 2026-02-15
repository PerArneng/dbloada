use std::collections::HashMap;
use async_trait::async_trait;
use crate::models::{ColumnIdentifier, Table, TableSpec};
use crate::traits::{Logger, CsvParser, CsvParserError};

pub struct CsvParserImpl {
    logger: Box<dyn Logger>,
}

impl CsvParserImpl {
    pub fn new(logger: Box<dyn Logger>) -> Self {
        CsvParserImpl { logger }
    }
}

pub fn strip_csv_field(field: &str) -> String {
    let trimmed = field.trim();
    trimmed
        .strip_prefix('"')
        .and_then(|s| s.strip_suffix('"'))
        .unwrap_or(trimmed)
        .to_string()
}

pub fn resolve_column_indices(
    table: &TableSpec,
    header_map: &Option<HashMap<String, usize>>,
) -> Result<Vec<usize>, CsvParserError> {
    let mut indices = Vec::with_capacity(table.columns.len());
    for col in &table.columns {
        let idx = match &col.column_identifier {
            ColumnIdentifier::Index(i) => *i as usize,
            ColumnIdentifier::Name(name) => {
                let map = header_map.as_ref().ok_or_else(|| CsvParserError::ParseError {
                    table_name: table.name.clone(),
                    message: format!(
                        "column '{}' uses name identifier '{}' but has_header is false",
                        col.name, name
                    ),
                })?;
                *map.get(name).ok_or_else(|| CsvParserError::ParseError {
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

pub fn extract_row(record: &csv::StringRecord, indices: &[usize]) -> Vec<String> {
    indices
        .iter()
        .map(|&i| strip_csv_field(record.get(i).unwrap_or("")))
        .collect()
}

#[async_trait]
impl CsvParser for CsvParserImpl {
    async fn parse(&self, content: &str, table: &TableSpec) -> Result<Table, CsvParserError> {
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(table.has_header)
            .trim(csv::Trim::All)
            .from_reader(content.as_bytes());

        let header_map = if table.has_header {
            let headers = reader.headers().map_err(|e| CsvParserError::ParseError {
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
            let record = result.map_err(|e| CsvParserError::ParseError {
                table_name: table.name.clone(),
                message: format!("failed to parse CSV record: {}", e),
            })?;
            rows.push(extract_row(&record, &indices));
        }

        let column_names: Vec<String> = table.columns.iter().map(|c| c.name.clone()).collect();

        Ok(Table::new(table.name.clone(), column_names, rows))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ColumnSpec, ColumnType, SourceSpec, FileSourceSpec};
    use crate::components::test_helpers::TestLogger;

    fn file_source() -> SourceSpec {
        SourceSpec::File(FileSourceSpec {
            filename: "test.csv".to_string(),
            character_encoding: "utf-8".to_string(),
        })
    }

    fn table_spec_with_header(name: &str, columns: Vec<ColumnSpec>) -> TableSpec {
        TableSpec {
            name: name.to_string(),
            description: String::new(),
            has_header: true,
            source: file_source(),
            columns,
            relationships: vec![],
        }
    }

    fn table_spec_no_header(name: &str, columns: Vec<ColumnSpec>) -> TableSpec {
        TableSpec {
            name: name.to_string(),
            description: String::new(),
            has_header: false,
            source: file_source(),
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
    fn strip_csv_field_removes_quotes() {
        assert_eq!(strip_csv_field("\"hello\""), "hello");
    }

    #[test]
    fn strip_csv_field_trims_whitespace() {
        assert_eq!(strip_csv_field("  hello  "), "hello");
    }

    #[test]
    fn strip_csv_field_no_quotes() {
        assert_eq!(strip_csv_field("hello"), "hello");
    }

    #[test]
    fn resolve_column_indices_by_index() {
        let spec = table_spec_no_header("t", vec![
            col_by_index("a", 2),
            col_by_index("b", 0),
        ]);
        let indices = resolve_column_indices(&spec, &None).unwrap();
        assert_eq!(indices, vec![2, 0]);
    }

    #[test]
    fn resolve_column_indices_by_name() {
        let spec = table_spec_with_header("t", vec![
            col_by_name("col_b", "B"),
            col_by_name("col_a", "A"),
        ]);
        let mut map = HashMap::new();
        map.insert("A".to_string(), 0);
        map.insert("B".to_string(), 1);
        let indices = resolve_column_indices(&spec, &Some(map)).unwrap();
        assert_eq!(indices, vec![1, 0]);
    }

    #[test]
    fn resolve_column_indices_name_without_header_errors() {
        let spec = table_spec_no_header("t", vec![
            col_by_name("col", "A"),
        ]);
        let result = resolve_column_indices(&spec, &None);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn parse_with_headers() {
        let parser = CsvParserImpl::new(Box::new(TestLogger));
        let content = "Name,Country\nLondon,UK\nBerlin,Germany\n";
        let spec = table_spec_with_header("city", vec![
            col_by_name("name", "Name"),
            col_by_name("country", "Country"),
        ]);
        let table = parser.parse(content, &spec).await.unwrap();
        assert_eq!(table.name, "city");
        assert_eq!(table.num_rows(), 2);
        assert_eq!(table.cell(0, 0), Some("London"));
        assert_eq!(table.cell(1, 1), Some("Germany"));
    }

    #[tokio::test]
    async fn parse_without_headers() {
        let parser = CsvParserImpl::new(Box::new(TestLogger));
        let content = "\"United Kingdom\"\n\"Germany\"\n";
        let spec = table_spec_no_header("country", vec![
            col_by_index("name", 0),
        ]);
        let table = parser.parse(content, &spec).await.unwrap();
        assert_eq!(table.num_rows(), 2);
        assert_eq!(table.cell(0, 0), Some("United Kingdom"));
        assert_eq!(table.cell(1, 0), Some("Germany"));
    }

    #[tokio::test]
    async fn parse_reorders_columns() {
        let parser = CsvParserImpl::new(Box::new(TestLogger));
        let content = "A,B,C\n1,2,3\n";
        let spec = table_spec_with_header("t", vec![
            col_by_name("col_c", "C"),
            col_by_name("col_a", "A"),
        ]);
        let table = parser.parse(content, &spec).await.unwrap();
        assert_eq!(table.headers(), &["col_c", "col_a"]);
        assert_eq!(table.cell(0, 0), Some("3"));
        assert_eq!(table.cell(0, 1), Some("1"));
    }
}
