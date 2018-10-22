use core::panic::PanicInfo;
use x86_64;

/// Dumps panics to the console.
#[panic_handler]
#[no_mangle]
pub extern "C" fn panic(info: &PanicInfo) -> ! {
    x86_64::instructions::interrupts::disable();

    #[cfg(feature = "panic-console")]
    {
        use arch::x86_64::device::vga_console;
        vga_console::WRITER
            .lock()
            .set_style(vga_console::CharStyle::new(
                vga_console::Color::Black,
                vga_console::Color::Red,
            ));
        println!();

        if let Some(location) = info.location() {
            println!(
                "!!! PANIC in {} {}:{} !!!",
                location.file(),
                location.line(),
                location.column()
            );
        } else {
            println!("!!! PANIC at unknown location !!!");
        }

        if let Some(message) = info.message() {
            println!("  {}", message);
        }
    }

    #[cfg(feature = "panic-serial")]
    {
        use arch::x86_64::device::serial;
        use core::fmt::Write;
        let mut port = serial::COM1.write();

        if let Some(location) = info.location() {
            let _ = writeln!(
                port,
                "!!! PANIC in {} {}:{} !!!",
                location.file(),
                location.line(),
                location.column()
            );
        } else {
            let _ = writeln!(port, "!!! PANIC at unknown location !!!");
        }

        if let Some(message) = info.message() {
            let _ = writeln!(port, "  {}", message);
        }
    }

    unsafe {
        loop {
            ::arch::x86_64::halt();
        }
    };
}
