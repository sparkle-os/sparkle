//! Shorthand for flipping CPU bits.

// TODO: stuff in here should be unsafe; NXE/WRPROT could compromise memory safety.

/// Turn on no-execute page protection.
pub fn enable_nxe() {
    use x86_64::registers::model_specific::{Efer, EferFlags};

    unsafe {
        Efer::update(|flags| *flags |= EferFlags::NO_EXECUTE_ENABLE);
    }
}

/// Turn on page write-protect enforcement.
pub fn enable_wrprot() {
    use x86_64::registers::control::{Cr0, Cr0Flags};
    unsafe {
        Cr0::update(|flags| *flags |= Cr0Flags::WRITE_PROTECT);
    }
}
