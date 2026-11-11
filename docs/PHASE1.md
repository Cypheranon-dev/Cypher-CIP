# Phase 1 — Fixed MAX_DEPTH circuit

**Status: membership gadget complete. Full AIR integration pending.**

## Structure

One circuit shape. `MAX_DEPTH = 20`. A spend of a shallower tree pads unused levels with a selector. Real levels mix a sibling. Padding levels are the identity on the accumulator. Selectors must be 1 then 0, in order.

## Implemented

`crypto/cip/src/padded_path.rs`:

- `step` applies one level under the selector.
- `walk` checks selector order and returns the root.
- `pad` builds a MAX_DEPTH path from a shorter real path.
- Tests pass for depth 3, 4, and 8, and reject an unordered selector.

## Production AIR

The spend AIR in `lib.rs` is still the depth-3 circuit. It is unchanged and still proves. Replacing its Merkle path with `padded_path` is the remaining integration. Until that lands, the anonymity set in consensus is 8 leaves.

## Gate

Membership gadget: met. Consensus circuit: not switched. Do not claim a 20-level anonymity set in production until the AIR uses this path and a depth-3 proof still verifies.
