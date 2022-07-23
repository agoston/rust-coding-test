# Design

Design is covered in the spec.

# How to run

```shell
cargo run -- transactions.csv
```

# Design notes

* Sanity checks in operations, guarding against balance going below 0 are missing. Left a couple of FIXMEs for that; it's not clear if we want this or not (some banks allow temporarily going below 0).
* Specs do not cover if negative transaction amount should be allowed. I decided not to and added a safeguard in `Ledger`.
* Specs do not cover if a client should be added even if the transaction is refused/bogus. I took the liberty to keep a client with 0 balance if transaction failed. So I made the choice to do create a zero-balance client if the transaction was otherwise correctly formatted (no negative amounts and referenced transactions are valid).
* I added FIXMEs where I believe a maintenance debt was left behind. 
