//! Driver for the Programmable Interrupt Timer (Intel 8253/8254).

use x86_64::instructions::port::Port;

pub static mut PIT: Pit = Pit {
    chan0: Port::new(0x40),
    mode: Port::new(0x43),
};

const SELECT_CHAN0: u8 = 0;
const ACCESS_LOHI: u8 = 0x30;
const MODE_2: u8 = 1 << 2;

/// Base frequency of the PIT, in _Hz_.
pub const FREQ: u32 = 1193182;

/// The target frequency to tick at, in _Hz_.
pub const TICK_FREQ: u32 = 20;

/// Initialize the PIT.
pub unsafe fn init() {
    const DIVISOR: u16 = (FREQ / TICK_FREQ) as u16;

    PIT.mode.write(ACCESS_LOHI | SELECT_CHAN0 | MODE_2);
    PIT.chan0.write((DIVISOR & 0xff) as u8);
    PIT.chan0.write((DIVISOR >> 8) as u8);
}

pub struct Pit {
    chan0: Port<u8>,
    mode: Port<u8>,
}
