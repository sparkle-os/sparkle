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
#[macro_use]
extern crate bitflags;
extern crate x86;
extern crate multiboot2;

#[macro_use]
pub mod macros;
mod misc;
mod logger;
mod arch;

use arch::x86_64::device::vga_console;
use arch::x86_64::memory;
use arch::x86_64::memory::FrameAllocator;

#[no_mangle]
pub extern fn kernel_main(multiboot_info_pointer: usize) {
    //////////// !!! WARNING !!! ////////////
    // THE STACK IS LARGER NOW, BUT        //
    // WE STILL HAVE NO GUARD PAGE         //
    /////////////////////////////////////////

    vga_console::WRITER.lock().clear_screen();
    println!("--- Sparkle v{} booting! ---", ::misc::VERSION);

    logger::init().expect("Logger failed to launch!");

    let boot_info = unsafe {multiboot2::load(multiboot_info_pointer)};
    let memory_map_tag = boot_info.memory_map_tag()
        .expect("multiboot: Memory map tag required");
    let elf_sections_tag = boot_info.elf_sections_tag()
        .expect("multiboot: ELF sections tag required");

    println!("memory areas:");
    for area in memory_map_tag.memory_areas() {
        println!("  start: 0x{:x}, length: 0x{:x}",
            area.base_addr, area.length);
    }
    /*debug!("kernel sections:");
    for section in elf_sections_tag.sections() {
        debug!("  start: 0x{:x}, size: 0x{:x}, flags: 0x{:x}",
            section.addr, section.size, section.flags);
    }*/

    let kernel_start = elf_sections_tag.sections().map(|s| s.addr)
        .min().unwrap();
    let kernel_end = elf_sections_tag.sections().map(|s| s.addr + s.size)
        .max().unwrap();
    let multiboot_start = multiboot_info_pointer;
    let multiboot_end = multiboot_start + (boot_info.total_size as usize);
    let mut frame_allocator = memory::AreaFrameAllocator::new(
        kernel_start as usize, kernel_end as usize, multiboot_start,
        multiboot_end, memory_map_tag.memory_areas());

    memory::remap_kernel(&mut frame_allocator, boot_info);

    println!("-- remap_kernel finished! --");

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
