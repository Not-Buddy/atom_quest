use chrono::Local;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

pub struct FileLogger {
    log_path: PathBuf,
}

impl FileLogger {
    pub fn new() -> std::io::Result<Self> {
        // Create logs directory if it doesn't exist
        fs::create_dir_all("logs")?;

        // Get current date for filename
        let date = Local::now().format("%Y-%m-%d").to_string();
        let log_path = PathBuf::from(format!("logs/{}.log", date));

        // Create or open the log file
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)?;

        // Write session start marker
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        writeln!(file, "\n========== Session started at {} ==========", timestamp)?;

        Ok(Self { log_path })
    }

    pub fn log(&self, message: &str) -> std::io::Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)?;

        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        writeln!(file, "[{}] {}", timestamp, message)?;
        Ok(())
    }

    pub fn log_with_level(&self, level: &str, message: &str) -> std::io::Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)?;

        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        writeln!(file, "[{}] [{}] {}", timestamp, level, message)?;
        Ok(())
    }

    pub fn log_request(&self, method: &str, path: &str, status: u16, duration_ms: f64) -> std::io::Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)?;

        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        writeln!(
            file,
            "[{}] [REQUEST] {} {} - {} ({:.2}ms)",
            timestamp, method, path, status, duration_ms
        )?;
        Ok(())
    }
}
