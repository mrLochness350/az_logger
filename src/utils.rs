use chrono::{Datelike, Local, Timelike};

/// Expands a log file name format string into a concrete file name using the current timestamp and executable name.
///
/// This function replaces placeholder tokens in the given format string with runtime values based on the
/// current local time and the name of the executable. This is useful for generating timestamped log files,
/// e.g., `"myapp_22_04_13.log"`.
///
/// ## Supported Tokens
///
/// - `<exe>`       — Name of the current executable (without extension)
/// - `<dd>`        — Day of the month, zero-padded (e.g., `07`)
/// - `<mm>`        — Month of the year, zero-padded (e.g., `04`)
/// - `<HH>`        — Hour in 24-hour format, zero-padded (e.g., `13`)
/// - `<MM>`        — Minute of the hour, zero-padded (e.g., `09`)
/// - `<SS>`        — Second of the minute, zero-padded (e.g., `45`)
/// - `<yy>`        — Two-digit year (e.g., `25`)
/// - `<yyyy>`      — Full year (e.g., `2025`)
/// - `<timestamp>` — UNIX timestamp (e.g., `1713706800`)
///
/// ## Example
///
/// ```rust,ignore
/// let name = expand_log_name_fmt("<exe>_<yyyy>-<mm>-<dd>_<HH><MM><SS>.log");
/// // Possible result: "myapp_2025-04-22_134512.log"
/// ```
///
/// ## Notes
/// - If the executable name cannot be determined, `<exe>` will be replaced with `"az_log"`.
/// - The date and time values are based on the local system clock.
///
/// # Parameters
/// - `fmt`: A string containing placeholder tokens to be expanded.
///
/// # Returns
/// A formatted log file name with all tokens replaced.
pub fn expand_log_name_fmt(fmt: impl Into<String>) -> String {
    let now = Local::now();
    let exe = std::env::current_exe()
        .ok()
        .and_then(|p| p.file_stem().map(|s| s.to_string_lossy().to_string()))
        .unwrap_or_else(|| "az_log".to_string());
    let fmt = fmt.into();
    let unsanitized = fmt.replace("<exe>", &exe)
        .replace("<dd>", &format!("{:02}", now.day()))
        .replace("<mm>", &format!("{:02}", now.month()))
        .replace("<HH>", &format!("{:02}", now.hour()))
        .replace("<yyyy>", &now.format("%Y").to_string())
        .replace("<yy>", &now.format("%y").to_string())
        .replace("<MM>", &format!("{:02}", now.minute()))
        .replace("<SS>", &format!("{:02}", now.second()))
        .replace("<timestamp>", &now.timestamp().to_string());
    sanitize_filename(&unsanitized)
}

/// Small helper to sanitize the filename
fn sanitize_filename(filename: &str) -> String {
    filename.chars()
        .filter(|c| !r#"<>"\/|?*:"#.contains(*c))
        .collect()
}