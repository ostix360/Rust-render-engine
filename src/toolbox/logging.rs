#![allow(unused)]
//! Lightweight logging helpers for stdout, files, and optional GL error tracing.

use crate::toolbox::logging::LogLevel::*;
use chrono;
use lazy_static::lazy_static;
use std::env;
use std::io::Write;
use std::thread;
use std::thread::ThreadId;

#[derive(Debug, PartialEq)]
pub enum LogLevel {
    None,
    Debug,
    Info,
    Warning,
    Error,
    GLDebug,
}

pub struct Logger {
    level: Vec<LogLevel>,
    log_file: String,
}
/// Builds the default set of enabled log levels.
///
/// OpenGL debug logging is opt-in through the `RENDER_ENGINE_GL_DEBUG` environment variable.
fn default_log_levels() -> Vec<LogLevel> {
    let mut levels = vec![Info, Debug, Error];
    if env::var_os("RENDER_ENGINE_GL_DEBUG").is_some() {
        levels.push(GLDebug);
    }
    levels
}

lazy_static! {
    pub static ref LOGGER: Logger = Logger::new(default_log_levels(), "log.txt".to_string());
}

impl Logger {
    /// Creates a logger with the supplied enabled levels and output file path.
    pub const fn new(level: Vec<LogLevel>, log_file: String) -> Logger {
        Logger { level, log_file }
    }

    /// Emits one log entry to stdout when the level is enabled and always appends it to the log
    /// file.
    #[inline]
    pub fn log(&self, level: LogLevel, message: &str) {
        let log = format!(
            "[{}]-[{:?}]-[{:?}]: {}\n",
            Logger::get_time(),
            self.get_current_thread_id(),
            level,
            message
        );
        if self.level.contains(&level) {
            println!("{}", log);
        }
        // write to file
        let file = match std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(&self.log_file)
        {
            Ok(file) => file,
            Err(error) => {
                eprintln!("Error opening log file: {}", error);
                return;
            }
        };
        let mut file = std::io::BufWriter::new(file);

        if let Err(error) = file.write(log.as_bytes()) {
            eprintln!("Error writing to log file: {}", error);
        }
    }

    /// Logs one message at `Info` level.
    #[inline]
    pub fn info(&self, message: &str) {
        self.log(Info, message);
    }

    /// Logs one message at `Debug` level.
    #[inline]
    pub fn debug(&self, message: &str) {
        self.log(Debug, message);
    }

    /// Logs one message at `Warning` level.
    #[inline]
    pub fn warning(&self, message: &str) {
        self.log(Warning, message);
    }

    /// Logs one message at `Error` level and then panics.
    ///
    /// This is used for unrecoverable runtime failures.
    #[inline]
    pub fn error(&self, message: &str) {
        self.log(Error, message);
        panic!("Error: {}", message)
    }

    /// Logs one message at `GLDebug` level when OpenGL reports an error.
    ///
    /// When GL debug logging is disabled, the function returns immediately.
    #[inline]
    pub fn gl_debug(&self, message: &str) {
        if !self.level.contains(&GLDebug) {
            return;
        }
        let error = unsafe { gl::GetError() };
        if error != gl::NO_ERROR {
            self.log(GLDebug, message);
        }
    }

    /// Returns the id of the calling thread.
    pub fn get_current_thread_id(&self) -> ThreadId {
        thread::current().id()
    }

    /// Returns the current UTC timestamp in RFC 3339 format.
    #[inline]
    pub fn get_time() -> String {
        let now = chrono::Utc::now();
        now.to_rfc3339()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_logger() {
        let logger = Logger::new(vec![Debug, Info], String::from("log.txt"));
        logger.info("This is an info message");
        logger.debug("This is a debug message");
        logger.warning("This is a warning message");
        // logger.error("This is an error message");
        // logger.gl_debug("This is a gl debug message"); // GL context is not initialized
    }
}
