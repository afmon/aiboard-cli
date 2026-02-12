use chrono::Local;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

/// Returns the path to the aiboard data directory (~/.aiboard/).
/// Creates the directory if it does not exist.
pub fn data_dir() -> Result<PathBuf, std::io::Error> {
    let home = dirs_home();
    let dir = home.join(".aiboard");
    if !dir.exists() {
        fs::create_dir_all(&dir)?;
    }
    Ok(dir)
}

/// Logs an error message to ~/.aiboard/error.log with a timestamp.
pub fn log_error(message: &str) {
    if let Err(e) = try_log_error(message) {
        eprintln!("warning: failed to write to error.log: {}", e);
    }
}

fn try_log_error(message: &str) -> Result<(), std::io::Error> {
    let dir = data_dir()?;
    let log_path = dir.join("error.log");
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)?;

    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    writeln!(file, "[{}] {}", timestamp, message)?;
    Ok(())
}

fn dirs_home() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        if let Ok(profile) = std::env::var("USERPROFILE") {
            return PathBuf::from(profile);
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home);
    }
    PathBuf::from(".")
}
