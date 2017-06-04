#![feature(asm)]
#![feature(unique)]
#![feature(const_fn)]
#![feature(lang_items)]
#![no_std]

#[macro_use]
extern crate log;
extern crate rlibc;
extern crate spin;
extern crate volatile;
extern crate x86;

mod arch;
#[macro_use]
mod macros;
mod misc;
mod logger;

use arch::x86_64::device::vga_console;

#[no_mangle]
pub extern fn kernel_main() {
    //////////// !!! WARNING !!! ////////////
    // THE STACK IS LARGER NOW, BUT        //
    // WE STILL HAVE NO GUARD PAGE         //
    /////////////////////////////////////////

    vga_console::WRITER.lock().clear_screen();
    println!("--- Sparkle v{} booting! ---", ::misc::VERSION);

    logger::init().expect("Logger failed to launch!");


    loop {}
}

#[lang = "eh_personality"]
#[no_mangle]
pub extern fn eh_personality() {}
#[lang = "panic_fmt"]
#[no_mangle]
pub extern fn panic_fmt(fmt: core::fmt::Arguments, file: &'static str, line: u32) -> ! {
    vga_console::WRITER.lock().set_style(vga_console::CharStyle::new(vga_console::Color::Black, vga_console::Color::Red));
    println!();
    println!("!!! PANIC in {} on line {} !!!", file, line);
    println!("  {}", fmt);

    unsafe{loop{x86::shared::halt();}};
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn _Unwind_Resume() -> ! {
    unsafe{loop{x86::shared::halt();}};
}
