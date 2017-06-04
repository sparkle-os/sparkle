//! A basic console logging backend for the `log` crate.

use log;
use log::{Log, LogRecord, LogLevel, LogLevelFilter, LogMetadata};
use log::{SetLoggerError};

pub fn init() -> Result<(), SetLoggerError> {
    unsafe {
        log::set_logger_raw(|max_log_level| {
            static LOGGER: ConsoleLogger = ConsoleLogger;
            max_log_level.set(LogLevelFilter::Info);
            &ConsoleLogger
        })
    }
}

struct ConsoleLogger;
impl Log for ConsoleLogger {
    fn enabled(&self, metadata: &LogMetadata) -> bool {
        metadata.level() <= LogLevel::Info
    }

    fn log(&self, record: &LogRecord) {
        if self.enabled(record.metadata()) {
            println!("{} - {}", record.level(), record.args());
        }
    }
}
