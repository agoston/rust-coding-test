# How to run

```shell
cargo run -- transactions.csv
```

# Design notes

Design is covered in the spec, this document merely extends on it.

## Requirements

Specs are well written, but still miss some important questions:

* if balance may go below 0. The provided example didn't allow it, so I also added safeguard, but it's unclear.
* if negative transaction amount should be allowed. I decided not to and added a safeguard in `Ledger`.
* if a new client should be added if the transaction is refused/bogus. I opted for not to save memory. 
* which of the dispute/resolve/chargeback transaction can be executed on which transactions. For now I've decided not to implement a state machine, as there were a lot of questions I couldn't decide (e.g. can you dispute a withdrawal?) 
* what to do with `locked` clients; I've added a safeguard in `Ledger` to ignore transactions of locked clients.

I added FIXMEs where I believe a maintenance debt was left behind. 

## Performance

I used the readily available streaming option to read input file, thus the input file size itself should pose no issues.
Only `deposit` and `withdrawal` transactions are kept in memory, so that the 'dispute' transaction types can refer to them.
Client data is kept in a `HashMap`, but clients are only stored if they had at least 1 successful transaction.

## Security

I only used basic dependencies. I opted for deser for its ease of use, but ultimately for a code of this complexity, manual parsing would also be fine.

**The code currently trusts input at too many places for production use**. I've added a FIXME for those, should come back once the spec is updated to cover for the design questions above.

## Threading

The code is now single-threaded. For multi-thread support (e.g. in a REST service), mutations in the `transactions` and `clients` hashmaps will collide.
To fix that, we can either access `Ledger` via `Arc<Ledger>` (cheap qua development time), or for a more sophisticated approach,
refactor `transactions` to an append-only `Vec`, with monotonous `id` in it for quick binary searches.
For `clients`, we can switch to concurrent HashMap implementation that shards the keys into sub-hashmaps, only locking a small portion of the data set on mutations.

## Testing

I've added average test coverage to all code to prove it working for the happy case scenarios, and also covered the corner cases.
Test coverage for features with many question marks is lower to accommodate for expected changes. Should be reviewed/extended before deploy.

## Safety

Now the code is effectively a single-node, single-run, in-memory solution. To protect data, we need an on-disk database, multiple nodes, load balancing, failover, logging, metrics, monitoring and of course backups.

## Efficiency

I've opted to make my own monetary type, tailor-built for this purpose (fixed point, up to 4 digits).
This requires less storage space, while making operations much faster than a regular, arbitrary precision lib.

For transactions and clients, I used the most effective ways possible to make sure it stays performant.
I also opted to process input in a streaming fashion, to allow for effectively unlimited input, as long as enough memory is provided for transaction and client store.\
If there was a need to scale, I'd consider a redis-like shared storage, or some other shardable nosql database.

## Code cleanliness

Maintainability is a big factor. I'm a firm believer that 80%+ of all development costs is in maintenance work, so clean, concise design and readable code is just as important as performance and security.

I've dissected the code into independent, reusable and testable modules. I've used serde to ease extendability and maintenance, while reducing features to test.
I've made method signatures re-usable for other purposes, even if it meant some added complexity.

And all the while, testing made sure that the 'eat your own dogfood' principle is applied.