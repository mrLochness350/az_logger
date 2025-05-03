# az_logger

**az_logger** is a minimal and thread-safe logger for Rust applications. It provides easy-to-use logging macros, optional file output, colored terminal output, and configurable verbosity and filtering.
I'm not expecting anyone to download this, as I'll be using it mostly for my Malware Development projects.

## Features

- Simple logging macros: `info!`, `warn!`, `debug!`, `error!`, `success!`, `critical!`
- Optional log file output
- Template-based log file output
- Colorful terminal output using the `colored` crate
- Runtime-configurable verbosity, log level filtering, and color selection
- Thread-safe
- Stores recent log entries in memory with a configurable maximum
- Serde serializable
- Optional async runtime support via [tokio](https://crates.io/crates/tokio) and the `async` feature flag

>[!IMPORTANT]  
>Using the async logger requires the tokio runtime to be enabled, so make sure it's running via the `#[tokio::main]` macro in your main function!

## Logging Macros

All macros capture the source file and line number automatically:

```rust
info!("Info message");
debug!("Debug details");
warn!("Warning issued");
error!("Error occurred");
success!("Operation succeeded");
critical!("Critical failure");
```

## LoggerOptions

You can customize the logger behaviour using the `LoggerOptions` struct:

```rust
fn main() {
    let custom_log_styles = LogFormatStyles {
        error: LogFormatStyle { fg: Some(Color::BrightRed), bg: None, style: Style::default().bold() },
        warn: LogFormatStyle { fg: Some(Color::Blue), bg: None, style: Style::default().underline() },
        info: LogFormatStyle { fg: Some(Color::White), bg: None, style: Style::default() },
        debug: LogFormatStyle { fg: Some(Color::Magenta), bg: None, style: Style::default() },
        success: LogFormatStyle { fg: Some(Color::Green), bg: None, style: Style::default() },
        critical: LogFormatStyle { fg: Some(Color::BrightYellow), bg: None, style: Style::default().bold() },
    };

    let options = LoggerOptions {
        no_console: false,
        log_to_stdout: true,
        log_to_stderr: true,
        color_output: true,
        show_debug: true,
        show_info: true,
        max_logs: 500,
        truncate_previous_logs: true,
        log_name_format: Some("<exe>_<yyyy>-<mm>-<dd>_<HH><MM><SS>.log".to_string()),
        log_dir: Some("logs_dir/".to_string()),
        custom_log_styles: Some(custom_log_styles),
        no_line_num: false,
        no_file_name: false,
        no_level_string: false,
        no_date: false,
    };
}
```

| Field                    | Description                                                             |
|--------------------------|-------------------------------------------------------------------------|
| `no_console`             | Enables or disables all output                                          |
| `log_to_stdout`          | Enables printing to stdout                                              |
| `log_to_stderr`          | Enables printing error and critical logs to stderr                      |
| `color_output`           | Enables colored output                                                  |
| `show_debug`             | Enables debug-level logs                                                |
| `show_info`              | Enables info-level logs                                                 |
| `max_logs`               | Specifies a limit on the amount of logs in the buffer                   |
| `truncate_previous_logs` | If true, the existing log file will be truncated instead of appended    |
| `log_name_format`        | Optional pattern to dynamically format the log file name                |
| `log_dir`                | Directory path where the log file will be created (if specified)        |
| `custom_log_styles`      | An optional [LogFormatStyles] struct to configure log styling per level |
| `no_line_num`            | Disables line number logging for every logger except debug/critical     |
| `no_file_name`           | Disables file name logging for every logger except debug/critical       |
| `no_level_string`        | Disables level string logging in the log entry                          |
| `no_date`                | Disables date logging for entries                                       |
>[!NOTE]  
> If `color_output` is set to `false`, `custom_log_styles` will be ignored

## File Output

To log messages to a file, provide the path to `Logger::init`:

```rust
fn main() {
    Logger::init(Some("log.txt"), LoggerOptions::default).unwrap();
}
```

You can also customize the log file's location and filename dynamically using the `log_dir` and `log_name_format` options:

```rust
fn main() {
    let mut options = LoggerOptions::default();
    options.log_dir = Some("logs".to_string());
    options.log_name_format = Some("<exe>_<yyyy>-<mm>-<dd>_<HH><MM><SS>.log".to_string());
    Logger::init(Some("ignored.txt"), options).unwrap();
}
```

### Supported filename tokens

The `log_name_format` string can contain the following placeholders:

| Token         | Description                         |
|---------------|-------------------------------------|
| `<exe>`       | Executable name (without extension) |
| `<dd>`        | Day of the month (e.g. 07)          |
| `<mm>`        | Month (e.g. 04)                     |
| `<yyyy>`      | Four-digit year (e.g. 2025)         |
| `<yy>`        | Two-digit year (e.g. 25)            |
| `<HH>`        | Hour (24-hour format)               |
| `<MM>`        | Minute                              |
| `<SS>`        | Second                              |
| `<timestamp>` | UNIX timestamp (e.g. 1713706800)    |

### Examples

Here are a few example `log_name_format` values and their possible outputs:

| Format string                                  | Example output                     | Description                                |
|------------------------------------------------|------------------------------------|--------------------------------------------|
| `<exe>.log`                                    | `my_app.log`                       | Simple log file with the executable name   |
| `<exe>_<dd>_<mm>.log`                          | `my_app_22_04.log`                 | Log file with day and month                |
| `<exe>_<yyyy>-<mm>-<dd>_<HH><MM><SS>.log`      | `my_app_2025-04-22_143015.log`     | Full timestamped log file                  |
| `session_<timestamp>.log`                      | `session_1713802456.log`           | Log file using UNIX timestamp              |
| `<exe>_<yy><mm><dd>.log`                       | `my_app_250422.log`                | Compact date format with two-digit year    |

### Notes

- If log_name_format is provided, the `Some("log.txt")` argument to `Logger::init` is ignored — the final filename is generated from the format string.
- If `log_dir` is provided, it will be automatically created if it doesn't exist.
- If neither `log_name_format` nor `log_dir` are specified, the logger will use the provided path as-is.

## Getting the Logs

If for some reason you want to get the logs currently stored in the buffer, you can call the `Logger::get_logs()` function after initializing the logger:

```rust
use az_logger::{Logger, LoggerOptions, LogEntry, info, debug, error, success};

fn main() {
  Logger::init(Some("log.txt"), LoggerOptions::default()).unwrap();
  info!("Some info");
  let logs: Vec<LogEntry> = Logger::get_logs().unwrap();
}
```

## Thread Safety

All internal state, including the log buffer and file handle, is wrapped in Arc, Mutex, and RwLock, ensuring safe concurrent access from multiple threads.
I am thinking of adding async support, but I don't know yet if it will be useful

## Example

```rust
use az_logger::{Logger, LoggerOptions, info, debug, error, success};

fn main() {
    // Initialize the logger
    let opts = LoggerOptions::default();
    Logger::init(Some("log.txt"), opts).expect("Logger initialization failed");

    info!("Application started");
    debug!("Debug details here");
    error!("An error occurred");
    success!("Finished successfully");
}
```

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
az_logger = "0.1.5"

```

Or, alternatively:

```shell
cargo add az_logger
```

## License

MIT
