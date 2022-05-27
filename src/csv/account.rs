use std::io::Write;

use rust_decimal::Decimal;
use serde::Serialize;

use crate::db::{Database, ClientId};

pub fn write(writer: impl Write, db: &Database) -> anyhow::Result<()> {
    let mut writer = csv::Writer::from_writer(writer);

    for client in db.clients.values() {
        writer.serialize(Account {
            client: client.id,
            available: client.available,
            held: client.held,
            total: client.available + client.held,
            locked: client.locked,
        })?;
    }

    Ok(())
}

#[derive(Serialize)]
struct Account {
    client: ClientId,
    available: Decimal,
    held: Decimal,
    total: Decimal,
    locked: bool,
}
