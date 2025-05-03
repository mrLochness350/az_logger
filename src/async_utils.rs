use std::path::PathBuf;
use colored::Colorize;
use tokio::sync::mpsc::UnboundedSender;
use crate::{LogEntry, Logger};

impl Logger {
    #[cfg(feature = "async")]
    /// Spawns an asynchronous thread for asynchronous logging
    fn spawn_async_writer(path: PathBuf, truncate: bool, hide_level: bool, hide_date: bool) -> UnboundedSender<LogEntry> {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<LogEntry>();
        tokio::spawn(async move {
            use tokio::io::AsyncWriteExt;

            let mut opts = tokio::fs::OpenOptions::new();
            opts.create(true).write(true);
            if truncate {
                opts.truncate(true);
            } else {
                opts.append(true);
            }

            let mut file = match opts.open(&path).await {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("Failed to open async log file: {}", e);
                    return;
                }
            };

            while let Some(entry) = rx.recv().await {
                let msg = entry.format(hide_level, hide_date);
                println!("Received entry");
                if let Err(e) = file.write_all(msg.as_bytes()).await {
                    eprintln!("Logger async write error: {}", e);
                }
            }
        });
        tx
    }
    #[cfg(feature = "async")]
    /// Small wrapper function to check if the current runtime is a tokio runtime
    pub(crate) fn try_spawn_async_writer(path: PathBuf, truncate: bool, hide_level: bool, hide_date: bool) -> Option<UnboundedSender<LogEntry>> {
        if tokio::runtime::Handle::try_current().is_err() {
            eprintln!("{}", "[az_logger] Async logging is enabled, but no Tokio runtime is active. Defaulting to sync logging".bright_red().bold().underline().to_string());
            return None;
        }

        Some(Self::spawn_async_writer(path, truncate, hide_level, hide_date))
    }
}