mod csv;
mod db;

use std::fs::File;
use std::io::stdout;
use std::path::{Path, PathBuf};

use anyhow::Context;
use clap::Parser;

use crate::db::Database;

/// Command-line arguments.
#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    file_name: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let args = Args::try_parse()?;

    let db = process_txs(&args.file_name)?;
    csv::account::write(stdout(), &db)?;

    Ok(())
}

fn process_txs(file_name: &Path) -> anyhow::Result<Database> {
    let mut db = Database::new();
    let file = File::open(&file_name)
        .with_context(|| format!("{}: could not open file", file_name.display()))?;

    // Read and process each transaction one at a time.
    for (line, record) in csv::tx::read(file).enumerate() {
        let line = line + 1;
        let record = record.with_context(|| {
            format!("{}:{}: error parsing CSV record", file_name.display(), line)
        })?;

        if let Err(err) = record.apply(&mut db) {
            // Ignore errors. Diagnose them but don't stop processing.
            let err = err.context(format!(
                "{}:{}: error processing {:?}",
                file_name.display(),
                line,
                record
            ));
            eprintln!("{:?}", err);
        }
    }

    Ok(db)
}
