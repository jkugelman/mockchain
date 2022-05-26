# Mockchain

## Design decisions

- Uses a `Decimal` type to avoid floating point accuracy issues. It can handle 4 decimal places of
  precision no problem.

- CSV records are streamed via a custom `Iterator` so as to avoid O(n) memory usage, and to allow
  for files to be read at the same time they are being written.

- There's a large suite of unit tests. Left to my own devices I would have condensed them quite a
  bit by referencing test CSV files, or by using `include_bytes!` to embed test CSV data in the Rust
  code. I didn't do that, though, because the design spec says not to commit any CSV files.

- Clients and transactions are stored in `HashMap`s. I originally used `BTreeMap`s so the records
  would be sorted by ids, leading to nicer output. The spec makes a point to specify that order
  doesn't matter, though, so I switched to `HashMap`s for those sweet O(log n) -> O(1) lookup and
  insertion efficiency gains. (They're not actually *huge* gains, but they're something.)

- The spec doesn't say what to do when an account is locked. It says the account should immediately
  be "frozen" but doesn't specify what freezing entails. Should withdrawals be blocked? Should
  deposits? In the absence of guidance I did not implement any restrictions on locked/frozen
  accounts.

  If I encountered this on the job, I would file a bug report against the spec. "Frozen" should
  either be defined and spec'ed out; or, if it's synonymous with "locked", it should be changed to
  "locked". Synonyms should be avoided in technical documents.

## Edge cases handled

- Most errors are ignored as that seems to be the general philosophy of the design spec. The only
  error that is a hard failure is having a duplicate transaction id.

- Available funds cannot go negative from withdrawals, but they can go negative from disputes. If
  you deposit $100, withdraw $100, and then the deposit is disputed, your available funds to go
  -$100.

- Ignores resolves when there are insufficient held funds. Held funds cannot go negative.

  This will only happen if there are erroneous resolves that resolve the same disputed transaction
  more than once, or that resolve an undisputed transaction. The spec does not describe how to
  handle such cases and so this program does not prevent them.

  Doing so would require storing additional mutable state about past transactions such as an
  `is_disputed` flag. Or if transactions are considered immutable and the transaction log is
  append-only, it would necessitate an extra layer of bookkeeping to keep track of associations
  between journal records.

## Edge cases not handled

- Does not check that the client id in a dispute, resolve, or chargeback record matches the ones in
  earlier records.

- Does not check that resolves and chargebacks point to disputed transactions.

- Does not prevent multiple disputes, resolves, or chargebacks of the same transaction.

- There's no way to set up some starting state, or process transactions against an existing
  database. The program always starts from scratch with an empty set of users and transactions.
  This could be remedied by adding methods to the public API, but I didn't spend time on that since
  the CLI front end doesn't require such functionality.
