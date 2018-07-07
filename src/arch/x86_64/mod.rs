//! Hardware support and boot for 64-bit Intel x86 processors.

pub mod device;
pub mod memory;
pub mod interrupts;
pub mod bits;

/// Halts the CPU.
#[inline(always)]	
pub unsafe fn halt() {	
    asm!("hlt" :::: "volatile");	
}