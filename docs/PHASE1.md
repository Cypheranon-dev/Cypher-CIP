# Phase 1 — Fixed MAX_DEPTH circuit

**Status: design decided. Gate not met.**

## Decision

Option B. One shared note tree, one circuit, one verifying key. `MAX_DEPTH = 20`. Shallower spends pad unused levels with a selector column. Depth 3, 4, and 8 are the same AIR with different amounts of padding.

## Layout

`layout.rs` computes sizes for MAX_DEPTH = 20:

- pad slot 24, trace length 1024
- root / nullifier / output / event rows follow the max-depth slots
- sibling and range columns scale to 20 levels
- selector column and flag column follow

## Remaining integration

The AIR in `lib.rs` and the step flags in `poseidon2.rs` are still the depth-3 circuit. Wiring MAX_DEPTH through requires, in order:

1. Size the boundary flag array and the periodic boundary columns to the pad slot.
2. Size sibling and bit columns to MAX_DEPTH.
3. Add the selector column. Selector 1 mixes the sibling; selector 0 is the identity on the accumulator.
4. Constrain that selectors are 1 for the first `d` levels and 0 after, in order, where `d` is public.
5. Confirm a depth-3 witness still verifies on the padded circuit.
6. Confirm depth-4 and depth-8 witnesses verify on the same circuit.

An integration attempt that resized the boundary array without resizing the periodic columns panicked. It was reverted so depth-3 proofs still pass.

## Gate

Not met. Phase 2 comparison is written separately and does not depend on this gate.
