# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.3] - 2025-04-22

### Added

- Support for template-based log file naming (`log_name_format`)
- `log_dir` option to write logs to a specific directory
- `truncate_previous_logs` option to control file truncation behavior
- `custom_log_styles` option to add customizability to the logs
- `no_line_num` option to disable logging line numbers
- `no_file_name` option to disable storing the filename
- Async logging support
- Integration test scaffolding for async and sync logging (`tests/` directory)
- `expand_log_name_fmt` is now public and can be reused for custom log naming in external code.
- `CHANGELOG.md` for accurately storing the changelog
- `LICENSE` for storing the license file
- `LogFormatStyle` struct for log format styling
- `LogFormatStyles` struct to store a `LogFormatStyle` for each log level
- `log_entry.rs` for clearing the clutter from `core.rs`
- `utils.rs` for utility functions

### Changed

- Improved internal formatting logic in `Logger::init`
- Improved internal formatting logic for the `Logger::log` function
- Re-exported `colored` for customizing logs
- Changes `verbose` flag to `no_console` flag

## [0.1.2] - 2025-04-21

### Added

- Complete documentation for all public types, macros, and modules

## [0.1.1] - 2025-04-21

### Fixed

- Macro exports now work properly outside the crate

## [0.1.0] - 2025-04-21

### Added

- Initial release with core `Logger` functionality
- In-memory log buffer and file output
- Colored terminal output using `colored`
