use chrono::{TimeZone, Utc};
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Copy)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

pub struct LogEntry {
    id: u64,
    level: LogLevel,
    message: String,
    timestamp_ms: u64,
}

impl fmt::Display for LogEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "({:>2}) [{}] {}: {}",
            self.id,
            self.level,
            self.timestamp_to_str(),
            self.message,
        )
    }
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            LogLevel::Trace => write!(f, "Trace"),
            LogLevel::Debug => write!(f, "Debug"),
            LogLevel::Info => write!(f, "Info"),
            LogLevel::Warn => write!(f, "Warn"),
            LogLevel::Error => write!(f, "Error"),
        }
    }
}

impl LogEntry {
    pub fn new(id: u64, level: LogLevel, message: String) -> LogEntry {
        LogEntry {
            id,
            level,
            message,
            timestamp_ms: Self::current_timestamp_ms(),
        }
    }

    fn current_timestamp_ms() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }

    fn timestamp_to_str(&self) -> String {
        let dt = Utc.timestamp_millis_opt(self.timestamp_ms as i64).unwrap();
        format!("{}", dt.format("%Y-%m-%d %H:%M:%S"))
    }

    pub fn level(&self) -> LogLevel {
        self.level
    }

    pub fn set_message(&mut self, message: String) {
        self.message = message;
    }

    pub fn message(&self) -> &str {
        self.message.as_str()
    }
}
