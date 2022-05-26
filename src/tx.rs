use std::path::Path;

use anyhow::{anyhow, Context, Result};
use csv::Trim;
use rust_decimal::Decimal;
use serde::Deserialize;

/// Client ID.
pub type ClientId = u16;

/// Transaction ID.
pub type TxId = u16;

/// One entry from the transaction file.
///
/// Currency values are stored as `Decimal`s. Floating point numbers are a bad idea for currency due
/// the errors introduced by their base-2 representation. A float can store `0.50` exactly but not
/// `0.20`, for example.
///
/// Integers could be used if we stored cents instead of dollars. That would be awkward in this
/// program, though, since the specification requires 4 decimal places of precision rather than 2.
/// That also wouldn't translate well to other currencies. Not every currency is divisible into
/// hundredths.
#[derive(Clone, Debug)]
pub enum Record {
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

/// I couldn't get `serde` to deserialize [`Record`] objects directly. According to @BurntSushi,
/// [tagged enums and CSVs don't play nicely][1]. As a workaround, I use `serde` to deserialize
/// `RawRecord`s and then turn those into `Record`s with some handwritten code.
///
/// [1]: https://github.com/BurntSushi/rust-csv/issues/211
#[derive(Deserialize, Debug)]
struct RawRecord {
    #[serde(rename = "type")]
    record_type: RawRecordType,
    client: ClientId,
    tx: TxId,
    amount: Option<Decimal>,
}

#[derive(Deserialize, Clone, Copy, Debug)]
#[serde(rename_all = "lowercase")]
enum RawRecordType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

impl TryFrom<RawRecord> for Record {
    type Error = anyhow::Error;

    fn try_from(raw: RawRecord) -> Result<Self, Self::Error> {
        match raw.record_type {
            RawRecordType::Deposit => Ok(Record::Deposit {
                client: raw.client,
                tx: raw.tx,
                amount: raw.amount.ok_or(anyhow!("deposit missing amount"))?,
            }),
            RawRecordType::Withdrawal => Ok(Record::Withdrawal {
                client: raw.client,
                tx: raw.tx,
                amount: raw.amount.ok_or(anyhow!("withdrawal missing amount"))?,
            }),
            RawRecordType::Dispute => Ok(Record::Dispute {
                client: raw.client,
                tx: raw.tx,
            }),
            RawRecordType::Resolve => Ok(Record::Resolve {
                client: raw.client,
                tx: raw.tx,
            }),
            RawRecordType::Chargeback => Ok(Record::Chargeback {
                client: raw.client,
                tx: raw.tx,
            }),
        }
    }
}

/// Returns a `Vec` containing all of the transaction records in the named file.
//
// From a scalability standpoint, reading the entire file into memory isn't ideal. It requires O(n)
// memory usage and doesn't let the caller read the entries in a streaming fashion, one line at a
// time. That could be a problem if the CSV files could be 100s of MBs or more.
//
// I considered returning a live iterator, something like `-> Result<impl Iterator<Item =
// Result<Record>>>`. This would be a more "optimal" interface, with `Result`s attached to both the
// file as a whole as well as each individual line.
//
// The tradeoff is complexity. The iterator version was getting hairier than I liked, and I didn't
// want to blow my "complexity budget". Since this is just a take home test I decided not to go
// crazy and just return a `Result<Vec<_>>`, with the acknowledgement that it's a compromise.
pub fn read_records(path: impl AsRef<Path>) -> Result<Vec<Record>> {
    let path = path.as_ref();
    let mut file = csv::ReaderBuilder::new()
        .trim(Trim::All)
        .from_path(path)
        .with_context(|| format!("Could not open {}", path.display()))?;

    let mut records = Vec::new();
    for raw_record in file.deserialize() {
        let raw_record: RawRecord = raw_record?;
        let record = raw_record.try_into()?;
        records.push(record);
    }
    Ok(records)
}