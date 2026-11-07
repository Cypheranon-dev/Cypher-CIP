# Anonymity set redesign

**Status: design only. Current tree is depth 3, 8 leaves. This is a mainnet blocker.**

Capacity is not effective anonymity. A live set of one note in a depth-20 tree is still a set of one.

## Why a constant increase is not enough

`boundaries()` in `crypto/cip/src/lib.rs` uses a fixed array of 7 and overwrites slots 3–5 after the depth loop. Those slots hold the nullifier, output, and event loads. Raising `DEPTH` without restructuring those slots produces a proof that does not match its public inputs. A depth-4 attempt compiled the constants and then failed proof generation for that reason. It was reverted.

## Required design before implementation

1. Target depth and capacity. A research target is depth 20 (about one million leaves). A first step is depth 10 (1,024 leaves).
2. Trace layout. Sibling columns are `DEPTH * 4`. Path bits are `DEPTH`. Both must be placed so range columns and the coinbase flag do not overlap.
3. Boundary model. Separate merkle path boundaries from nullifier, output, and event boundaries. Do not overwrite path slots.
4. Root history. Define which historical roots are accepted for a spend, and how many are retained.
5. Rollover. Define when a full tree opens the next one, and how the old root remains spendable.
6. Reorg reconstruction. Rebuilding the note trees from a fork must be deterministic.
7. Versioning. A depth change is a protocol version change. Old proofs must not verify under the new parameters.
8. Privacy claim. State that effective anonymity is the number of indistinguishable live notes, not the tree capacity.

## Acceptance tests for a future implementation

- Valid spend at the new depth opens and verifies.
- A leaf that is not the note is rejected.
- A wrong path bit is rejected.
- Historical root is accepted only within the documented window.
- Reorg reconstructs the same roots.
- Proof time and size are measured and recorded.
