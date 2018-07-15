//! A basic console logging backend for the `log` crate.

use arch::x86_64::device::serial::COM1;
use log;
use log::{Level, LevelFilter, Log, Metadata, Record, SetLoggerError};

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
        ::consts::LOG_LEVEL.to_level_filter()
    }
}
impl Log for KernelLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= ::consts::LOG_LEVEL
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            #[cfg(feature = "logging-serial")]
            {
                use core::fmt::Write;
                writeln!(COM1.write(), "[{}]: {}", record.level(), record.args()).unwrap();
            }
            #[cfg(feature = "logging-console")]
            {
                use arch::x86_64::device::vga_console::{CharStyle, Color, WRITER};
                use core::fmt::Write;

                let mut wtr = WRITER.lock();
                let sty = CharStyle::new(
                    match record.level() {
                        Level::Error => Color::Red,
                        Level::Warn => Color::Magenta,
                        Level::Info => Color::Green,
                        Level::Debug => Color::Cyan,
                        Level::Trace => Color::White,
                    },
                    Color::DarkGray,
                );
                write!(wtr.styled().set_style(sty), "{:>5}", record.level()).unwrap();

                writeln!(wtr, ": {}", record.args()).unwrap();;
            }
        }
    }

    fn flush(&self) {}
}
