# az_logger

**az_logger** is a minimal and thread-safe logger for Rust applications. It provides easy-to-use logging macros, optional file output, colored terminal output, and configurable verbosity and filtering.
I'm not expecting anyone to download this, as I'll be using it mostly for my Malware Development projects.

## Features

- Simple logging macros: `info!`, `warn!`, `debug!`, `error!`, `success!`, `critical!`
- Optional log file output
- Colorful terminal output using the `colored` crate
- Runtime-configurable verbosity and log level filtering
- Thread-safe using `RwLock` and `Mutex`
- Stores recent log entries in memory with a configurable maximum
- Serde serializable

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
    let options = LoggerOptions {
        verbose: true,
        log_to_stdout: true,
        log_to_stderr: true,
        color_output: true,
        show_debug: true,
        show_info: true,
        max_logs: 500
    };
}
```

| Field         | Description                                           |
|---------------|-------------------------------------------------------|
| verbose       | Enables or disables all output                        |
 | log_to_stdout | Enables printing to stdout                            |
 | log_to_stderr | Enables printing error and critical logs to stderr    |
 | color_output  | Enables colored output                                |
 | show_debug    | Enables debug-level logs                              |
| show_info     | Enables info-level logs                               |
| max_logs      | Specifies a limit on the amount of logs in the buffer |

## File Output

To log messages to a file, provide the path to `Logger::init`:

```rust
fn main() {
    Logger::init(Some("log.txt"), LoggerOptions::default).unwrap();
}
```

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
I am thinking of adding async support but I don't know yet if it will be useful

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
az_logger = "0.1.0"

```

Or, alternatively:

```shell
cargo add az_logger
```

## License

MIT
