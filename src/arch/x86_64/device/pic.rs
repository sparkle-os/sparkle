//! Driver for the Programmable Interrupt Controller (Intel 8259A).

use x86_64::instructions::port::Port;

// using the terminology from Intel's 1988 datasheet,
// "8259A PROGRAMMABLE INTERRUPT CONTROLLER".
const MASTER: Pic = Pic::new(0x20);
const SLAVE: Pic = Pic::new(0xA0);

// command constants to send
const PIC_OCW2_EOI: u8 = 0x20;

const PIC_ICW1_INIT: u8 = 0x10;
const PIC_ICW1_ICW4: u8 = 0x01;
const PIC_ICW4_MODE_8086: u8 = 0x01;

/// Initializes the PIC hardware.
///
/// # Notes
/// This remaps the master PIC to IRQs 0x20..0x28, and the slave to 0x28..0x36.
pub unsafe fn init() {
    // ICW1: start initialization, signal that we want the ICW4 phase
    MASTER.cmd.write(PIC_ICW1_INIT | PIC_ICW1_ICW4);
    SLAVE.cmd.write(PIC_ICW1_INIT | PIC_ICW1_ICW4);

    // ICW2: set offsets
    MASTER.data.write(0x20);
    SLAVE.data.write(0x28);

    // ICW3: configure PIC cascading. IRQ 2 [via PC99] is used to chain to the slave PIC.
    MASTER.data.write(1 << 2); // the IRQ 2 line is used for cascading
    SLAVE.data.write(2); // the slave has an ID of 2

    // ICW4: put the PICs into 8086 mode (EOIs are required)
    MASTER.data.write(PIC_ICW4_MODE_8086);
    SLAVE.data.write(PIC_ICW4_MODE_8086);

    // initialization done. clear IRQ masks
    MASTER.set_irq_mask(0);
    SLAVE.set_irq_mask(0);
}

pub struct Pic {
    cmd: Port<u8>,
    data: Port<u8>,
}

impl Pic {
    pub const fn new(port: u16) -> Pic {
        Pic {
            cmd: Port::new(port),
            data: Port::new(port + 1),
        }
    }

    pub unsafe fn eoi(&mut self) {
        self.cmd.write(PIC_OCW2_EOI);
    }

    unsafe fn get_irq_mask(&self) -> u8 {
        self.data.read()
    }

    unsafe fn set_irq_mask(&mut self, mask: u8) {
        self.data.write(mask)
    }
}
