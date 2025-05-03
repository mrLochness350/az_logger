use std::path::Path;
use az_logger::{expand_log_name_fmt, LogEntry, LogLevel, Logger, LoggerOptions};

fn init_log() {
    let opts = LoggerOptions {
        truncate_previous_logs: true,
        log_dir: Some("test_logs".to_string()),
        log_name_format: Some("<exe>_test_<dd><mm><HH><MM><SS>.log".to_string()),
        ..Default::default()
    };
    Logger::init(Some("ignored.log"), opts).unwrap();
}

#[test]
fn test_log_writes_to_memory() {
    //init_log();
    Logger::log_info("This is a test info log", file!(), line!());
    let logs = Logger::get_logs().unwrap();
    assert!(logs.iter().any(|l| l.message.contains("test info log")));
}

#[test]
fn test_filename_expansion() {
    let result = expand_log_name_fmt("<exe>_<dd>_<mm>_<HH><MM><SS>.log");
    assert!(result.contains(".log"));
    assert!(result.contains("_"));
    assert!(!result.contains("<"));
}

#[test]
fn test_log_dir_created() {
    let path = Path::new("test_logs");
    println!("Path: {}",path.display());
    assert!(path.exists() && path.is_dir());
}

#[test]
fn test_log_entry_format_all_fields() {
    let entry = LogEntry::new(
        "01:05 1234".to_string(),
        LogLevel::Info,
        "hello",
        Some("main.rs".to_string()),
        Some(42),
    );

    let formatted = entry.format(false);
    assert_eq!(formatted, "[01:05 1234] [INFO][main.rs:42]: hello");
}

#[test]
fn test_log_entry_format_hide_level() {
    let entry = LogEntry::new(
        "01:05 1234".to_string(),
        LogLevel::Debug,
        "debugging",
        Some("debug.rs".to_string()),
        Some(10),
    );

    let formatted = entry.format(true);
    assert_eq!(formatted, "[01:05 1234] [debug.rs:10]: debugging");
}

#[test]
fn test_log_entry_format_no_file() {
    let entry = LogEntry::new(
        "01:05 1234".to_string(),
        LogLevel::Warn,
        "warning",
        None,
        Some(3),
    );

    let formatted = entry.format(false);
    assert_eq!(formatted, "[01:05 1234] [WARN][line 3]: warning");
}

#[test]
fn test_logger_respects_no_file_name_and_line_num() {
    let opts = LoggerOptions {
        no_file_name: true,
        no_line_num: true,
        log_dir: None,
        truncate_previous_logs: true,
        ..Default::default()
    };
    Logger::init(Some("in_memory.log"), opts).unwrap();
    Logger::log_warn("warn no file/line", file!(), line!());

    let logs = Logger::get_logs().unwrap();
    let entry = logs.last().unwrap();
    assert!(entry.file.is_none());
    assert!(entry.line.is_none());
    assert_eq!(entry.message, "warn no file/line");
}
