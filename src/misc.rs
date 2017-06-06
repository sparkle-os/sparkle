//! Miscellaneous things, mostly constants baked in at compile time.

use log::LogLevel;

pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");

pub const LOG_LEVEL: LogLevel = LogLevel::Info;
