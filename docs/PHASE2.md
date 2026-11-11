# Phase 2 — Fold AIR

**Status: working. Not a FRI verifier.**

## What it does

`recursive.rs` proves an ordered fold:

- Trace holds the accumulator, the current digest, and a count.
- Assertions bind the final commitment and the count.
- `prove_fold` / `verify_fold_proof` generate and check a Winterfell proof.
- `verify_fold` checks the Poseidon2 absorb natively. The node calls this before accepting the proof.

## What it does not do

It does not re-execute FRI or attest that the digests came from valid spend proofs. The node still calls `verify_bytes` on each spend. Reordering digests changes the commitment.

## Tests

- `fold_roundtrip_and_reorder` — Poseidon2 fold matches, reorder fails.
- `prove_and_verify_fold` — Winterfell proof generates and verifies.
