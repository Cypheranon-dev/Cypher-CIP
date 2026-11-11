# Phase 1 — Depth-parameterized anonymity set

**Status: design landed. Gate not met.**

## What is done

`crypto/cip/src/layout.rs` computes trace length, pad slot, assertion rows, and column offsets from depth:

- `slots_for(depth) = depth + 3`
- `trace_len(depth)` = next power of two covering `(slots + 1) * 32`
- `root_row(depth)` = end of last merkle slot
- `null_row`, `out_row`, `event_row` follow
- `sib_start`, `range_start`, `width` derived from depth

Tests confirm the formulas for depth 3, 4, and 8.

## What is not done

The AIR in `lib.rs` and the step flags in `poseidon2.rs` still use the depth-3 hardcoded layout. Integrating the layout formulas so that:

1. The state at `root_row(depth)` equals the merkle root,
2. Nullifier, output, and event assertions land on their computed rows,
3. Sibling and bit columns scale with depth,
4. Proof generation and verification pass at depth 3, 4, and 8,

is not finished. An earlier integration attempt moved the assertion rows without rewriting the boundary-load order; the depth-3 constraint check failed. That change was reverted so existing depth-3 proofs still pass.

## Gate

Not met. Do not start Phase 2 or Phase 3 until proofs generate and verify at depth 3, 4, and 8 against this layout.
