use std::fmt::Display;
use colored::{Color, ColoredString, Colorize, Style};
use serde::Serialize;


/// Configuration for a log message's text style.
///
/// This struct allows you to define foreground color, background color, and
/// additional style attributes (e.g., bold, underline) for a given log level.
///
/// It is typically used inside [`LogFormatStyles`] to define how each log level
/// should appear in the terminal output.
///
/// ### Example
///
/// ```rust
/// use az_logger::{LogFormatStyle, Color, Style};
///
/// let style = LogFormatStyle {
///     fg: Some(Color::Red),
///     bg: Some(Color::Black),
///     style: Style::default().bold().underline(),
/// };
///
/// let output = style.apply("Critical error!");
/// println!("{}", output);
/// ```
///
/// If `fg` or `bg` is `None`, the respective color will not be applied.
/// If `color_output` is disabled in [`LoggerOptions`], styling will be skipped entirely.
///
/// [`LoggerOptions`]: crate::LoggerOptions
/// [`LogFormatStyles`]: crate::LogFormatStyles
#[derive(Debug, Clone)]
pub struct LogFormatStyle {
    /// Optional foreground color for the log message.
    pub fg: Option<Color>,

    /// Optional background color for the log message.
    pub bg: Option<Color>,

    /// Additional style options (e.g. bold, underline).
    pub style: Style,
}


impl LogFormatStyle {
    /// Applies the configured foreground color, background color, and style to a given string.
    ///
    /// This method returns a [`ColoredString`] with the formatting applied.
    ///
    /// ### Example
    ///
    /// ```rust,ignore
    /// let styled = style.apply("log message");
    /// println!("{}", styled);
    /// ```
    ///
    /// If either `fg` or `bg` is `None`, that component is skipped.
    pub fn apply(&self, text: &str) -> ColoredString {
        let mut styled = text.normal();
        styled.style = self.style;
        if let Some(fg) = self.fg {
            styled = styled.color(fg);
        };
        if let Some(bg) = self.bg {
            styled = styled.on_color(bg);
        };
        styled
    }
}

/// Optional custom style configuration for each log level.
///
/// If set, this overrides the default color and style scheme used for terminal output.
/// Each log level can be configured with a foreground color, background color,
/// and additional styling like `bold` or `underline`.
///
/// ### Example
///
/// ```rust
/// use az_logger::{Logger, LoggerOptions, LogFormatStyles, LogFormatStyle, Color, Style};
///
/// let styles = LogFormatStyles {
///     error: LogFormatStyle {
///         fg: Some(Color::BrightRed),
///         bg: None,
///         style: Style::default().bold(),
///     },
///     warn: LogFormatStyle {
///         fg: Some(Color::Yellow),
///         bg: None,
///         style: Style::default().underline(),
///     },
///     info: LogFormatStyle {
///         fg: Some(Color::Cyan),
///         bg: None,
///         style: Style::default(),
///     },
///     debug: LogFormatStyle {
///         fg: Some(Color::Magenta),
///         bg: None,
///         style: Style::default(),
///     },
///     success: LogFormatStyle {
///         fg: Some(Color::Green),
///         bg: None,
///         style: Style::default(),
///     },
///     critical: LogFormatStyle {
///         fg: Some(Color::BrightRed),
///         bg: Some(Color::White),
///         style: Style::default().bold(),
///     },
/// };
///
/// let mut opts = LoggerOptions::default();
/// opts.color_output = true;
/// opts.custom_log_styles = Some(styles);
///
/// Logger::init(Some("log.txt"), opts).unwrap();
/// ```
///
/// If `color_output` is `false`, styles will not be applied.
///
/// Defaults to `None` (use the built-in style scheme).

#[derive(Debug, Clone)]
pub struct LogFormatStyles {
    pub error: LogFormatStyle,
    pub warn: LogFormatStyle,
    pub info: LogFormatStyle,
    pub debug: LogFormatStyle,
    pub success: LogFormatStyle,
    pub critical: LogFormatStyle,
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

    pub fn format(&self, hide_level: bool, no_date: bool) -> String {
        let date_str = if no_date { String::new() } else { format!("[{}] ", self.timestamp) };
        let level_str = if hide_level { String::new() } else { format!("[{}]", self.level) };

        match (self.file.as_ref(), self.line) {
            (Some(file), Some(line)) => format!("{}{}[{}:{}]: {}", date_str, level_str, file, line, self.message),
            (Some(file), None) => format!("{}{}[{}]: {}", date_str, level_str, file, self.message),
            (None, Some(line)) => format!("{}{}[line {}]: {}", date_str, level_str, line, self.message),
            (None, None) => format!("{}{}{}", date_str, level_str, self.message),
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