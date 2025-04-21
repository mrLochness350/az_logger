use std::{fmt::Display, fs::{File, OpenOptions}, io::{self, Write}, sync::{Arc, Mutex, OnceLock, RwLock}};
use chrono::Local;
use colored::Colorize;
use serde::Serialize;

/// Configuration options for the global logger instance.
#[derive(Debug, Clone)]
pub struct LoggerOptions {
    /// Enables or disables logging output entirely.
    pub verbose: bool,
    /// If true, logs will be printed to stdout.
    pub log_to_stdout: bool,
    /// If true, errors and critical logs will be printed to stderr.
    pub log_to_stderr: bool,
    /// Enables or disables colored terminal output.
    pub color_output: bool,
    /// Whether debug-level logs should be emitted.
    pub show_debug: bool,
    /// Whether info-level logs should be emitted.
    pub show_info: bool,
    /// Maximum number of logs retained in memory.
    pub max_logs: usize,
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


/// Severity level of a log entry.
#[derive(Debug, PartialEq, Eq, Serialize, Clone)]
pub enum LogLevel {
    /// Used for unrecoverable fatal issues.
    Critical,
    /// Used for runtime errors or failures.
    Error,
    /// Used for warning conditions.
    Warn,
    /// Used for general informational messages.
    Info,
    /// Used for debugging-level diagnostics.
    Debug,
    /// Used to indicate successful operations.
    Success,
}

/// Represents a single log entry with metadata.
#[derive(Serialize, Debug, Clone)]
pub struct LogEntry {
    /// Timestamp in `dd:mm HH:MM` format.
    pub timestamp: String,
    /// Severity level of the log.
    pub level: LogLevel,
    /// Optional file path where the log originated.
    pub file: Option<String>,
    /// Optional line number in the source file.
    pub line: Option<u32>,
    /// Actual log message content.
    pub message: String,
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
/// A globally accessible, thread-safe logger.
///
/// `Logger` provides centralized logging functionality for applications,
/// including:
///
/// - Log message routing to `stdout`, `stderr`, and/or a file
/// - Runtime filtering based on log levels (`Info`, `Debug`, etc.)
/// - Optional colorized output via the `colored` crate
/// - In-memory log history up to a configurable limit (`max_logs`)
/// - Internal locking via `RwLock` and `Mutex` to ensure thread safety
///
/// This logger is initialized once per application using [`Logger::init`],
/// and subsequent logging is done via either its static methods
/// (`log_info`, `log_error`, etc.) or through logging macros like [`crate::info!`] and [`crate::error!`].
///
/// Internally, the logger stores a circular buffer of recent logs,
/// and can optionally persist log entries to disk.
///
/// # Example
///
/// ```
/// use az_logger::{Logger, LoggerOptions, info, error};
///
/// Logger::init(Some("output.log"), LoggerOptions::default()).unwrap();
///
/// info!("Application started");
/// error!("Something went wrong");
/// ```
#[derive(Debug)]
pub struct Logger {
    /// Optional handle to a file for persistent logging.
    log_file: Option<Arc<Mutex<File>>>,

    /// Shared in-memory storage of recent log entries.
    logs: Arc<RwLock<Vec<LogEntry>>>,

    /// Logger behavior configuration (verbosity, coloring, etc.)
    options: LoggerOptions,
}

lazy_static::lazy_static! {
    static ref LOGGER_INSTANCE: OnceLock<RwLock<Logger>> = OnceLock::new();
}

impl Logger {

    /// Returns a copy of the in-memory logs.
    pub fn get_logs() -> io::Result<Vec<LogEntry>> {
        let logger = LOGGER_INSTANCE
            .get()
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Logger not initialized"))?;
        let logs_guard = logger.read().map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to acquire logger read lock"))?;
        let logs = logs_guard.logs.read().map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to acquire logs read lock"))?;
        Ok(logs.clone())
    }


    /// Initializes the global logger instance.
    ///
    /// This function should be called once, usually at application startup.
    /// Subsequent calls will fail unless guarded or made idempotent.
    ///
    /// # Arguments
    ///
    /// * `log_file` - Optional path to a log file for persistent logging.
    /// * `options` - LoggerOptions to control verbosity, output, and behavior.
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

    /// Core internal logging function.
    ///
    /// This method is responsible for:
    /// - Filtering logs based on level and `LoggerOptions`.
    /// - Formatting and printing colored or plain logs to stdout/stderr.
    /// - Writing logs to file (if enabled).
    /// - Storing logs in the internal buffer up to `max_logs`.
    ///
    /// This function is invoked by the public level-specific wrappers like `log_info`, `log_error`, etc.
    ///
    /// # Parameters
    /// - `level`: The severity level of the log.
    /// - `message`: The actual log message.
    /// - `file`: Source file path (typically captured via `file!()`).
    /// - `line`: Line number in source file (typically captured via `line!()`).
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

    /// Logs an error-level message.
    pub fn log_err(message: &str, file: &str, line: u32) {
        LOGGER_INSTANCE.get().unwrap().write().unwrap().log(LogLevel::Error, message, file, line);
    }

    /// Logs a success-level message.
    pub fn log_success(message: &str, file: &str, line: u32) {
        LOGGER_INSTANCE.get().unwrap().write().unwrap().log(LogLevel::Success, message, file, line);
    }

    /// Logs an info-level message.
    pub fn log_info(message: &str, file: &str, line: u32) {
        LOGGER_INSTANCE.get().unwrap().write().unwrap().log(LogLevel::Info, message, file, line);
    }

    /// Logs a debug-level message.
    pub fn log_debug(message: &str, file: &str, line: u32) {
        LOGGER_INSTANCE.get().unwrap().write().unwrap().log(LogLevel::Debug, message, file, line);
    }

    /// Logs a warning-level message.
    pub fn log_warn(message: &str, file: &str, line: u32) {
        LOGGER_INSTANCE.get().unwrap().write().unwrap().log(LogLevel::Warn, message, file, line);
    }

    /// Logs a critical-level message.
    pub fn log_critical(message: &str, file: &str, line: u32) {
        LOGGER_INSTANCE.get().unwrap().write().unwrap().log(LogLevel::Critical, message, file, line);
    }

}
