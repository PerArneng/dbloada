use std::path::Path;
use async_trait::async_trait;
use crate::models::{SourceSpec, Table, TableSpec};
use crate::traits::{Logger, CsvParser};
use crate::traits::table_reader::{TableReader, TableReaderError};

pub struct CmdCsvTableReader {
    logger: Box<dyn Logger>,
    csv_parser: Box<dyn CsvParser>,
}

impl CmdCsvTableReader {
    pub fn new(logger: Box<dyn Logger>, csv_parser: Box<dyn CsvParser>) -> Self {
        CmdCsvTableReader { logger, csv_parser }
    }
}

pub fn substitute_temp_path(args: &[String], path: &str) -> Vec<String> {
    args.iter()
        .map(|a| a.replace("$TEMP_CSV_PATH", path))
        .collect()
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
impl TableReader for CmdCsvTableReader {
    fn name(&self) -> &str {
        "cmd_csv"
    }

    fn can_read(&self, table: &TableSpec) -> bool {
        matches!(&table.source, SourceSpec::Cmd(_))
    }

    async fn read_table(&self, table: &TableSpec, project_dir: &Path) -> Result<Table, TableReaderError> {
        let cmd_source = match &table.source {
            SourceSpec::Cmd(cs) => cs,
            SourceSpec::File(_) => {
                return Err(TableReaderError::ReadError {
                    table_name: table.name.clone(),
                    message: "CmdCsvTableReader does not support file sources".to_string(),
                });
            }
        };

        let content = if cmd_source.stdout {
            self.logger.info(&format!(
                "running command (stdout mode): {} {:?}",
                cmd_source.command, cmd_source.args
            )).await;

            let output = tokio::process::Command::new(&cmd_source.command)
                .args(&cmd_source.args)
                .current_dir(project_dir)
                .output()
                .await
                .map_err(|e| TableReaderError::ReadError {
                    table_name: table.name.clone(),
                    message: format!("failed to execute command '{}': {}", cmd_source.command, e),
                })?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(TableReaderError::ReadError {
                    table_name: table.name.clone(),
                    message: format!(
                        "command '{}' exited with status {}: {}",
                        cmd_source.command,
                        output.status,
                        stderr.trim()
                    ),
                });
            }

            decode_bytes(&output.stdout, &cmd_source.character_encoding).map_err(|msg| {
                TableReaderError::ReadError {
                    table_name: table.name.clone(),
                    message: msg,
                }
            })?
        } else {
            let temp_dir = std::env::temp_dir();
            let temp_filename = format!("dbloada-{}.csv", uuid::Uuid::new_v4());
            let temp_path = temp_dir.join(&temp_filename);
            let temp_path_str = temp_path.display().to_string();

            let args = substitute_temp_path(&cmd_source.args, &temp_path_str);

            self.logger.info(&format!(
                "running command (temp file mode): {} {:?} -> {}",
                cmd_source.command, args, temp_path_str
            )).await;

            let status = tokio::process::Command::new(&cmd_source.command)
                .args(&args)
                .current_dir(project_dir)
                .stdout(std::process::Stdio::inherit())
                .stderr(std::process::Stdio::inherit())
                .status()
                .await
                .map_err(|e| TableReaderError::ReadError {
                    table_name: table.name.clone(),
                    message: format!("failed to execute command '{}': {}", cmd_source.command, e),
                })?;

            if !status.success() {
                return Err(TableReaderError::ReadError {
                    table_name: table.name.clone(),
                    message: format!(
                        "command '{}' exited with status {}",
                        cmd_source.command, status
                    ),
                });
            }

            let bytes = tokio::fs::read(&temp_path).await.map_err(|e| {
                TableReaderError::ReadError {
                    table_name: table.name.clone(),
                    message: format!("failed to read temp file '{}': {}", temp_path_str, e),
                }
            })?;

            let _ = tokio::fs::remove_file(&temp_path).await;

            decode_bytes(&bytes, &cmd_source.character_encoding).map_err(|msg| {
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
    use crate::models::CmdSourceSpec;

    #[test]
    fn substitute_temp_path_replaces_placeholder() {
        let args = vec![
            "script.sh".to_string(),
            "$TEMP_CSV_PATH".to_string(),
        ];
        let result = substitute_temp_path(&args, "/tmp/dbloada-123.csv");
        assert_eq!(result, vec!["script.sh", "/tmp/dbloada-123.csv"]);
    }

    #[test]
    fn substitute_temp_path_no_placeholder() {
        let args = vec!["script.sh".to_string(), "--flag".to_string()];
        let result = substitute_temp_path(&args, "/tmp/dbloada-123.csv");
        assert_eq!(result, vec!["script.sh", "--flag"]);
    }

    #[test]
    fn substitute_temp_path_empty_args() {
        let args: Vec<String> = vec![];
        let result = substitute_temp_path(&args, "/tmp/dbloada-123.csv");
        assert!(result.is_empty());
    }

    #[test]
    fn can_read_cmd_source() {
        let reader = CmdCsvTableReader::new(
            Box::new(crate::components::test_helpers::TestLogger),
            Box::new(crate::components::csv_parser::CsvParserImpl::new(
                Box::new(crate::components::test_helpers::TestLogger),
            )),
        );
        let spec = TableSpec {
            name: "t".to_string(),
            description: String::new(),
            has_header: true,
            source: SourceSpec::Cmd(CmdSourceSpec {
                command: "bash".to_string(),
                args: vec![],
                stdout: true,
                character_encoding: "utf-8".to_string(),
            }),
            columns: vec![],
            relationships: vec![],
        };
        assert!(reader.can_read(&spec));
    }

    #[test]
    fn cannot_read_file_source() {
        let reader = CmdCsvTableReader::new(
            Box::new(crate::components::test_helpers::TestLogger),
            Box::new(crate::components::csv_parser::CsvParserImpl::new(
                Box::new(crate::components::test_helpers::TestLogger),
            )),
        );
        let spec = TableSpec {
            name: "t".to_string(),
            description: String::new(),
            has_header: true,
            source: SourceSpec::File(crate::models::FileSourceSpec {
                filename: "data/test.csv".to_string(),
                character_encoding: "utf-8".to_string(),
            }),
            columns: vec![],
            relationships: vec![],
        };
        assert!(!reader.can_read(&spec));
    }
}
