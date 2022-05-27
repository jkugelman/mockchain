# Mockchain

## Design decisions

- Uses a `Decimal` type to avoid floating point accuracy issues. It can handle 4 decimal places of
  precision no problem.

- CSV records are streamed via a custom `Iterator` so as to avoid O(n) memory usage, and to allow
  for files to be read while still being written to.

- There's a large suite of unit tests. Left to my own devices I would have written integration tests
  with that use test CSV files, or that use `include_bytes!` to embed test CSV data in the Rust
  code. I didn't do that, though, because the design spec says not to commit any CSV files.

- Clients and transactions are stored in `HashMap`s. I originally used `BTreeMap`s so the records
  would be sorted by ids, leading to nicer output. The spec makes a point to specify that order
  doesn't matter, though, so I switched to `HashMap`s for those sweet O(log n) -> O(1) lookup and
  insertion efficiency gains. (They're not actually *huge* gains, but they're something.)

## Edge cases handled

- Errors are detected and printed to stderr but do not interrupt processing.

- Available funds cannot go negative from withdrawals, but they can go negative from disputes. If
  you deposit $100, withdraw $100, and then the deposit is disputed, your available funds to go
  -$100.

- The spec doesn't explicitly say what types of transactions can be disputed. Deposits can obviously
  be disputed. Can withdrawals? I'm not sure that's sensible, so I ignore disputed withdrawals.

## Edge cases not handled

- The spec doesn't say what to do when an account is locked. It says the account should immediately
  be "frozen" but doesn't specify what freezing entails. Should withdrawals be blocked? Should
  deposits? In the absence of guidance I did not implement any restrictions on locked/frozen
  accounts.

  If I encountered this on the job, I would file a bug report against the spec. "Frozen" should
  either be defined and spec'ed out; or, if it's synonymous with "locked", it should be changed to
  "locked". Synonyms should be avoided in technical documents.

- Does not check that the client id in a dispute, resolve, or chargeback record matches the ones in
  earlier records.
