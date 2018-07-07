//! Hardware support and boot for 64-bit Intel x86 processors.

pub mod bits;
pub mod device;
pub mod interrupts;
pub mod memory;

/// Halts the CPU.
#[inline(always)]
pub unsafe fn halt() {
    asm!("hlt" :::: "volatile");
}
