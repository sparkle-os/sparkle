use color_eyre::eyre::Result;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
struct Options {
    #[structopt(parse(from_os_str))]
    kernel_bin: PathBuf,
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let opts = Options::from_args();

    Ok(())
}
