//! Device drivers for devices that *all x86_64 processors* (and their chipsets) have.
//!
//! Basically, stuff that's mandated by PC99.

pub mod pic;
pub mod pit;
pub mod serial;
pub mod vga_console;
