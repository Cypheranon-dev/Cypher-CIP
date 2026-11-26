# Depth-4 membership design

**Status: design. Production `CipAir` is still depth 3 (8 leaves).**

The production spend already constrains a depth-3 Merkle path. `boundaries()` in [crypto/cip/src/lib.rs](../crypto/cip/src/lib.rs) loads each sibling into the Poseidon2 state, and the next slot permutes it with `parent()`. That is not `padded_path::walk`. The padded module is a degree-1 mux (`bit * sibling + (1 - bit) * acc`). Substituting it for `parent()` would remove the hash. It stays a standalone gadget.

## Why depth is not flipped in this change

A fourth level needs one more 32-row Poseidon slot before the nullifier, output, and event slots. The spend trace is 256 rows (8 slots):

| Slot | Depth 3 (current) | Depth 4 (not shipped) |
| --- | --- | --- |
| 0 | note preimage | note preimage |
| 1 | merkle 0 | merkle 0 |
| 2 | merkle 1 | merkle 1 |
| 3 | merkle 2, root asserted at row 127 | merkle 2 |
| 4 | nullifier | merkle 3, root asserted at row 159 |
| 5 | output | nullifier |
| 6 | event | output |
| 7 | pad | event (no spare pad slot) |

Shipping that schedule also changes `LEAVES` from 8 to 16, the witness array, genesis trees, and every existing proof. `layout::MAX_DEPTH` (20) is not the production width. Nullifier, output, event, coinbase, and range columns stay where they are until that schedule is implemented and the depth-3 vectors still verify or are explicitly versioned.

## Required before raising depth

- New assertion rows for root, nullifier, output, and event, with selectors that cannot overwrite those digests.
- Path bits still boolean. Siblings stay constant across the trace.
- Tests: every leaf index, wrong sibling, wrong bit, non-member leaf, reordered padding selectors, unchanged range and coinbase checks, prove and verify.
- Historical-root window, rollover, nullifier replay across reorgs, and migration. Not specified as consensus yet.
- Proving cost measured on the new trace length before any testnet parameter change.

Until those land, anonymity capacity is 8 leaves. Live-note usage, not tree capacity, is the anonymity set.
