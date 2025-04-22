//!# az_logger
//!
//! **az_logger** is a minimal and thread-safe logger for Rust applications. It provides easy-to-use logging macros, optional file output, colored terminal output, and configurable verbosity and filtering.
//! I'm not expecting anyone to download this, as I'll be using it mostly for my Malware Development projects.
//!
//! ## Example
//! ```rust
//! use az_logger::{Logger, LoggerOptions, info};
//! Logger::init(Some("log.txt"), LoggerOptions::default()).unwrap();
//! info!("Application started!");
//! ```

mod core;

#[macro_use]
pub mod macros;
mod utils;
mod log_entry;
#[cfg(feature = "async")]
mod async_utils;

pub use core::{Logger, LoggerOptions};
pub use log_entry::{LogEntry, LogFormatStyles, LogLevel, LogFormatStyle};

/// Re-exporting color so that users can specify custom colors
pub use colored::{Color, ColoredString, Style};

/// Re-exporting for tests
pub use utils::expand_log_name_fmt;