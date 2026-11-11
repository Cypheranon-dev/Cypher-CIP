# Missing capabilities

## A. Full ML-DSA verification inside the STARK

**Status: design only. Not implemented.**

The node verifies Dilithium with FIPS 204 outside the proof. The STARK binds the public key into the statement via a Poseidon2 commitment in the payload. The user signature covers the proof bytes and public inputs. This is not in-circuit signature verification.

A real circuit would constrain matrix expansion, NTT/INTT, the challenge polynomial, hint decompression, and infinity-norm checks for the chosen FIPS 204 parameter set. Estimated size for Dilithium2 is several hundred thousand constraints. Until that AIR exists and is tested against FIPS 204 known-answer vectors, the claim remains "signature checked outside the proof."

**Owner decision required:** is full in-circuit ML-DSA a hard launch requirement, or is the current binding (key commitment + external FIPS 204 check) acceptable for launch?

## B. Recursive verifier circuit

**Status: fold only. Not a verifier.**

`verify_fold` uses a domain-separated Poseidon2 absorb (`Poseidon2(acc || digest || DOM_AGG)`). Reordering or omitting a digest changes the commitment. The AIR does not re-execute FRI or attest that the digests came from valid spend proofs. The node must call `verify_bytes` on each spend before folding.

**Owner decision required:** is a genuine Winterfell verifier circuit a hard launch requirement, or is native individual verification plus a Poseidon2 fold acceptable?

## C. Public gossip network

**Status: partial.**

Peers exchange hellos with a network identifier, request blocks by work, and accept a signed announcement that is structure-checked. There is no peer discovery, no scoring, no ban policy, and no Sybil or eclipse defense. `serve_one` validates announcement structure and id but does not apply the block; the caller must call `chain.consider`.

**Owner decision required:** is public gossip with Sybil/eclipse defense a launch requirement, or is a private/static-peer testnet the launch scope?

## D. External audit

**Status: not audited.**

An implementing agent cannot certify an external audit. Status stays not audited until independent reviewers finish and critical/high findings are resolved.

## E. Anonymity set

**Status: 8 leaves (depth 3). Not raised.**

`boundaries()` uses a fixed array of 7 and overwrites slots 3–5 after the depth loop. Raising depth requires restructuring those slots so Merkle path operations cannot overwrite nullifier, output, or event boundaries. A mainnet set needs depth 20 or more and that circuit change. Effective anonymity depends on how many notes are live, not on tree capacity.

**Owner decision required:** target depth and acceptable proving cost, or retain the eight-leaf limit for launch.
