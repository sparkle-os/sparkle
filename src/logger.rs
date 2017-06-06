//! A basic console logging backend for the `log` crate.

use log;
use log::{Log, LogRecord, LogLevel, LogLevelFilter, LogMetadata};
use log::{SetLoggerError};

pub fn init() -> Result<(), SetLoggerError> {
    unsafe {
        log::set_logger_raw(|max_log_level| {
            static LOGGER: ConsoleLogger = ConsoleLogger;
            max_log_level.set(LOGGER.filter());
            &LOGGER
        })
    }
}

struct ConsoleLogger;
impl ConsoleLogger {
    fn filter(&self) -> LogLevelFilter {
        LogLevelFilter::Debug
    }
}
impl Log for ConsoleLogger {
    fn enabled(&self, metadata: &LogMetadata) -> bool {
        metadata.level() <= ::misc::LOG_LEVEL
    }

    fn log(&self, record: &LogRecord) {
        if self.enabled(record.metadata()) {
            println!("{}: {}", record.level(), record.args());
        }
    }
}
