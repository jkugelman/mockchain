use rust_decimal::Decimal;

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
