use std::{fmt::Display, fs::{File, OpenOptions}, io::{self, Write}, sync::{Arc, Mutex, OnceLock, RwLock}};
use chrono::Local;
use colored::Colorize;
use serde::Serialize;

#[derive(Debug, Clone)]
pub struct LoggerOptions {
    pub verbose: bool,
    pub log_to_stdout: bool,
    pub log_to_stderr: bool,
    pub color_output: bool,
    pub show_debug: bool,
    pub show_info: bool,
    pub max_logs: usize
}

impl Default for LoggerOptions {
    fn default() -> Self {
        Self {
            verbose: true,
            log_to_stdout: true,
            log_to_stderr: true,
            color_output: true,
            show_debug: true,
            show_info: true,
            max_logs: 500
        }
    }
}


#[derive(Debug, PartialEq, Eq, Serialize, Clone)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Success,
    Critical
}

#[derive(Serialize, Debug, Clone)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: LogLevel,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub message: String
}

impl LogEntry {
    pub fn new(timestamp: String, level: LogLevel, message: &str, file: Option<String>, line: Option<u32>) -> Self {
        Self {
            timestamp,
            level,
            line,
            file,
            message: message.to_string()
        }
    }
}

impl Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Error => write!(f, "ERROR"),
            Self::Debug => write!(f, "DEBUG"),
            Self::Info => write!(f, "INFO"),
            Self::Success => write!(f, "SUCCESS"),
            Self::Warn => write!(f, "WARN"),
            Self::Critical => write!(f, "CRITICAL")
        }
    }
}
#[derive(Debug)]
pub struct Logger {
    log_file: Option<Arc<Mutex<File>>>,
    logs: Arc<RwLock<Vec<LogEntry>>>,
    options: LoggerOptions
}


lazy_static::lazy_static! {
    static ref LOGGER_INSTANCE: OnceLock<RwLock<Logger>> = OnceLock::new();
}

impl Logger {
    pub fn get_logs() -> io::Result<Vec<LogEntry>> {
        let logger = LOGGER_INSTANCE
            .get()
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Logger not initialized"))?;
        let logs_guard = logger.read().map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to acquire logger read lock"))?;
        let logs = logs_guard.logs.read().map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to acquire logs read lock"))?;
        Ok(logs.clone())
    }



    pub fn init(log_file: Option<impl Into<String>>, options: LoggerOptions) -> io::Result<()> {
        let logfile = if let Some(log_file) = log_file {
            let log_file = log_file.into();
            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(log_file)
                .ok()
                .map(|f| Arc::new(Mutex::new(f)));
            file
        } else {
            None
        };
        let logger = Logger {
            log_file: logfile,
            logs: Arc::new(RwLock::new(Vec::with_capacity(options.max_logs))),
            options,
        };

        LOGGER_INSTANCE.set(RwLock::new(logger)).map_err(|_| io::Error::new(std::io::ErrorKind::Other, "Failed to set logger instance!"))?;
        Ok(())
    }

    fn log(&self, level: LogLevel, message: &str, file: &str, line: u32) {
        if !self.options.verbose {
            return;
        }

        if (level == LogLevel::Debug && !self.options.show_debug) ||
            (level == LogLevel::Info && !self.options.show_info) {
            return;
        }

        let timestamp = Local::now().format("%d:%m %H:%M").to_string();
        let mut log_lock = self.logs.write().unwrap();

        if log_lock.len() >= self.options.max_logs {
            log_lock.pop();
        }

        let fmt = match level {
            LogLevel::Debug | LogLevel::Error | LogLevel::Warn | LogLevel::Critical => {
                log_lock.push(LogEntry::new(timestamp.clone(), level.clone(), message, Some(file.to_string()), Some(line)));
                format!("[{}] [{}][{}:{}]: {}", timestamp, level, file, line, message)
            }
            _ => {
                log_lock.push(LogEntry::new(timestamp.clone(), level.clone(), message, None, None));
                format!("[{}] [{}]: {}", timestamp, level, message)
            }
        };

        if self.options.log_to_stdout || self.options.log_to_stderr {
            let log_entry = if self.options.color_output {
                match level {
                    LogLevel::Debug => fmt.yellow().on_black().to_string(),
                    LogLevel::Error => fmt.bright_red().bold().to_string(),
                    LogLevel::Warn => fmt.yellow().to_string(),
                    LogLevel::Info => fmt.cyan().to_string(),
                    LogLevel::Success => fmt.green().to_string(),
                    LogLevel::Critical => fmt.bright_red().bold().on_bright_cyan().to_string(),
                }
            } else {
                fmt.clone()
            };

            match level {
                LogLevel::Error | LogLevel::Critical if self.options.log_to_stderr => {
                    eprintln!("{}", log_entry);
                }
                _ if self.options.log_to_stdout => {
                    println!("{}", log_entry);
                }
                _ => {}
            }
        }

        if let Some(file) = &self.log_file {
            let mut file = file.lock().unwrap();
            writeln!(file, "{}", fmt).unwrap();
        }
    }

    pub fn log_err(message: &str, file: &str, line: u32) {
        LOGGER_INSTANCE.get().unwrap().write().unwrap().log(LogLevel::Error, message, file, line);
    }

    pub fn log_success(message: &str, file: &str, line: u32) {
        LOGGER_INSTANCE.get().unwrap().write().unwrap().log(LogLevel::Success, message, file, line);
    }

    pub fn log_info(message: &str, file: &str, line: u32) {
        LOGGER_INSTANCE.get().unwrap().write().unwrap().log(LogLevel::Info, message, file, line);
    }

    pub fn log_debug(message: &str, file: &str, line: u32) {
        LOGGER_INSTANCE.get().unwrap().write().unwrap().log(LogLevel::Debug, message, file, line);
    }

    pub fn log_warn(message: &str, file: &str, line: u32) {
        LOGGER_INSTANCE.get().unwrap().write().unwrap().log(LogLevel::Warn, message, file, line);
    }
    pub fn log_critical(message: &str, file: &str, line: u32) {
        LOGGER_INSTANCE.get().unwrap().write().unwrap().log(LogLevel::Critical, message, file, line);
    }

}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        $crate::logger::log::Logger::log_info(&format!($($arg)*), file!(), line!());
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        $crate::logger::log::Logger::log_warn(&format!($($arg)*), file!(), line!());
    };
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        $crate::logger::log::Logger::log_debug(&format!($($arg)*), file!(), line!())
    };
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        $crate::logger::log::Logger::log_err(&format!($($arg)*), file!(), line!());
    };
}

#[macro_export]
macro_rules! success {
    ($($arg:tt)*) => {
        $crate::logger::log::Logger::log_success(&format!($($arg)*), file!(), line!());
    };
}

#[macro_export]
macro_rules! critical {
    ($($arg:tt)*) => {
        $crate::logger::log::Logger::log_critical(&format!($($arg)*), file!(), line!());
    };
}