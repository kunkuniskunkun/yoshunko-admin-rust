// Log Manager — per-run log files with timestamp naming

use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;

const MAX_LOGS_PER_KEY: usize = 10;

pub struct LogManager {
    log_dir: PathBuf,
}

#[derive(serde::Serialize)]
pub struct LogEntry {
    pub filename: String,
    pub display_name: String,
    pub size: u64,
}

impl LogManager {
    pub fn new(exe_dir: &std::path::Path) -> Self {
        let log_dir = exe_dir.join("logs");
        let _ = fs::create_dir_all(&log_dir);
        LogManager { log_dir }
    }

    /// Create a new log file with timestamp name. Returns (File, filename).
    pub fn create_log(&self, key: &str) -> (File, String) {
        use chrono::Local;
        let ts = Local::now().format("%Y%m%d_%H%M%S").to_string();
        let filename = format!("{}_{}.log", key, ts);
        let path = self.log_dir.join(&filename);
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .expect("Failed to create log file");
        self.cleanup_old_logs(key);
        (file, filename)
    }

    /// List log files for a key, sorted by name (newest last).
    pub fn list_logs(&self, key: &str) -> Vec<LogEntry> {
        let prefix = format!("{}_", key);
        let mut entries: Vec<LogEntry> = Vec::new();
        if let Ok(dir) = fs::read_dir(&self.log_dir) {
            for entry in dir.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with(&prefix) && name.ends_with(".log") {
                    let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                    let display = name
                        .strip_prefix(&prefix)
                        .and_then(|s| s.strip_suffix(".log"))
                        .unwrap_or(&name)
                        .to_string();
                    entries.push(LogEntry {
                        filename: name,
                        display_name: display,
                        size,
                    });
                }
            }
        }
        entries.sort_by(|a, b| a.filename.cmp(&b.filename));
        entries
    }

    /// Read content from a specific log file starting at offset.
    pub fn read_log(&self, filename: &str, offset: u64) -> (String, u64) {
        let path = self.log_dir.join(filename);
        let mut file = match File::open(&path) {
            Ok(f) => f,
            Err(_) => return (String::new(), 0),
        };
        let file_len = match file.metadata() {
            Ok(m) => m.len(),
            Err(_) => return (String::new(), 0),
        };
        if offset > file_len {
            return (String::new(), 0);
        }
        if offset == file_len {
            return (String::new(), offset);
        }
        if file.seek(SeekFrom::Start(offset)).is_err() {
            return (String::new(), 0);
        }
        let mut content = String::new();
        if file.read_to_string(&mut content).is_err() {
            return (String::new(), offset);
        }
        (content, file_len)
    }

    /// Delete old log files, keeping the newest MAX_LOGS_PER_KEY per key.
    fn cleanup_old_logs(&self, key: &str) {
        let mut logs = self.list_logs(key);
        if logs.len() <= MAX_LOGS_PER_KEY {
            return;
        }
        // Remove oldest (first in sorted order)
        let to_remove = logs.len() - MAX_LOGS_PER_KEY;
        for entry in logs.drain(..to_remove) {
            let _ = fs::remove_file(self.log_dir.join(&entry.filename));
        }
    }

    /// Return the log directory path.
    pub fn log_dir(&self) -> &std::path::Path {
        &self.log_dir
    }
}
