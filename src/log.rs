//! Little logging library in case the deserialization of the data in `settings.json`
//! fails and other scenarios.
use std::fmt;
use std::fs;
use std::io::Write;

/// The format used by [`chrono`] to convert a [`chrono::DateTime`] to a [`String`].
const DATETIME_LOG_FORMAT: &str = "%Y-%m-%d %H:%M:%S:%3f";

const LOG_PATH: &str = "log\\log.log";

/// Level of the log
#[derive(Debug)]
enum Level {
    Warning,
    Critical,
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Main function for logging a message
fn log(msg: impl Into<String>, level: Level) {
    let msg = format!(
        "{} {}: {}\n",
        today().format(DATETIME_LOG_FORMAT),
        level,
        msg.into()
    );

    // Ignore error (because we couldn't log it anywhere else)
    let Ok(mut file) = fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(LOG_PATH) else { return; };

    // Ignore error (because we couldn't log it anywhere else)
    let _ = file.write_all(msg.as_bytes()) as Result<_, _>;
}

/// Helper for [`log`] with warning level
pub fn warning(msg: impl Into<String>) {
    log(msg, Level::Warning)
}

/// Helper for [`log`] with critical level
pub fn critical(msg: impl Into<String>) {
    log(msg, Level::Critical)
}

/// Return today date as [`chrono::DateTime`]
fn today() -> chrono::DateTime<chrono::Local> {
    chrono::offset::Local::now()
}
