// Log Manager — process stdout/stderr capture with rotation

use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;

const MAX_LOG_SIZE: u64 = 5 * 1024 * 1024; // 5MB
const MAX_BACKUPS: u32 = 3;

pub struct LogManager {
    log_dir: PathBuf,
}

impl LogManager {
    pub fn new(exe_dir: &std::path::Path) -> Self {
        let log_dir = exe_dir.join("logs");
        let _ = fs::create_dir_all(&log_dir);
        LogManager { log_dir }
    }

    pub fn log_path(&self, key: &str) -> PathBuf {
        self.log_dir.join(format!("{}.log", key))
    }

    /// Rotate log file if it exceeds MAX_LOG_SIZE. Best-effort.
    pub fn rotate_log(&self, key: &str) {
        let path = self.log_path(key);
        let meta = match fs::metadata(&path) {
            Ok(m) => m,
            Err(_) => return,
        };
        if meta.len() < MAX_LOG_SIZE {
            return;
        }
        // Delete oldest backup
        let oldest = self.log_dir.join(format!("{}.log.{}", key, MAX_BACKUPS));
        let _ = fs::remove_file(&oldest);
        // Shift backups: .2 → .3, .1 → .2, current → .1
        for i in (1..MAX_BACKUPS).rev() {
            let from = self.log_dir.join(format!("{}.log.{}", key, i));
            let to = self.log_dir.join(format!("{}.log.{}", key, i + 1));
            let _ = fs::rename(&from, &to);
        }
        let backup = self.log_dir.join(format!("{}.log.1", key));
        let _ = fs::rename(&path, &backup);
    }

    /// Read log content from offset. Returns (content, new_offset).
    pub fn read_log(&self, key: &str, offset: u64) -> (String, u64) {
        let path = self.log_path(key);
        let mut file = match File::open(&path) {
            Ok(f) => f,
            Err(_) => return (String::new(), 0),
        };
        let file_len = match file.metadata() {
            Ok(m) => m.len(),
            Err(_) => return (String::new(), 0),
        };
        if offset > file_len {
            // Log was rotated/truncated — reset to beginning
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

    /// Return the log directory path.
    pub fn log_dir(&self) -> &std::path::Path {
        &self.log_dir
    }
}
