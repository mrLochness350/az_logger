pub(crate) use crate::log_entry::{LogEntry, LogLevel};
use crate::utils::expand_log_name_fmt;
use crate::LogFormatStyles;
use chrono::Local;
use colored::Colorize;
use std::path::{Path, PathBuf};
use std::{fs, fs::{File, OpenOptions}, io::{self, Write}, sync::{Arc, Mutex, OnceLock, RwLock}};
#[cfg(feature="async")]
use tokio::sync::mpsc::UnboundedSender;

/// Configuration options for the global logger instance.
#[derive(Debug, Clone)]
pub struct LoggerOptions {
    /// Enables or disables logging entirely (This flag may be removed in the future).
    pub no_console: bool,
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
    /// If set to `true`, the existing log file (if any) will be truncated when the logger starts.
    /// Otherwise, new logs will be appended to the existing file.
    pub truncate_previous_logs: bool,
    /// Optional formatting string for dynamically naming log files.
    /// Supports tokens like `<exe>`, `<dd>`, `<mm>`, `<HH>`, `<MM>`, `<SS>`, `<yy>`, `<yyyy>`, and `<timestamp>`.
    ///
    /// Example: `"<exe>_<yyyy>-<mm>-<dd>_<HH><MM><SS>.log"` might become `"myapp_2025-04-22_145109.log"`
    pub log_name_format: Option<String>,
    /// Optional directory path where log files will be stored.
    /// If the directory does not exist, it will be created automatically.
    pub log_dir: Option<String>,
    /// Optional custom color configuration for each log level.
    pub custom_log_styles: Option<LogFormatStyles>,
    /// Turns off the line number logging for every logger but the debug and critical loggers
    pub no_line_num: bool,
    /// Turns off the file name logging for every logger but the debug and critical loggers
    pub no_file_name: bool
}


impl Default for LoggerOptions {
    fn default() -> Self {
        Self {
            no_console: true,
            log_to_stdout: true,
            log_to_stderr: true,
            color_output: true,
            show_debug: true,
            show_info: true,
            max_logs: 500,
            truncate_previous_logs: false,
            log_dir: None,
            log_name_format: None,
            custom_log_styles: None,
            no_file_name: false,
            no_line_num: false

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

    #[cfg(feature = "async")]
    /// Async sender for the logger
    pub(crate) async_sender: Option<UnboundedSender<LogEntry>>,
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
        if let Some(log_dir) = &options.log_dir {
            let pb = PathBuf::from(log_dir);
            if !pb.exists() {
                fs::create_dir_all(&pb)?;
            }
        }

        let path = match (&options.log_name_format, log_file) {
            (Some(fmt), _) => {
                let filename = expand_log_name_fmt(fmt);
                match &options.log_dir {
                    Some(dir) => Path::new(dir).join(filename),
                    None => PathBuf::from(filename),
                }
            }
            (None, Some(path)) => {
                let path: String = path.into();
                PathBuf::from(path)
            },
            _ => PathBuf::new(),
        };

        let logfile = if path.as_os_str().is_empty() {
            None
        } else {
            let opts = Self::create_log_file_options(options.truncate_previous_logs);
            Some(opts.open(&path)
                .map(|f| Arc::new(Mutex::new(f)))
                .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to open log file: {}", e)))?)
        };
        #[cfg(feature = "async")]
        let async_sender = if !path.as_os_str().is_empty() {
            Self::try_spawn_async_writer(path.clone(), options.truncate_previous_logs)
        } else {
            None
        };

        let logger = Logger {
            log_file: logfile,
            logs: Arc::new(RwLock::new(Vec::with_capacity(options.max_logs))),
            options,
            #[cfg(feature = "async")]
            async_sender,
        };

        LOGGER_INSTANCE
            .set(RwLock::new(logger))
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "Logger already initialized"))?;

        Ok(())
    }

    /// Internal util to clear clutter
    fn create_log_file_options(truncate: bool) ->  OpenOptions {
        let mut opts = OpenOptions::new();
        opts.create(true);
        opts.write(true);
        if truncate {
            opts.truncate(true);
        } else {
            opts.append(true);
        }
        opts
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
        if !self.options.no_console {
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

        let entry = match level {
            LogLevel::Error | LogLevel::Warn | LogLevel::Success | LogLevel::Info => {
                if self.options.no_line_num {
                    LogEntry::new(timestamp.clone(), level.clone(), message, Some(file.to_string()), None)
                } else if self.options.no_line_num && self.options.no_file_name {
                    LogEntry::new(timestamp.clone(), level.clone(), message, None, None)
                } else {
                    LogEntry::new(timestamp.clone(), level.clone(), message, Some(file.to_string()), Some(line))
                }
            }
            _ => {
                LogEntry::new(timestamp.clone(), level.clone(), message, Some(file.to_string()), Some(line))
            }
        };
        let fmt = entry.format();
        log_lock.push(entry);
        if self.options.log_to_stdout || self.options.log_to_stderr {
            let log_entry = self.apply_log_color(&level, &fmt);
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

        #[cfg(feature = "async")]
        if let Some(sender) = &self.async_sender {
            let log_entry = log_lock.last().unwrap().clone();
            let _ = sender.send(log_entry);
        } else if let Some(file) = &self.log_file {
            let mut file = file.lock().unwrap();
            writeln!(file, "{}", fmt).unwrap();
        }
        #[cfg(not(feature = "async"))]
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

    /// Apply color formatting to the log message based on level and user options.
    fn apply_log_color(&self, level: &LogLevel, message: &str) -> String {
        if !self.options.color_output {
            return message.to_string();
        }

        if let Some(colors) = &self.options.custom_log_styles {
            let colored = match level {
                LogLevel::Error => colors.error.apply(message),
                LogLevel::Warn => colors.warn.apply(message),
                LogLevel::Info => colors.info.apply(message),
                LogLevel::Debug => colors.debug.apply(message),
                LogLevel::Success => colors.success.apply(message),
                LogLevel::Critical => colors.critical.apply(message),
            };
            return colored.to_string();
        }

        let default_colors = match level {
            LogLevel::Debug => message.yellow().on_black(),
            LogLevel::Error => message.bright_red().bold(),
            LogLevel::Warn => message.yellow(),
            LogLevel::Info => message.cyan(),
            LogLevel::Success => message.green(),
            LogLevel::Critical => message.bright_red().bold().on_bright_cyan(),
        };

        default_colors.to_string()
    }
}
