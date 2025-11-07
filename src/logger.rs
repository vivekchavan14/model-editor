use std::fs::{self, File, OpenOptions};
use std::path::PathBuf;
use std::io::Write;
use log::{Level, LevelFilter, Metadata, Record};
use chrono::Local;
use anyhow::Result;

pub struct FileLogger {
    log_file: File,
}

impl FileLogger {
    pub fn init(log_path: impl Into<PathBuf>) -> Result<()> {
        let path = log_path.into();
        
        // Create log directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let logger = FileLogger {
            log_file: OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(&path)?,
        };

        log::set_boxed_logger(Box::new(logger))?;
        log::set_max_level(LevelFilter::Debug);
        Ok(())
    }

    fn format_record(&self, record: &Record) -> String {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        format!(
            "[{}] {} [{}:{}] {}\n",
            timestamp,
            record.level(),
            record.file().unwrap_or("unknown"),
            record.line().unwrap_or(0),
            record.args()
        )
    }
}

impl log::Log for FileLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Debug
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let log_entry = self.format_record(record);
            let _ = self.log_file.try_clone().and_then(|mut file| {
                file.write_all(log_entry.as_bytes())
            });
        }
    }

    fn flush(&self) {
        let _ = self.log_file.try_clone().and_then(|mut file| file.flush());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_logger_initialization() {
        let temp_dir = tempdir().unwrap();
        let log_path = temp_dir.path().join("test.log");
        
        assert!(FileLogger::init(&log_path).is_ok());
        
        log::info!("Test log message");
        log::warn!("Test warning message");
        
        let log_contents = fs::read_to_string(&log_path).unwrap();
        assert!(log_contents.contains("Test log message"));
        assert!(log_contents.contains("Test warning message"));
    }
}
