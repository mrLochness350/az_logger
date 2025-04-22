/// Logs an info-level message.
///
/// ```
/// use az_logger::{Logger, LoggerOptions, info};
/// Logger::init(None::<String>, LoggerOptions::default()).unwrap();
/// info!("Server is starting...");
/// ```
///
/// Output:
/// `[21:04 15:28] [INFO]: Server is starting...`
#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        az_logger::Logger::log_info(&format!($($arg)*), file!(), line!());
    };
}

/// Logs a warning-level message.
///
/// Captures the source file and line number automatically.
///
/// ```
/// use az_logger::{Logger, LoggerOptions, warn};
/// Logger::init(None::<String>, LoggerOptions::default()).unwrap();
/// warn!("This is a warning!");
/// ```
///
/// Output:
/// `[21:04 15:28] [WARN][foo\src\main.rs:3]: This is a warning!`
#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        az_logger::Logger::log_warn(&format!($($arg)*), file!(), line!());
    };
}

/// Logs a debug-level message.
///
/// Captures the source file and line number automatically.
///
/// ```
/// use az_logger::{Logger, LoggerOptions, debug};
/// Logger::init(None::<String>, LoggerOptions::default()).unwrap();
/// debug!("Debug message!");
/// ```
///
/// Output:
/// `[21:04 15:28] [DEBUG][foo\src\main.rs:3]: Debug message!`
#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        az_logger::Logger::log_debug(&format!($($arg)*), file!(), line!());
    };
}

/// Logs an error-level message.
///
/// Captures the source file and line number automatically.
///
/// ```
/// use az_logger::{Logger, LoggerOptions, error};
/// Logger::init(None::<String>, LoggerOptions::default()).unwrap();
/// error!("This is an error message!");
/// ```
///
/// Output:
/// `[21:04 15:45] [ERROR][foo\src\main.rs.rs:3]: This is an error message!`
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        az_logger::Logger::log_err(&format!($($arg)*), file!(), line!());
    };
}

/// Logs a success-level message.
///
/// ```
/// use az_logger::{Logger, LoggerOptions, success};
/// Logger::init(None::<String>, LoggerOptions::default()).unwrap();
/// success!("Server has started!");
/// ```
///
/// Output:
/// `[21:04 15:45] [SUCCESS]: Finished reading file!`
#[macro_export]
macro_rules! success {
    ($($arg:tt)*) => {
        az_logger::Logger::log_success(&format!($($arg)*), file!(), line!());
    };
}


/// Logs a critical-level message.
///
/// Captures the source file and line number automatically.
///
/// ```
/// use az_logger::{Logger, LoggerOptions, critical};
/// Logger::init(None::<String>, LoggerOptions::default()).unwrap();
/// critical!("A critical event has occurred!");
/// ```
///
/// Output:
/// `[21:04 15:48] [CRITICAL][foo\src\main.rs.rs:3]: A critical event has occurred!`
#[macro_export]
macro_rules! critical {
    ($($arg:tt)*) => {
        az_logger::Logger::log_critical(&format!($($arg)*), file!(), line!());
    };
}
