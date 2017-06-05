pub mod device;
pub mod memory;

pub fn enable_nxe_bit() {
    use x86::shared::msr;

    const NXE_BIT: u64 = 1<<11;
    unsafe {
        let efer = msr::rdmsr(msr::IA32_EFER);
        msr::wrmsr(msr::IA32_EFER, efer | NXE_BIT)
    }
}

pub fn enable_wrprot_bit() {
    use x86::shared::control_regs::{cr0, cr0_write, CR0_WRITE_PROTECT};
    unsafe {
        cr0_write(cr0() | CR0_WRITE_PROTECT);
    }
}
