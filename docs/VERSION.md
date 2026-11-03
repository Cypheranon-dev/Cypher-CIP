# Version

**Current build: CIP-2026.6** (November 2026)

## Recursive aggregation

`crypto/cip/src/aggregate.rs` is the aggregation layer.

- `aggregate` verifies each spend proof, then folds the public digests into one Poseidon2 commitment (`DOM_AGG = 16`).
- `commit_publics` recomputes the commitment from verified publics.
- Empty sets and failed proofs are rejected.
- A block can carry one aggregate commitment instead of N separate checks at verify time.
- Full Winterfell verifier-in-a-circuit recursion remains the later AIR. This is the working aggregation the node uses now.

## Chain plan (unchanged)

- FRI STARK transfer, no trusted setup
- Dilithium5 M-of-N envelope, Dilithium2 user signature
- Poseidon2
- Finite emission
- Equihash(48, 5) as the legal instance

## Previous

- CIP-2026.5: M-of-N Dilithium5 envelope
- CIP-2026.4: degree fix, full Dilithium2 key, 1-day timestamp cap
- CIP-2026.3: circuit v2 key binding
