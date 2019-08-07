//! Driver for the Programmable Interrupt Controller (Intel 8259A).

use spin::RwLock;
use x86_64::instructions::port::Port;

/// A static, locked access point for the default chained pair of PICs on every x86 motherboard.
pub static PICS: RwLock<ChainedPics> = RwLock::new(ChainedPics::new(0x20, 0x28)); // offsets

// command constants to send
const PIC_OCW2_EOI: u8 = 0x20;

const PIC_ICW1_INIT: u8 = 0x10;
const PIC_ICW1_ICW4: u8 = 0x01;
const PIC_ICW4_MODE_8086: u8 = 0x01;

/// A pair of chained PICs.
pub struct ChainedPics {
    /// The first PIC, attached directly to the CPU.
    pub primary: Pic,
    /// The second PIC, attached to the first PIC.
    pub secondary: Pic,
}

impl ChainedPics {
    const fn new(primary_offset: u8, secondary_offset: u8) -> ChainedPics {
        ChainedPics {
            primary: Pic::new(0x20, primary_offset),
            secondary: Pic::new(0xA0, secondary_offset),
        }
    }

    /// Initializes the PIC hardware.
    ///
    /// # Notes
    /// This remaps the primary PIC to IRQs 0x20..0x28, and the secondary PIC to 0x28..0x36.
    pub unsafe fn init(&mut self) {
        // ICW1: start initialization, signal that we want the ICW4 phase
        self.primary.cmd.write(PIC_ICW1_INIT | PIC_ICW1_ICW4);
        self.secondary.cmd.write(PIC_ICW1_INIT | PIC_ICW1_ICW4);

        // ICW2: set offsets
        self.primary.data.write(self.primary.offset);
        self.secondary.data.write(self.secondary.offset);

        // ICW3: configure PIC cascading. IRQ 2 [via PC99] is used to chain to the secondary PIC.
        self.primary.data.write(1 << 2); // the IRQ 2 line is used for cascading
        self.secondary.data.write(2); // the secondary PIC has an ID of 2

        // ICW4: put the PICs into 8086 mode (EOIs are required)
        self.primary.data.write(PIC_ICW4_MODE_8086);
        self.secondary.data.write(PIC_ICW4_MODE_8086);

        // initialization done. clear IRQ masks
        self.primary.set_irq_mask(0);
        self.secondary.set_irq_mask(0);
    }

    /// Do any of the PICs in this chain handle the given INT?
    pub fn handles_int(&mut self, irq: u8) -> bool {
        [&self.secondary, &self.primary]
            .iter()
            .any(|p| p.handles_int(irq))
    }

    /// Given an interrupt, send an end-of-interrupt message to the
    /// PICs in this chain which should hear it.
    pub unsafe fn eoi(&mut self, int: u8) {
        if self.handles_int(int) {
            if self.secondary.handles_int(int) {
                self.secondary.eoi();
            }

            self.primary.eoi();
        }
    }
}

/// A single PIC.
pub struct Pic {
    cmd: Port<u8>,
    data: Port<u8>,

    /// The offset with which this PIC maps its IRQs to interrupts (ie, what INT does IRQ0 become?).
    pub offset: u8,
}

impl Pic {
    const fn new(port: u16, offset: u8) -> Pic {
        Pic {
            cmd: Port::new(port),
            data: Port::new(port + 1),
            offset,
        }
    }

    /// Send an End Of Interrupt message to this PIC.
    pub unsafe fn eoi(&mut self) {
        self.cmd.write(PIC_OCW2_EOI);
    }

    /// Returns true if this PIC handles the given interrupt.
    pub fn handles_int(&self, int: u8) -> bool {
        self.offset <= int && int < self.offset + 8
    }

    /// Get the IRQ mask of this PIC.
    ///
    /// Each bit in the IRQ mask is 0 if that IRQ is enabled, and 1 if that IRQ is masked;
    /// masked IRQs will not trigger interrupts.
    pub unsafe fn get_irq_mask(&mut self) -> u8 {
        self.data.read()
    }

    /// Get the IRQ mask of this PIC.
    ///
    /// Each bit in the IRQ mask is 0 if that IRQ is enabled, and 1 if that IRQ is masked;
    /// masked IRQs will not trigger interrupts.
    pub unsafe fn set_irq_mask(&mut self, mask: u8) {
        self.data.write(mask)
    }
}
