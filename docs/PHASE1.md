# Phase 1 — Wire padded_path into the production AIR

**Status: not integrated. Gate not met.**

## Why it stopped

`padded_path::step` mixes with a linear combination. The production spend AIR mixes with Poseidon2 through the boundary loads and the transition. Replacing the fill with `padded_path` changes the membership computation. The depth-3 production trace hash is `17909019558313341407`. A fill that calls `padded_path::step` will not match it.

## What exists

- `padded_path.rs`: standalone selector-padded walk. Tests pass at depth 3, 4, 8. Not called by `lib.rs`.
- Production AIR: depth 3, Poseidon2 path, unchanged.

## Gate

Not met. Depth-3 production proof still passes on the existing AIR. Depth 4 and 8 are not proven through that AIR.
