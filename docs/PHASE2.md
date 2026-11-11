# Phase 2 — Recursion

**Status: path chosen. Interim is the Poseidon2 fold. No verifier circuit.**

## Comparison

| Path | Cost | Outcome |
| --- | --- | --- |
| Port AIR to Plonky2/3 | Full rewrite, new stack, discard Winterfell proofs and pinned options | Recursion becomes a library feature |
| Custom Winterfell verifier AIR | Hundreds of thousands of constraints, FRI relation must be exact | Stays on current stack |

## Decision

Stay on Winterfell. Recursion is not a launch requirement until an owner says otherwise. The interim is:

1. Verify each spend with `verify_bytes`.
2. Fold the verified digests with domain-separated Poseidon2 (`aggregate.rs`).
3. Do not call the fold a recursive proof.

A Plonky port is justified only if recursion becomes required and a Winterfell verifier AIR is estimated to cost more than the migration. That estimate does not exist.

## Prototype

The fold in `aggregate.rs` is the prototype. It rejects an empty set, rejects a bad spend, and changes when digests are reordered. It does not re-execute FRI.

## Gate

Comparison written. Path chosen. No further recursion work is scheduled.
