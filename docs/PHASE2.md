# Phase 2 — Fold AIR

**Status: absorb circuit, not a FRI verifier.**

## What it does

`recursive.rs` re-executes the same absorb as `aggregate::fold`:

- Four slots, 32 rows each. A periodic flag selects the Poseidon2 round. The boundary row is not multiplied by a trace selector, so the degree stays compatible with blowup 8.
- Assertions bind the commitment, the count, and a row-0 anchor so the domain separator is enforced.
- `prove_fold` / `verify_fold_proof` generate and check a Winterfell proof of that absorb.
- `verify_fold` checks the same absorb natively, including the digest list. The list is not a public input of the proof.

## What it does not do

It does not re-execute FRI or attest that the digests came from valid spend proofs. The node still calls `verify_bytes` on each spend. Reordering digests changes the commitment. The absorb uses the first two lanes of each digest, matching `aggregate::fold`.

## Tests

- `fold_roundtrip_and_reorder` — Poseidon2 fold matches, reorder fails.
- `trace_matches_absorb` — the trace commitment matches the native absorb.
- `prove_and_verify_fold` — Winterfell proof generates and verifies.
