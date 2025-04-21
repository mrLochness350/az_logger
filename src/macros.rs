#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        az_logger::Logger::log_info(&format!($($arg)*), file!(), line!());
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        az_logger::Logger::log_warn(&format!($($arg)*), file!(), line!());
    };
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        az_logger::Logger::log_debug(&format!($($arg)*), file!(), line!());
    };
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        az_logger::Logger::log_err(&format!($($arg)*), file!(), line!());
    };
}

#[macro_export]
macro_rules! success {
    ($($arg:tt)*) => {
        az_logger::Logger::log_success(&format!($($arg)*), file!(), line!());
    };
}

#[macro_export]
macro_rules! critical {
    ($($arg:tt)*) => {
        az_logger::Logger::log_critical(&format!($($arg)*), file!(), line!());
    };
}
