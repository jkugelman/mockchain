use std::fs::File;
use std::path::PathBuf;

use anyhow::Context;
use clap::Parser;

use crate::tx::Database;

mod csv;
mod tx;

/// Command-line arguments.
#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    file_name: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let args = Args::try_parse()?;
    let file_name = args.file_name;

    let mut db = Database::new();
    let file = File::open(&file_name)
        .with_context(|| format!("{}: could not open file", file_name.display()))?;

    for (line, record) in csv::read(file).enumerate() {
        let line = line + 1;
        let record = record.with_context(|| {
            format!("{}:{}: error parsing CSV record", file_name.display(), line)
        })?;

        match record.apply(&mut db).with_context(|| {
            format!(
                "{}:{}: error processing {:?}",
                file_name.display(),
                line,
                record
            )
        }) {
            Ok(()) => {}
            Err(err) => {
                // Ignore errors. Diagnose them but don't stop processing.
                eprintln!("{:?}", err);
            }
        }
    }

    for client in db.clients.values() {
        println!("{:?}", client);
    }

    Ok(())
}
