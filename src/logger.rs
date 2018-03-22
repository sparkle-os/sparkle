//! A basic console logging backend for the `log` crate.

use log;
use log::{Log, LevelFilter, Metadata, Record, SetLoggerError};
use arch::x86_64::device::serial::COM1;

static LOGGER: KernelLogger = KernelLogger;

/// Initializes the logger at kernel boot.
pub fn init() -> Result<(), SetLoggerError> {
    log::set_logger(&LOGGER)?;
    log::set_max_level(LOGGER.filter());

    Ok(())
}

/// The kernel-level logger.
struct KernelLogger;
impl KernelLogger {
    fn filter(&self) -> LevelFilter {
        LevelFilter::Debug
    }
}
impl Log for KernelLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= ::misc::LOG_LEVEL
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            #[cfg(feature = "logging-serial")]
            {
                use core::fmt::Write;
                write!(COM1.write(), "[{}]: {}\n", record.level(), record.args()).unwrap();
            }
            #[cfg(feature = "logging-console")]
            {
                println!("{}: {}", record.level(), record.args());
            }
        }
    }

    fn flush(&self) {}
}
