use std::path::Path;
use az_logger::{expand_log_name_fmt, Logger, LoggerOptions};

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
    init_log();
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