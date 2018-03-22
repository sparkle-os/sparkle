#![feature(asm, ptr_internals, const_fn, lang_items, const_unique_new, alloc, allocator_api,
           global_allocator, abi_x86_interrupt)]
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
extern crate x86_64 as x86;

// sparkle-* libs

#[macro_use]
pub mod macros;
mod alloca;
mod misc;
mod logger;
mod arch;

use arch::x86_64::device::{serial, vga_console};
use arch::x86_64::memory;
use arch::x86_64::interrupts;
use alloca::Allocator;

/// Our globally-visible allocator. Plugs into whatever allocator we set up in [`alloca`].
//
/// [`alloca`]: alloca/index.html
#[global_allocator]
static GLOBAL_ALLOC: Allocator = Allocator {};

/// Kernel main function. Called by the bootstrapping assembly stub.
#[no_mangle]
pub extern "C" fn kernel_main(multiboot_info_pointer: usize) {
    vga_console::WRITER.lock().clear_screen();
    println!("--- Sparkle v{} booting! ---", ::misc::VERSION);

    logger::init().expect("Logger failed to launch!");

    let boot_info = unsafe { multiboot2::load(multiboot_info_pointer) };

    // initialize paging, remap kernel
    let mut mem_ctrl = memory::init(boot_info);
    info!("memory::init() success!");

    // initialize idt
    interrupts::init(&mut mem_ctrl);
    info!("int: initialized idt");

    // spin
    #[cfg_attr(feature = "cargo-clippy", allow(empty_loop))]
    loop {}
}

/// Related to stack landing pads. Don't care, do nothing.
#[lang = "eh_personality"]
#[no_mangle]
pub extern "C" fn eh_personality() {}

/// Dumps panics to the console.
#[lang = "panic_fmt"]
#[no_mangle]
pub extern "C" fn panic_fmt(
    fmt: core::fmt::Arguments,
    file: &'static str,
    line: u32,
    column: u32,
) -> ! {
    #[cfg(feature = "panic-console")]
    {
        vga_console::WRITER
            .lock()
            .set_style(vga_console::CharStyle::new(
                vga_console::Color::Black,
                vga_console::Color::Red,
            ));
        println!();
        println!("!!! PANIC in {} {}:{} !!!", file, line, column);
        println!("  {}", fmt);
    }

    #[cfg(feature = "panic-serial")]
    {
        use core::fmt::Write;
        let mut port = serial::COM1.write();
        let _ = write!(
            port,
            "\n!!! PANIC in {} {}:{} !!!\n  {}",
            file, line, column, fmt
        );
    }

    unsafe {
        loop {
            x86::instructions::halt();
        }
    };
}

/// Stack unwinding. Don't care, just halt.
#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn _Unwind_Resume() -> ! {
    unsafe {
        loop {
            x86::instructions::halt();
        }
    };
}
