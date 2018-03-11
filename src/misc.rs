//! Miscellaneous things, mostly constants baked in at compile time.

use log::LogLevel;

/// The crate version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The highest log level to dump to the console. Baked in at compile time for now.
pub const LOG_LEVEL: LogLevel = LogLevel::Info;
