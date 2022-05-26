use std::collections::BTreeMap;

use anyhow::bail;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// Client ID.
pub type ClientId = u16;

/// Transaction ID.
pub type TxId = u16;

/// One entry from the transaction file.
///
/// # Note
///
/// Currency values are stored as `Decimal`s. Floating point numbers are a bad idea for currency due
/// the errors introduced by their base-2 representation. A float can store `0.50` exactly but not
/// `0.20`, for example.
///
/// Integers could be used if we stored cents instead of dollars. That would be awkward in this
/// program, though, since the specification requires 4 decimal places of precision rather than 2.
/// That also wouldn't translate well to other currencies. Not every currency is divisible into
/// hundredths.
#[derive(Debug)]
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

/// A client's funds and account status.
pub struct Client {
    pub id: ClientId,
    pub available: Decimal,
    pub held: Decimal,
    pub locked: bool,
}

impl Client {
    pub fn new(id: ClientId) -> Client {
        Client {
            id,
            available: dec!(0),
            held: dec!(0),
            locked: false,
        }
    }

    fn credit(&mut self, amount: Decimal) -> anyhow::Result<()> {
        assert!(amount >= dec!(0));
        self.available += amount;
        Ok(())
    }

    fn debit(&mut self, amount: Decimal) -> anyhow::Result<()> {
        assert!(amount >= dec!(0));
        if amount > self.available {
            bail!("cannot withdraw {}, only {} available", amount, self.available);
        }
        self.available -= amount;
        Ok(())
    }
}

pub struct Tx {
    pub id: TxId,
    pub amount: Decimal,
}

impl Tx {
    pub fn new(id: TxId, amount: Decimal) -> Tx {
        Tx { id, amount }
    }
}

/// Process a series of transaction records and return the resultant list of clients, their
/// balances, and their lock status.
///
/// This function purposefully takes a vector of owned `Record`s (`Vec<Record>`) rather than a
/// borrowed slice (`&[Record]`) in order to "consume" the records. This prevents them from being
/// reused after being processed.
pub fn process(records: Vec<Record>) -> anyhow::Result<BTreeMap<ClientId, Client>> {
    let mut clients = BTreeMap::new();
    let mut txs = BTreeMap::new();

    for record in records {
        match record {
            Record::Deposit {
                client: client_id,
                tx: tx_id,
                amount,
            } => {
                if amount < dec!(0) {
                    eprintln!("ignoring negative deposit: {}", amount);
                    continue;
                }

                let client = clients
                    .entry(client_id)
                    .or_insert_with(|| Client::new(client_id));
                let _ = client.credit(amount);

                let tx = Tx::new(tx_id, amount);
                if txs.insert(tx_id, tx).is_some() {
                    bail!("duplicate transaction id {}", tx_id);
                }
            }

            Record::Withdrawal {
                client: client_id,
                tx: tx_id,
                amount,
            } => {
                if amount < dec!(0) {
                    eprintln!("ignoring negative withdrawal: {}", amount);
                    continue;
                }

                let client = clients
                    .entry(client_id)
                    .or_insert_with(|| Client::new(client_id));
                let _ = client.debit(amount);

                let tx = Tx::new(tx_id, -amount);
                if txs.insert(tx_id, tx).is_some() {
                    bail!("duplicate transaction id {}", tx_id);
                }
            }

            Record::Dispute { client, tx } => todo!(),
            Record::Resolve { client, tx } => todo!(),
            Record::Chargeback { client, tx } => todo!(),
        }
    }

    Ok(clients)
}

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;

    use super::*;

    #[test]
    fn deposit() {
        let records = vec![
            Record::Deposit {
                client: 1,
                tx: 1,
                amount: dec!(100),
            },
            Record::Deposit {
                client: 1,
                tx: 2,
                amount: dec!(20),
            },
            Record::Deposit {
                client: 1,
                tx: 3,
                amount: dec!(3),
            },
        ];
        let clients = process(records).unwrap();
        assert_eq!(clients.len(), 1);
        let client = clients.get(&1).unwrap();
        assert_eq!(client.id, 1);

        assert_eq!(client.available, dec!(123));
        assert_eq!(client.held, dec!(0));
    }

    #[test]
    fn duplicate_tx_ids() {
        let records = vec![
            Record::Deposit {
                client: 1,
                tx: 1,
                amount: dec!(100),
            },
            Record::Deposit {
                client: 1,
                tx: 1,
                amount: dec!(20),
            },
            Record::Deposit {
                client: 1,
                tx: 1,
                amount: dec!(3),
            },
        ];
        assert!(matches!(process(records), Err(_)));
    }

    #[test]
    fn withdrawal() {
        let records = vec![
            Record::Deposit {
                client: 1,
                tx: 1,
                amount: dec!(100),
            },
            Record::Withdrawal {
                client: 1,
                tx: 2,
                amount: dec!(20),
            },
            Record::Withdrawal {
                client: 1,
                tx: 3,
                amount: dec!(3),
            },
        ];
        let clients = process(records).unwrap();
        assert_eq!(clients.len(), 1);
        let client = clients.get(&1).unwrap();

        assert_eq!(client.available, dec!(77));
        assert_eq!(client.held, dec!(0));
    }

    #[test]
    fn over_withdrawal_ignored() {
        let records = vec![
            Record::Deposit {
                client: 1,
                tx: 1,
                amount: dec!(100),
            },
            Record::Withdrawal {
                client: 1,
                tx: 2,
                amount: dec!(60),
            },
            Record::Withdrawal {
                client: 1,
                tx: 3,
                amount: dec!(80),
            },
        ];
        let clients = process(records).unwrap();
        assert_eq!(clients.len(), 1);
        let client = clients.get(&1).unwrap();

        assert_eq!(client.available, dec!(40));
        assert_eq!(client.held, dec!(0));
    }
}
