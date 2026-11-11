# Phase 1 — Depth-parameterized anonymity set

## Depth 3: identity confirmed

`layout.rs` derives rows and columns from depth. For depth 3 the formulas match the previous hardcoded values:

- pad slot 7, trace length 256
- root row 127, nullifier 159, output 191, event 223
- bit column 20, sibling start 23, range start 35, width 132

The fill loop, step flags, and boundary assertions all source these from `layout.rs`. A golden trace hash (`17909019558313341407`) is unchanged, and `accepts_valid_witness_and_rejects_wrong_nullifier` passes.

## Depth 4 and 8: not yet

The layout formulas give sizes for depth 4 (trace 512, pad 8) and depth 8 (trace 512, pad 12). The AIR is still monomorphic at depth 3: the boundary array is `[E; 7]`, step flags carry `[bool; 7]`, and column widths are fixed at compile time. Proofs at depth 4 or 8 require the AIR, step flags, and column layout to be generic over depth. That is not done.

## Gate

Depth 3 is green and identity-checked. Depth 4 and depth 8 are not. Phase 2 is not started.
