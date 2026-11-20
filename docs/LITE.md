# Lite membership circuit

**Status: standalone. Not the production spend.**

A degree-2 Winterfell AIR over eight levels (`crypto/cip/src/lite.rs`).

What it proves:

- each path bit and selector is boolean
- a selector that is 0 stays 0
- `mixed = bit * sibling + (1 - bit) * acc`
- `next = real * mixed + (1 - real) * acc`
- the public leaf, the final accumulator, and the number of real levels match the trace

What it does not prove:

- Poseidon2. The mix is linear. It is not collision-resistant.
- The production spend. Depth on that path is still 3.
- An anonymity set. Eight routing slots are not a note tree.
- ML-DSA, a recursive verifier, or public gossip.

Degree is 2, so blowup 8 (the protocol floor set by the unsplit \(x^7\) S-box) still holds. This AIR does not raise it.

The production spend stays on Poseidon2. This module is the first checked routing shape, not a replacement for it.
