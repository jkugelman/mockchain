use std::collections::HashMap;

use anyhow::{anyhow, ensure, Context};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// Client ID.
pub type ClientId = u16;

/// Transaction ID.
pub type TxId = u32;

/// A client's funds and account status.
#[derive(Debug)]
pub struct Client {
    pub id: ClientId,
    pub available: Decimal,
    pub held: Decimal,
    pub locked: bool,
}

impl Client {
    pub fn new(id: ClientId) -> Self {
        Client {
            id,
            available: dec!(0),
            held: dec!(0),
            locked: false,
        }
    }

    pub fn deposit(&mut self, amount: Decimal) -> anyhow::Result<()> {
        ensure!(amount >= dec!(0), "negative deposit: {}", amount);
        self.available += amount;
        Ok(())
    }

    pub fn withdraw(&mut self, amount: Decimal) -> anyhow::Result<()> {
        ensure!(amount >= dec!(0), "negative withdrawal: {}", amount);
        ensure!(
            amount <= self.available,
            "cannot withdraw {}, only {} available",
            amount,
            self.available
        );
        self.available -= amount;
        Ok(())
    }

    pub fn hold(&mut self, amount: Decimal) -> anyhow::Result<()> {
        ensure!(amount >= dec!(0), "negative hold: {}", amount);
        self.available -= amount;
        self.held += amount;
        Ok(())
    }

    pub fn release(&mut self, amount: Decimal) -> anyhow::Result<()> {
        ensure!(amount >= dec!(0), "negative release: {}", amount);
        ensure!(
            amount <= self.held,
            "cannot release {}, only {} held",
            amount,
            self.held
        );
        self.available += amount;
        self.held -= amount;
        Ok(())
    }

    pub fn chargeback(&mut self, amount: Decimal) -> anyhow::Result<()> {
        ensure!(amount >= dec!(0), "negative chargeback: {}", amount);
        ensure!(
            amount <= self.held,
            "cannot chargeback {}, only {} held",
            amount,
            self.held
        );
        self.held -= amount;
        self.locked = true;
        Ok(())
    }
}

/// A deposit or withdrawal. A positive `amount` is a deposit, negative a withdrawal.
#[derive(Debug)]
pub struct Tx {
    pub id: TxId,
    pub amount: Decimal,
}

impl Tx {
    pub fn new(id: TxId, amount: Decimal) -> Self {
        Tx { id, amount }
    }
}

pub struct Database {
    pub clients: HashMap<ClientId, Client>,
    pub txs: HashMap<TxId, Tx>,
}

impl Database {
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
            txs: HashMap::new(),
        }
    }

    pub fn deposit(
        &mut self,
        client_id: ClientId,
        tx_id: TxId,
        amount: Decimal,
    ) -> anyhow::Result<()> {
        ensure!(
            !self.txs.contains_key(&tx_id),
            "duplicate transaction id {}",
            tx_id
        );
        let client = self.client(client_id);
        client
            .deposit(amount)
            .with_context(|| format!("failed deposit with {:?}", client))?;
        self.txs.insert(tx_id, Tx::new(tx_id, amount));
        Ok(())
    }

    pub fn withdraw(
        &mut self,
        client_id: ClientId,
        tx_id: TxId,
        amount: Decimal,
    ) -> anyhow::Result<()> {
        ensure!(
            !self.txs.contains_key(&tx_id),
            "duplicate transaction id {}",
            tx_id
        );
        let client = self.client(client_id);
        client
            .withdraw(amount)
            .with_context(|| format!("failed withdrawal with {:?}", client))?;
        self.txs.insert(tx_id, Tx::new(tx_id, -amount));
        Ok(())
    }

    pub fn dispute(&mut self, client_id: ClientId, tx_id: TxId) -> anyhow::Result<()> {
        let (client, tx) = self.lookup(client_id, tx_id)?;
        ensure!(tx.amount >= dec!(0), "cannot dispute a withdrawal");
        client
            .hold(tx.amount)
            .with_context(|| format!("failed dispute with {:?}", client))
    }

    pub fn resolve(&mut self, client_id: ClientId, tx_id: TxId) -> anyhow::Result<()> {
        let (client, tx) = self.lookup(client_id, tx_id)?;
        ensure!(tx.amount >= dec!(0), "cannot resolve a withdrawal");
        client
            .release(tx.amount)
            .with_context(|| format!("failed resolve with {:?}", client))
    }

    pub fn chargeback(&mut self, client_id: ClientId, tx_id: TxId) -> anyhow::Result<()> {
        let (client, tx) = self.lookup(client_id, tx_id)?;
        ensure!(tx.amount >= dec!(0), "cannot chargeback a withdrawal");
        client
            .chargeback(tx.amount)
            .with_context(|| format!("failed chargeback with {:?}", client))
    }

    /// Look up an existing client, or create a new one.
    fn client(&mut self, id: ClientId) -> &mut Client {
        self.clients.entry(id).or_insert_with(|| Client::new(id))
    }

    /// We need to lookup the client and tx at the same time in order to split the borrow of `&mut
    /// self` into borrows of two sub-fields.
    fn lookup(&mut self, client_id: ClientId, tx_id: TxId) -> anyhow::Result<(&mut Client, &Tx)> {
        Ok((
            self.clients
                .get_mut(&client_id)
                .ok_or_else(|| anyhow!("no such client {}", client_id))?,
            self.txs
                .get(&tx_id)
                .ok_or_else(|| anyhow!("no such tx {}", tx_id))?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;

    use super::*;

    #[test]
    fn deposit_withdraw() {
        let mut db = Database::new();

        db.deposit(1, 1, dec!(100)).unwrap();
        assert_funds(&db, 1, dec!(100), dec!(0));

        db.deposit(1, 2, dec!(20)).unwrap();
        assert_funds(&db, 1, dec!(120), dec!(0));

        db.deposit(1, 3, dec!(3)).unwrap();
        assert_funds(&db, 1, dec!(123), dec!(0));

        db.withdraw(1, 4, dec!(100)).unwrap();
        assert_funds(&db, 1, dec!(23), dec!(0));

        db.withdraw(1, 5, dec!(20)).unwrap();
        assert_funds(&db, 1, dec!(3), dec!(0));

        assert!(db.withdraw(1, 6, dec!(444)).is_err());
        assert_funds(&db, 1, dec!(3), dec!(0));
    }

    #[test]
    fn duplicate_tx_ids() {
        let mut db = Database::new();

        db.deposit(1, 1, dec!(100)).unwrap();
        assert!(db.deposit(2, 1, dec!(20)).is_err());
    }

    #[test]
    fn multiple_clients() {
        let mut db = Database::new();

        db.deposit(3, 30, dec!(300)).unwrap();
        db.deposit(2, 20, dec!(200)).unwrap();
        db.deposit(10, 1, dec!(100)).unwrap();
        db.withdraw(2, 21, dec!(20)).unwrap();
        db.withdraw(10, 2, dec!(10)).unwrap();
        db.withdraw(3, 3, dec!(30)).unwrap();

        assert_funds(&db, 10, dec!(90), dec!(0));
        assert_funds(&db, 2, dec!(180), dec!(0));
        assert_funds(&db, 3, dec!(270), dec!(0));
    }

    #[test]
    fn dispute_resolve_chargeback() {
        let mut db = Database::new();

        db.deposit(1, 1, dec!(100)).unwrap();
        db.deposit(1, 2, dec!(50)).unwrap();

        db.dispute(1, 1).unwrap();
        assert_funds_locked(&db, 1, dec!(50), dec!(100), false);

        db.resolve(1, 1).unwrap();
        assert_funds_locked(&db, 1, dec!(150), dec!(0), false);

        db.dispute(1, 1).unwrap();
        assert_funds_locked(&db, 1, dec!(50), dec!(100), false);

        db.chargeback(1, 1).unwrap();
        assert_funds_locked(&db, 1, dec!(50), dec!(0), true);
    }

    #[test]
    fn cannot_dispute_withdrawals() {
        let mut db = Database::new();

        db.deposit(1, 1, dec!(100)).unwrap();
        db.withdraw(1, 2, dec!(60)).unwrap();

        assert!(db.dispute(1, 2).is_err());
        assert!(db.resolve(1, 2).is_err());
        assert!(db.chargeback(1, 2).is_err());

        assert_funds_locked(&db, 1, dec!(40), dec!(0), false);
    }

    fn assert_funds(db: &Database, client_id: ClientId, available: Decimal, held: Decimal) {
        let client = db.clients.get(&client_id).unwrap();
        assert_eq!(client.available, available);
        assert_eq!(client.held, held);
    }

    fn assert_funds_locked(
        db: &Database,
        client_id: ClientId,
        available: Decimal,
        held: Decimal,
        locked: bool,
    ) {
        assert_funds(db, client_id, available, held);
        let client = db.clients.get(&client_id).unwrap();
        assert_eq!(client.locked, locked);
    }
}
