use std::{path::Path, process::Command};
use color_eyre::eyre::{Context, Result};

use crate::cargo_log;

pub fn run_qemu(image: &Path) -> Result<()> {
    let mut qemu = Command::new("qemu-system-x86_64");
    qemu.arg("-drive")
        .arg(format!("format=raw,file={}", image.display()))
        .arg("-no-reboot");

    cargo_log!("Launching", "{} in qemu-system-x86_64", image.display());
    let _child = qemu.spawn().context("spawning qemu failed")?;

    Ok(())
}