// Code used under
pub const CARGO_LOG_WIDTH: usize = 12;
pub use owo_colors::{style, OwoColorize};

/// Pretend we're cargo and log like it.
#[macro_export]
macro_rules! cargo_log {
    ($tag:expr, $($arg:tt)+) => {{
        use $crate::term::OwoColorize;
        eprintln!("{:>width$} {}", $tag.green().bold(), format_args!($($arg)+), width = $crate::term::CARGO_LOG_WIDTH);
    }};
}
