use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use csv::Trim;
use rust_decimal::Decimal;
use serde::Deserialize;

/// Client ID.
type ClientId = u16;

/// Transaction ID.
type TxId = u16;

/// One entry from the transaction file.
#[derive(Clone, Debug)]
enum Record {
    /// A deposit into a client's account.
    Deposit {
        client: ClientId,
        tx: TxId,
        amount: Decimal,
    },

    /// A withdrawal from a client's account.
    Withdrawal {
        client: ClientId,
        tx: TxId,
        amount: Decimal,
    },

    /// A dispute of a previous transaction. Funds are held until the dispute is resolved or charged
    /// back.
    Dispute { client: ClientId, tx: TxId },

    /// Resolves a previous dispute, lifting the hold.
    Resolve { client: ClientId, tx: TxId },

    /// Resolves a previous dispute by withdrawing held funds and freezing the client's account.
    Chargeback { client: ClientId, tx: TxId },
}

#[derive(Deserialize, Clone, Copy, Debug)]
#[serde(rename_all = "lowercase")]
enum RecordType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

/// I couldn't get `serde` to deserialize [`Record`] objects directly. According to @BurntSushi,
/// [tagged enums and CSVs don't play nicely][1]. As a workaround, I use `serde` to deserialize
/// `RawRecord`s and then turn those into `Record`s with some handwritten code.
///
/// [1]: https://github.com/BurntSushi/rust-csv/issues/211
#[derive(Deserialize, Debug)]
struct RawRecord {
    #[serde(rename = "type")]
    record_type: RecordType,
    client: ClientId,
    tx: TxId,
    amount: Option<Decimal>,
}

impl TryFrom<RawRecord> for Record {
    type Error = anyhow::Error;

    fn try_from(raw: RawRecord) -> Result<Self, Self::Error> {
        match raw.record_type {
            RecordType::Deposit => Ok(Record::Deposit {
                client: raw.client,
                tx: raw.tx,
                amount: raw.amount.ok_or(anyhow!("deposit missing amount"))?,
            }),
            RecordType::Withdrawal => Ok(Record::Withdrawal {
                client: raw.client,
                tx: raw.tx,
                amount: raw.amount.ok_or(anyhow!("withdrawal missing amount"))?,
            }),
            RecordType::Dispute => Ok(Record::Dispute {
                client: raw.client,
                tx: raw.tx,
            }),
            RecordType::Resolve => Ok(Record::Resolve {
                client: raw.client,
                tx: raw.tx,
            }),
            RecordType::Chargeback => Ok(Record::Chargeback {
                client: raw.client,
                tx: raw.tx,
            }),
        }
    }
}

/// Command-line arguments.
#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    file_name: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::try_parse()?;

    let file_name = &args.file_name;
    let mut file = csv::ReaderBuilder::new()
        .trim(Trim::All)
        .from_path(file_name)
        .with_context(|| format!("Could not open {}", file_name.display()))?;

    for record in file.deserialize() {
        let record: RawRecord = record?;
        let record: Record = record.try_into()?;
        println!("{:?}", record);
    }

    Ok(())
}
