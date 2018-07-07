//! A basic console logging backend for the `log` crate.

use log;
use log::{LevelFilter, Level, Log, Metadata, Record, SetLoggerError};
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
        ::misc::LOG_LEVEL.to_level_filter()
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
                use core::fmt::Write;
                use ::arch::x86_64::device::vga_console::{WRITER, CharStyle, Color};

                let mut wtr = WRITER.lock();
                let sty = CharStyle::new(match record.level() {
                    Level::Error => Color::Red,
                    Level::Warn => Color::Magenta,
                    Level::Info => Color::Green,
                    Level::Debug => Color::Cyan,
                    Level::Trace => Color::White,
                }, Color::DarkGray);
                let _ = write!(wtr.styled().set_style(sty), "{:>5}", record.level());

                let _ = writeln!(wtr, ": {}", record.args());
            }
        }
    }

    fn flush(&self) {}
}
