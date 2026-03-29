#![allow(unused)]

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
    pub const fn new(level: Vec<LogLevel>, log_file: String) -> Logger {
        Logger { level, log_file }
    }

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

    #[inline]
    pub fn info(&self, message: &str) {
        self.log(Info, message);
    }

    #[inline]
    pub fn debug(&self, message: &str) {
        self.log(Debug, message);
    }

    #[inline]
    pub fn warning(&self, message: &str) {
        self.log(Warning, message);
    }

    #[inline]
    pub fn error(&self, message: &str) {
        self.log(Error, message);
        panic!("Error: {}", message)
    }

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

    pub fn get_current_thread_id(&self) -> ThreadId {
        thread::current().id()
    }

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
