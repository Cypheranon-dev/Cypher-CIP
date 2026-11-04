# Missing capabilities

Reviewed against `main` at `7ca407c`. None of these are implemented. Designs only.

## A. Full ML-DSA verification inside the STARK

**Status: design only. Not implemented.**

The current path verifies Dilithium with FIPS 204 outside the proof and binds the public key into the STARK statement via `payload == commit[0]`. That is not in-circuit signature verification.

A real circuit would constrain:
- matrix expansion `A` from `ρ`
- NTT/INTT of the polynomials
- challenge polynomial `c` from the commitment hash
- hint decompression
- infinity-norm checks on `z` and `r`

Estimated size: several hundred thousand constraints for Dilithium2, more for Dilithium5. Winterfell over Goldilocks can host this, but it is a multi-month circuit project with its own audit. Until that AIR exists and is tested against FIPS 204 known-answer vectors, the claim must remain "signature checked outside the proof."

## B. Full Winterfell verifier circuit

**Status: not implemented. `recursive.rs` is relabeled.**

`RecursiveAir` constrains columns to be copies. It does not verify a STARK and does not constrain a Poseidon fold. Sound aggregation is `aggregate()`, which calls `verify_bytes` natively.

A verifier circuit would re-execute FRI, Merkle openings, and the AIR constraints inside another proof. That is the standard recursive-STARK construction. It is not in this tree.

## C. Public gossip network

**Status: partial. Authenticated point-to-point push only.**

`Node::broadcast` opens a TCP connection to each configured peer and sends a signed block id. There is no peer discovery, no scoring, no ban policy, no rate limit, and no Sybil or eclipse defense. A public network needs those before any deployment claim.

## D. External audit

**Status: not audited.**

An audit cannot be performed by the implementing agent. See `docs/AUDIT.md` for the scope a reviewer would need. Status stays "not audited" until an independent reviewer finishes and findings are resolved.

## E. Anonymity set larger than 8

**Status: design only.**

`DEPTH = 3`, `LEAVES = 8` are constants in the AIR. Raising them requires a new circuit, new public inputs, and a migration. Effective anonymity also depends on how many notes are actually live, not on the tree capacity.

Target for a later circuit: depth 20 (about 1 million leaves) with historical roots and nullifier set. Not this tree.
