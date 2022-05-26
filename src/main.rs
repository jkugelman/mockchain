use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

mod csv;
mod tx;

/// Command-line arguments.
#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    file_name: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::try_parse()?;

    for record in csv::read_records(&args.file_name)? {
        println!("{:?}", record);
    }

    Ok(())
}
