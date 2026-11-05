# Missing capabilities

## A. Full ML-DSA verification inside the STARK

**Status: design only. Not implemented.**

The node verifies Dilithium with FIPS 204 outside the proof and binds the public key into the STARK statement via the payload. That is not in-circuit signature verification.

A real circuit would constrain matrix expansion, NTT/INTT, the challenge polynomial, hint decompression, and infinity-norm checks. Estimated size for Dilithium2 is several hundred thousand constraints. Until that AIR exists and is tested against FIPS 204 known-answer vectors, the claim remains "signature checked outside the proof."

## B. Recursive verifier circuit

**Status: partial. Fold is constrained. Verifier is not.**

`RecursiveAir` constrains `next_acc = cur_acc + digest` and a monotonic count. It does not re-execute FRI or attest that the digests came from valid spend proofs. The node must call `verify_bytes` on each spend before folding.

## C. Public gossip network

**Status: partial.**

Peers exchange hellos, request blocks by work, and accept a signed block announcement. There is no peer discovery, no scoring, no ban policy, and no Sybil or eclipse defense.

## D. External audit

**Status: not audited.**

An audit cannot be performed by the implementing agent. See `docs/AUDIT.md`. Status stays not audited until an independent reviewer finishes.

## E. Anonymity set

**Status: 8 leaves (depth 3). Not raised.**

Raising depth requires restructuring the hardcoded boundary slots in `boundaries()` (fixed array of 7, slots 3..5 overwritten after the depth loop). A mainnet set needs depth 20 or more and that circuit change. Effective anonymity also depends on how many notes are live.
