# Recursive STARKs

Not implemented in the current AIR.

## Path

1. Prove each spend as today (Winterfell FRI).
2. Aggregate N spend proofs into one recursive proof that attests "these N proofs verified."
3. The block carries one recursive proof instead of N individual proofs.
4. The node verifies the recursive proof once.

Winterfell supports recursion via a verifier circuit. That circuit is not in this tree. The interface is the existing `verify_bytes` call; a recursive wrapper would call it inside a proof.

## Why it matters

Depth-3 note trees and per-transfer proofs do not scale. Recursion is the path to larger anonymity sets and cheaper block verification.
