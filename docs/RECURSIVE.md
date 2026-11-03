# Recursive STARKs

## Now

`crypto/cip/src/aggregate.rs` verifies each spend proof and folds the verified public digests into one Poseidon2 commitment (`DOM_AGG = 16`).

- `aggregate(&[SpendRef])` rejects an empty set and any proof that fails `verify_bytes`.
- `commit_publics` recomputes the commitment.
- A block can carry the aggregate commitment. The node checks it once.

## Later

A Winterfell verifier circuit that proves "these N proofs verified" inside one STARK. That AIR is not in this tree. The aggregation layer is the interface it will replace.
