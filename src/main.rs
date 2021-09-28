//! The Sparkle microkernel.
#![feature(lang_items)]
#![no_std]
#![no_main]

use core::panic::PanicInfo;

/// Kernel entry point
#[no_mangle]
pub extern "C" fn _start() -> ! {
    // spin
    loop {
        x86_64::instructions::hlt();
    }
}

#[panic_handler]
pub fn panic(_info: &PanicInfo) -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}
