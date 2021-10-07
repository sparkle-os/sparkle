use color_eyre::{
    eyre::{ensure, format_err, Result, WrapErr},
    Help,
};
use kamaji::{cargo_log, qemu::run_qemu};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use structopt::StructOpt;
use tracing_subscriber::prelude::*;

#[derive(StructOpt, Debug)]
#[structopt(name = "kamaji", about = "horrid little build system")]
struct Options {
    #[structopt(subcommand)]
    cmd: Option<Subcommand>,

    /// Path to the kernel binary.
    #[structopt(parse(from_os_str))]
    kernel_bin: PathBuf,

    /// Configures build logging.
    #[structopt(short, long, env = "RUST_LOG", default_value = "warn")]
    log: String,

    /// Bootloader manifest.
    #[structopt(long, parse(from_os_str))]
    bootloader_manifest: Option<PathBuf>,

    /// Kernel manifest.
    #[structopt(long, parse(from_os_str))]
    kernel_manifest: Option<PathBuf>,

    /// Output directory for Sparkle image
    #[structopt(long, parse(from_os_str))]
    out_dir: Option<PathBuf>,

    /// Target directory for image build process
    #[structopt(long, parse(from_os_str))]
    target_dir: Option<PathBuf>,
}

#[derive(Debug, StructOpt)]
enum Subcommand {
    Run,
}

impl Options {
    fn bootloader_manifest(&self) -> Result<PathBuf> {
        if let Some(path) = &self.bootloader_manifest {
            tracing::info!(path = %path.display(), "bootloader manifest path overridden");
            Ok(path.clone())
        } else {
            bootloader_locator::locate_bootloader("bootloader")
                .note("uh where the fuck is the `bootloader` Cargo.toml?")
                .suggestion("did you actually depend on `bootloader`?")
        }
    }

    fn kernel_manifest(&self) -> Result<PathBuf> {
        if let Some(path) = &self.kernel_manifest {
            tracing::info!(path = %path.display(), "kernel manifest path overridden");
            Ok(path.clone())
        } else {
            locate_cargo_manifest::locate_manifest()
                .note("where the fuck is your Cargo.toml")
                .note("literally how are you even running this??")
                .suggestion("stop having it be missing")
        }
    }

    fn kernel_bin(&self) -> Result<PathBuf> {
        self.kernel_bin
            .canonicalize() // bootloader build script gets mad if it's not a canonical path
            .context("couldn't canonicalize kernel binary path")
    }
}

fn out_dir(options: &Options, kernel_bin: &Path) -> Result<PathBuf> {
    if let Some(path) = &options.out_dir {
        tracing::info!(path = %path.display(), "out dir overridden");
        Ok(path.clone())
    } else {
        kernel_bin
            .parent()
            .map(Path::to_path_buf)
            .ok_or_else(|| format_err!("can't find a parent dir for the kernel"))
            .suggestion("why are you outputing the kernel bin in `/`???")
    }
}

fn target_dir(options: &Options, kernel_manifest: &Path) -> Result<PathBuf> {
    if let Some(path) = &options.target_dir {
        tracing::info!(path = %path.display(), "target dir overridden");
        Ok(path.clone())
    } else {
        kernel_manifest
            .parent()
            .map(Path::to_path_buf)
            .ok_or_else(|| format_err!("can't find a parent dir for the kernel"))
            .suggestion("why are you outputing the kernel bin in `/`???")
            .map(|p| p.join("target"))
    }
}

fn find_cargo() -> Result<PathBuf> {
    if let Some(cargo) = std::env::var_os("CARGO").map(PathBuf::from) {
        ensure!(
            cargo.exists(),
            "CARGO env variable ({}) points at a nonexistent file",
            cargo.display()
        );

        Ok(cargo)
    } else {
        let cargo = PathBuf::from(env!("CARGO"));
        ensure!(
            cargo.exists(),
            "build-time env! variable CARGO ({}) points at a nonexistent file",
            cargo.display()
        );
        Ok(cargo)
    }
}

fn cargo(cmd: &str) -> Result<Command> {
    let cargo = find_cargo().with_context(|| format!("failed to run `cargo {}`", cmd))?;

    let mut cargo = Command::new(cargo);
    cargo.arg(cmd);

    Ok(cargo)
}

struct Images {
    uefi: PathBuf,
    bios: PathBuf,
}

fn make_image(
    options: &Options,
    bootloader_manifest: &Path,
    kernel_manifest: &Path,
    kernel_bin: &Path,
) -> Result<Images> {
    cargo_log!(
        "Building",
        "disk image from kernel binary {}",
        kernel_bin.display()
    );

    let run_dir = bootloader_manifest
        .parent()
        .ok_or_else(|| format_err!("bootloader manifest doesn't have a parent dir"))
        .suggestion("did u somehow install `bootloader` in `/`?????")?;

    let target_dir = target_dir(options, kernel_manifest)?;
    let out_dir = out_dir(options, kernel_bin)?;

    let out = cargo("builder")?
        .current_dir(run_dir)
        .arg("--kernel-manifest")
        .arg(kernel_manifest)
        .arg("--kernel-binary")
        .arg(kernel_bin)
        .arg("--target-dir")
        .arg(&target_dir)
        .arg("--out-dir")
        .arg(&out_dir)
        .stderr(Stdio::inherit())
        .stdout(Stdio::piped())
        .status()
        .context("running builder command")?;

    ensure!(
        out.success(),
        "`bootloader` builder command exited with non-zero status {:?}",
        out.code()
    );

    let kernel_name = kernel_bin
        .file_name()
        .ok_or_else(|| format_err!("how the fuck do you have a kernel bin path ending in .."))?
        .to_str()
        .ok_or_else(|| format_err!("apparently your kernel binary name isn't valid utf-8 :///"))?;

    let uefi_image = out_dir.join(format!("boot-uefi-{}.img", kernel_name));
    ensure!(
        uefi_image.exists(),
        "uefi image really should exist after successfully running `bootloader` builder"
    );
    cargo_log!("Created", "bootable (UEFI) disk image at {}", uefi_image.display());

    let bios_image = out_dir.join(format!("boot-bios-{}.img", kernel_name));
    ensure!(
        bios_image.exists(),
        "uefi image really should exist after successfully running `bootloader` builder"
    );
    cargo_log!("Created", "bootable (BIOS) disk image at {}", bios_image.display());


    Ok(Images { uefi: uefi_image, bios: bios_image })
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let opts = Options::from_args();
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(opts.log.parse::<tracing_subscriber::EnvFilter>()?)
        .init();

    tracing::info! {
        ?opts.kernel_bin,
        ?opts.bootloader_manifest,
        ?opts.kernel_manifest,
        "kamaji stoking the boilers...",
    };

    let bootloader_manifest = opts.bootloader_manifest()?;
    tracing::info!(path = %bootloader_manifest.display(), "found bootloader manifest");

    let kernel_manifest = opts.kernel_manifest()?;
    tracing::info!(path = %kernel_manifest.display(), "found kernel manifest");

    let kernel_bin = opts.kernel_bin()?;
    tracing::info!(path = %kernel_bin.display(), "found kernel bin");

    let image_path = make_image(&opts, &bootloader_manifest, &kernel_manifest, &kernel_bin)?;

    match opts.cmd {
        None => (),
        Some(Subcommand::Run) => {
            run_qemu(&image_path.bios)?;
        },
    }

    Ok(())
}
