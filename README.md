## Pedersen Commitment FRAME pallet

This pallet implements a pedersen commitment scheme where an `AccountId` make a cryptographic commitment and later reveal the solution to the commitment. Currently, the pallet implements a Pederson commitment with the Ristretto curve25519, but it can be augmented in the future to use other cryptographic primitives.

The extrinsic callable traits for this pallet are:

```rust
fn commit(T::AccountId, md: Sha512, rd: Sha512)
fn reveal(T::AccountId, s: &str, m: &str)
fn has_been_revealed(T::AccountID, md: Sha512) -> bool
```

The pallet storage keeps a map of commited and not revealed messages and a map with the successful revealed commitments. The committed and not revealed map also keep the number of failed attempts an `T::AccountId` tried to revela the message unsuccessfully.