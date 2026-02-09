use async_trait::async_trait;
use tokio::io::{self, AsyncWriteExt};
use crate::traits::Logger;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum LogLevel {
    Error = 0,
    Warn = 1,
    Info = 2,
    Debug = 3,
    Trace = 4,
}

fn parse_log_level(s: &str) -> LogLevel {
    match s.to_lowercase().as_str() {
        "error" => LogLevel::Error,
        "warn" => LogLevel::Warn,
        "info" => LogLevel::Info,
        "debug" => LogLevel::Debug,
        "trace" => LogLevel::Trace,
        _ => LogLevel::Info,
    }
}

pub struct TokioLogger {
    level: LogLevel,
}

impl TokioLogger {
    pub fn new() -> Self {
        let level = std::env::var("RUST_LOG")
            .map(|s| parse_log_level(&s))
            .unwrap_or(LogLevel::Info);
        TokioLogger { level }
    }

    async fn log(&self, level: LogLevel, label: &str, msg: &str) {
        if level <= self.level {
            let line = format!("[{label}] {msg}\n");
            let _ = io::stdout().write_all(line.as_bytes()).await;
        }
    }
}

#[async_trait]
impl Logger for TokioLogger {
    async fn error(&self, msg: &str) {
        self.log(LogLevel::Error, "ERROR", msg).await;
    }

    async fn warn(&self, msg: &str) {
        self.log(LogLevel::Warn, "WARN", msg).await;
    }

    async fn info(&self, msg: &str) {
        self.log(LogLevel::Info, "INFO", msg).await;
    }

    async fn debug(&self, msg: &str) {
        self.log(LogLevel::Debug, "DEBUG", msg).await;
    }

    async fn trace(&self, msg: &str) {
        self.log(LogLevel::Trace, "TRACE", msg).await;
    }
}
