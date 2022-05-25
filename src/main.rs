use std::fs::File;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    file_name: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::try_parse()?;

    let file_name = &args.file_name;
    let _file = File::open(file_name)
        .with_context(|| format!("Could not open {}", file_name.display()))?;

    Ok(())
}
