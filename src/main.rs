//! The Sparkle microkernel.
#![feature(lang_items, asm)]
#![no_std]
#![no_main]

use core::panic::PanicInfo;
use bootloader::boot_info::BootInfo;

/// Kernel entry point
#[no_mangle]
pub extern "C" fn _start(boot_info: &'static mut BootInfo) -> ! {
    if let Some(framebuffer) = boot_info.framebuffer.as_mut() {
        for byte in framebuffer.buffer_mut() {
            *byte = 0x90;
        }
    }

    // spin
    loop {
    }
}

#[panic_handler]
pub fn panic(_info: &PanicInfo) -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}
