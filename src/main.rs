use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

mod tx;

/// Command-line arguments.
#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    file_name: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::try_parse()?;

    for record in tx::read_records(&args.file_name)? {
        println!("{:?}", record);
    }

    Ok(())
}
