//! Internal macros used by the kernel
//! (*e.g.* our implementation of `print!`/`println!`).

use arch::x86_64::device::vga_console::WRITER;
use arch::x86_64::interrupts;
use core::fmt::{self, Write};

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ({
        $crate::macros::print(format_args!($($arg)*));
    });
}

/// Helper function to print to the console.
///
/// # TODO
/// abstract this.
pub fn print(args: fmt::Arguments) {
    interrupts::without_interrupts(|| {
        let _ = WRITER.lock().write_fmt(args);
    });
}

#[macro_export]
macro_rules! println {
    () => (print!("\n"));
    ($fmt:expr) => (print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
}
