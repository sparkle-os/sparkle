//! The Sparkle microkernel.
#![feature(
    asm,
    ptr_internals,
    const_fn,
    lang_items,
    alloc,
    allocator_api,
    abi_x86_interrupt,
    panic_implementation,
    panic_info_message
)]
#![warn(missing_docs)]
#![no_std]
#![cfg_attr(feature = "cargo-clippy", allow(large_digit_groups))]

#[macro_use]
extern crate alloc;
extern crate bit_field;
#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate lazy_static;
extern crate linked_list_allocator;
#[macro_use]
extern crate log;
extern crate multiboot2;
#[macro_use]
extern crate once;
extern crate rlibc;
extern crate spin;
extern crate volatile;
extern crate x86_64;

// sparkle-* libs

#[macro_use]
pub mod macros;
pub mod alloca;
pub mod arch;
mod consts;
mod logger;
pub mod panic;

use alloca::Allocator;

/// Our globally-visible allocator. Plugs into whatever allocator we set up in [`alloca`].
//
/// [`alloca`]: alloca/index.html
#[global_allocator]
static GLOBAL_ALLOC: Allocator = Allocator {};

/// Kernel main function. Called by the architecture-specific entry point,
/// after initialization is finished.
pub fn kernel_main() -> ! {
    info!("arch-init: done, entering kernel_main");

    // spin
    unsafe {
        loop {
            arch::x86_64::halt();
        }
    }
}

/// Related to stack landing pads. Don't care, do nothing.
#[lang = "eh_personality"]
#[no_mangle]
pub extern "C" fn eh_personality() {}

/// Stack unwinding. Don't care, just halt.
#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn _Unwind_Resume() -> ! {
    unsafe {
        loop {
            arch::x86_64::halt();
        }
    };
}
