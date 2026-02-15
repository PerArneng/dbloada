use std::fmt::Write;

#[derive(Debug)]
pub struct Table {
    pub name: String,
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

impl Table {
    pub fn new(name: String, columns: Vec<String>, rows: Vec<Vec<String>>) -> Self {
        Table { name, columns, rows }
    }

    pub fn headers(&self) -> &[String] {
        &self.columns
    }

    pub fn num_rows(&self) -> usize {
        self.rows.len()
    }

    pub fn num_columns(&self) -> usize {
        self.columns.len()
    }

    pub fn row(&self, index: usize) -> Option<&[String]> {
        self.rows.get(index).map(|r| r.as_slice())
    }

    pub fn cell(&self, row: usize, col: usize) -> Option<&str> {
        self.rows.get(row).and_then(|r| r.get(col)).map(|s| s.as_str())
    }
}

pub fn table_to_string(table: &Table) -> String {
    let col_count = table.num_columns();
    let mut widths: Vec<usize> = table.columns.iter().map(|c| c.len()).collect();

    for row in &table.rows {
        for (i, val) in row.iter().enumerate() {
            if i < col_count {
                widths[i] = widths[i].max(val.len());
            }
        }
    }

    let mut out = String::new();
    let _ = writeln!(
        out,
        "Table: {} ({} rows, {} columns)",
        table.name,
        table.num_rows(),
        col_count,
    );

    let separator: String = widths.iter().map(|w| "-".repeat(w + 2)).collect::<Vec<_>>().join("+");
    let separator = format!("+{}+", separator);

    let _ = writeln!(out, "{}", separator);

    let header: String = widths
        .iter()
        .enumerate()
        .map(|(i, w)| format!(" {:width$} ", table.columns[i], width = w))
        .collect::<Vec<_>>()
        .join("|");
    let _ = writeln!(out, "|{}|", header);
    let _ = writeln!(out, "{}", separator);

    for row in &table.rows {
        let line: String = widths
            .iter()
            .enumerate()
            .map(|(i, w)| {
                let val = row.get(i).map(|s| s.as_str()).unwrap_or("");
                format!(" {:width$} ", val, width = w)
            })
            .collect::<Vec<_>>()
            .join("|");
        let _ = writeln!(out, "|{}|", line);
    }

    let _ = writeln!(out, "{}", separator);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_table_stores_fields() {
        let table = Table::new(
            "test".to_string(),
            vec!["a".to_string(), "b".to_string()],
            vec![vec!["1".to_string(), "2".to_string()]],
        );
        assert_eq!(table.name, "test");
        assert_eq!(table.headers(), &["a", "b"]);
        assert_eq!(table.num_rows(), 1);
        assert_eq!(table.num_columns(), 2);
    }

    #[test]
    fn row_returns_correct_row() {
        let table = Table::new(
            "t".to_string(),
            vec!["x".to_string()],
            vec![vec!["v0".to_string()], vec!["v1".to_string()]],
        );
        assert_eq!(table.row(0), Some(vec!["v0".to_string()].as_slice()));
        assert_eq!(table.row(1), Some(vec!["v1".to_string()].as_slice()));
        assert_eq!(table.row(2), None);
    }

    #[test]
    fn cell_returns_correct_value() {
        let table = Table::new(
            "t".to_string(),
            vec!["a".to_string(), "b".to_string()],
            vec![vec!["r0c0".to_string(), "r0c1".to_string()]],
        );
        assert_eq!(table.cell(0, 0), Some("r0c0"));
        assert_eq!(table.cell(0, 1), Some("r0c1"));
        assert_eq!(table.cell(1, 0), None);
        assert_eq!(table.cell(0, 2), None);
    }

    #[test]
    fn table_to_string_includes_summary() {
        let table = Table::new(
            "users".to_string(),
            vec!["name".to_string(), "age".to_string()],
            vec![vec!["Alice".to_string(), "30".to_string()]],
        );
        let output = table_to_string(&table);
        assert!(output.contains("Table: users (1 rows, 2 columns)"));
    }

    #[test]
    fn table_to_string_formats_borders() {
        let table = Table::new(
            "t".to_string(),
            vec!["a".to_string()],
            vec![vec!["x".to_string()]],
        );
        let output = table_to_string(&table);
        let lines: Vec<&str> = output.lines().collect();
        // summary, separator, header, separator, data, separator
        assert_eq!(lines.len(), 6);
        assert!(lines[1].starts_with('+'));
        assert!(lines[1].ends_with('+'));
        assert!(lines[3].starts_with('+'));
        assert!(lines[5].starts_with('+'));
    }

    #[test]
    fn table_to_string_aligns_columns() {
        let table = Table::new(
            "t".to_string(),
            vec!["name".to_string(), "id".to_string()],
            vec![
                vec!["Alice".to_string(), "1".to_string()],
                vec!["Bob".to_string(), "22".to_string()],
            ],
        );
        let output = table_to_string(&table);
        assert!(output.contains("| Alice | 1  |"));
        assert!(output.contains("| Bob   | 22 |"));
    }

    #[test]
    fn table_to_string_empty_table() {
        let table = Table::new(
            "empty".to_string(),
            vec!["col".to_string()],
            vec![],
        );
        let output = table_to_string(&table);
        assert!(output.contains("Table: empty (0 rows, 1 columns)"));
        let lines: Vec<&str> = output.lines().collect();
        // summary, separator, header, separator, separator (no data rows)
        assert_eq!(lines.len(), 5);
    }
}
