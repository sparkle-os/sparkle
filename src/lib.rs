#![feature(asm)]
#![feature(unique)]
#![feature(const_fn)]
#![feature(lang_items)]
#![feature(const_unique_new)]
#![feature(alloc)]
#![feature(allocator_api)]
#![feature(global_allocator)]
#![no_std]

#[macro_use]
extern crate log;
#[macro_use]
extern crate once;
#[macro_use]
extern crate alloc;
extern crate rlibc;
extern crate spin;
extern crate volatile;
#[macro_use]
extern crate bitflags;
extern crate x86;
extern crate multiboot2;
extern crate linked_list_allocator;

// sparkle-* libs

#[macro_use]
pub mod macros;
mod alloca;
mod misc;
mod logger;
mod arch;

use arch::x86_64;
use arch::x86_64::device::vga_console;
use arch::x86_64::memory;
use arch::x86_64::memory::FrameAllocator;
use alloca::Allocator;
use alloc::boxed::Box;

/// Our globally-visible allocator. Plugs into whatever allocator we set up in [`alloca`].
//
/// [`alloca`]: alloca/index.html
#[global_allocator]
static GLOBAL_ALLOC: Allocator = Allocator;

/// Kernel main function. Called by the bootstrapping assembly stub.
#[no_mangle]
pub extern fn kernel_main(multiboot_info_pointer: usize) {
    vga_console::WRITER.lock().clear_screen();
    println!("--- Sparkle v{} booting! ---", ::misc::VERSION);

    logger::init().expect("Logger failed to launch!");

    let boot_info = unsafe {multiboot2::load(multiboot_info_pointer)};

    memory::init(boot_info);
    info!("* memory::init(): success! *");
    info!("kheap: smoke test (boxing): {:?}", Box::new("hello world"));

    loop {}
}

/// Related to stack landing pads. Don't care, do nothing.
#[lang = "eh_personality"]
#[no_mangle]
pub extern fn eh_personality() {}

/// Dumps panics to the console.
#[lang = "panic_fmt"]
#[no_mangle]
pub extern fn panic_fmt(fmt: core::fmt::Arguments, file: &'static str, line: u32) -> ! {
    vga_console::WRITER.lock().set_style(vga_console::CharStyle::new(vga_console::Color::Black, vga_console::Color::Red));
    println!();
    println!("!!! PANIC in {} on line {} !!!", file, line);
    println!("  {}", fmt);

    unsafe{loop{x86::shared::halt();}};
}

/// Stack unwinding. Don't care, just halt.
#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn _Unwind_Resume() -> ! {
    unsafe{loop{x86::shared::halt();}};
}
