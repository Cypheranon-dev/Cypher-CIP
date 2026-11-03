# Protocol

What the Rust crates enforce. This is the full CIP chain for the CIP-2026 research stage: FRI STARK spend, Dilithium2 user signature, Dilithium5 M-of-N envelope, Poseidon2, no trusted setup.

## Hash

Poseidon2 over Goldilocks, width 8. Digest is four lanes. Domains: note, nullifier, event, pk (`DOM_PK = 8`), aggregate (`DOM_AGG = 16`).

## Notes

Depth 3, eight leaves. A full tree opens the next. Note is `Poseidon2(amount, sk, rho, domain, ...)`. Amounts under 2^32.

## Spend

One FRI proof. Binds note root, nullifier, output, event, height, fee, chain state. Conservation is range-checked. A Dilithium2 signature covers the proof. A Dilithium5 envelope (M-of-N) covers the same bytes. The spend also commits to the Dilithium public key (`v2::pk_commitment`); the node checks that binding.

## Aggregation

`aggregate` verifies each spend and folds the publics into one Poseidon2 commitment. `check_aggregate` confirms it. A block may carry the commitment.

## Block

Equihash(48, 5). Paper (512, 9) does not construct. Difficulty cap 24 bits. Timestamps caller-supplied, strictly increasing, 1-day jump cap. Coinbase commitment is public; field `sk` is not serialized. Emission is the integer form of §12.2.

## Not in this tree

Full recursive verifier AIR. Full ML-DSA inside the STARK. Public P2P. Audit.
