//! Hardware support and boot for 64-bit Intel x86 processors.

pub mod bits;
pub mod device;
pub mod interrupts;
pub mod memory;

use self::device::{pic, vga_console};
use logger;
use multiboot2;

/// `x86_64`-specific kernel entry point, called by the bootstrapping assembly stub.
#[no_mangle]
pub unsafe extern "C" fn _start(multiboot_info_pointer: usize) -> ! {
    vga_console::WRITER.lock().clear_screen();
    println!("--- Sparkle v{} booting! ---", ::misc::VERSION);

    logger::init().expect("Logger failed to launch!");

    let boot_info = multiboot2::load(multiboot_info_pointer);

    // initialize paging, remap kernel
    let mut mem_ctrl = memory::init(&boot_info);
    info!("memory::init() success!");

    // initialize idt
    interrupts::init(&mut mem_ctrl);
    info!("int: initialized idt");

    pic::init();
    info!("int: initialized pic");

    ::kernel_main();
}

/// Halts the CPU.
#[inline(always)]
pub unsafe fn halt() {
    asm!("hlt" :::: "volatile");
}
