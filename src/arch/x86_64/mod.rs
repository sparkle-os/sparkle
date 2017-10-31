//! Hardware support and boot for 64-bit Intel x86 processors.

pub mod device;
pub mod memory;
pub mod interrupts;

/// Turn on no-execute page protection.
pub fn enable_nxe_bit() {
    use x86::registers::msr;

    const NXE_BIT: u64 = 1<<11;
    unsafe {
        let efer = msr::rdmsr(msr::IA32_EFER);
        msr::wrmsr(msr::IA32_EFER, efer | NXE_BIT)
    }
}

/// Turn on page write-protect enforcement.
pub fn enable_wrprot_bit() {
    use x86::registers::control_regs::{cr0, cr0_write, Cr0};
    unsafe {
        cr0_write(cr0() | Cr0::WRITE_PROTECT);
    }
}
