//! A basic console logging backend for the `log` crate.

use log;
use log::{Log, LogLevelFilter, LogMetadata, LogRecord};
use log::SetLoggerError;
use arch::x86_64::device::serial::COM1;

/// Initializes the logger at kernel boot.
pub fn init() -> Result<(), SetLoggerError> {
    unsafe {
        log::set_logger_raw(|max_log_level| {
            static LOGGER: KernelLogger = KernelLogger;
            max_log_level.set(LOGGER.filter());
            &LOGGER
        })
    }
}

/// The kernel-level logger.
struct KernelLogger;
impl KernelLogger {
    fn filter(&self) -> LogLevelFilter {
        LogLevelFilter::Debug
    }
}
impl Log for KernelLogger {
    fn enabled(&self, metadata: &LogMetadata) -> bool {
        metadata.level() <= ::misc::LOG_LEVEL
    }

    fn log(&self, record: &LogRecord) {
        if self.enabled(record.metadata()) {
            #[cfg(feature = "logging-serial")]
            {
                use core::fmt::Write;
                write!(COM1.write(), "[{}]: {}\n", record.level(), record.args());
            }
            #[cfg(feature = "logging-console")]
            {
                println!("{}: {}", record.level(), record.args());
            }
        }
    }
}
