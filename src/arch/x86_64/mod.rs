//! Hardware support and boot for 64-bit Intel x86 processors.

pub mod bits;
pub mod device;
pub mod interrupts;
pub mod memory;

use self::device::{pic, pit, vga_console};
use logger;
use multiboot2;
use x86_64;

/// `x86_64`-specific kernel entry point, called by the bootstrapping assembly stub.
#[no_mangle]
pub unsafe extern "C" fn _start(multiboot_info_pointer: usize) -> ! {
    vga_console::WRITER.lock().clear_screen();
    println!("--- Sparkle v{} booting! ---", ::consts::VERSION);

    logger::init().expect("Logger failed to launch!");

    let boot_info = multiboot2::load(multiboot_info_pointer);

    // initialize paging, remap kernel
    let mut mem_ctrl = memory::init(&boot_info);
    info!("memory::init() success!");

    // initialize idt
    interrupts::init(&mut mem_ctrl);
    info!("int: initialized idt");

    pic::PICS.lock().init();
    info!("int: initialized pic");

    pit::init();
    info!("int: initialized pit");

    x86_64::instructions::interrupts::enable();
    info!("int: sti (enabled interrupts)");

    ::kernel_main();
}

/// Halts the CPU.
#[inline(always)]
pub unsafe fn halt() {
    asm!("hlt" :::: "volatile");
}
