## Pedersen Commitment FRAME pallet

This pallet implements a pedersen commitment scheme where an `AccountId` make a cryptographic commitment and later reveal the solution to the commitment. Currently, the pallet implements a Pederson commitment with the Ristretto curve25519, but it can be augmented in the future to use other cryptographic primitives.

The pallet storage keeps a map of commited and not revealed messages and a map with the successful revealed commitments.
