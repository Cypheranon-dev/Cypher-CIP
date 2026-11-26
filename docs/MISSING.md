# Missing capabilities

## A. Full ML-DSA verification inside the STARK

**Status: design only. Not implemented.**

The node verifies Dilithium with FIPS 204 outside the proof. The STARK binds the public key into the statement via a Poseidon2 commitment in the payload. The user signature covers the proof bytes and public inputs. This is not in-circuit signature verification.

A real circuit would constrain matrix expansion, NTT/INTT, the challenge polynomial, hint decompression, and infinity-norm checks for the chosen FIPS 204 parameter set. Estimated size for Dilithium2 is several hundred thousand constraints. Until that AIR exists and is tested against FIPS 204 known-answer vectors, the claim remains "signature checked outside the proof."

**Owner decision required:** is full in-circuit ML-DSA a hard launch requirement, or is the current binding (key commitment + external FIPS 204 check) acceptable for launch?

## B. Recursive verifier circuit

**Status: fold only. Not a verifier.**

`verify_fold` uses a domain-separated Poseidon2 absorb (`Poseidon2(acc || digest || DOM_AGG)`). Reordering or omitting a digest changes the commitment. The fold AIR re-executes that absorb and binds the commitment and the count. It does not re-execute FRI, and it does not attest that the digests came from valid spend proofs. The node must call `verify_bytes` on each spend before folding. The digest list is checked by `verify_fold`, not inside the proof.

**Owner decision required:** is a genuine Winterfell verifier circuit a hard launch requirement, or is native individual verification plus a Poseidon2 fold acceptable?

## C. Public gossip network

**Status: private testnet transport only.**

`broadcast` and `serve_one` share `encode_announce` / `validate_announce` / `ingest_announce`. The frame is version 1, the network id `cypher-testnet-0`, a Dilithium2 signature over `network id || block id`, and exactly one block. A block id already stored on the receiving chain is rejected as a replay. A block whose `prev` is the tip is passed to `append`; any other block is passed to `consider`, which keeps it only when the candidate chain has strictly more work. There is no peer discovery, no subnet diversity, and no Sybil or eclipse defense.

**Owner decision required:** is public gossip with Sybil/eclipse defense a launch requirement, or is a private/static-peer testnet the launch scope?

## D. External audit

**Status: not audited.**

An implementing agent cannot certify an external audit. Status stays not audited until independent reviewers finish and critical/high findings are resolved.

## E. Anonymity set

**Status: 8 leaves (depth 3). Not raised.**

Production membership is the depth-3 Poseidon path inside `CipAir` (`boundaries` loads siblings, the next slot runs `parent`). `boundaries()` uses a fixed array of 7 and writes nullifier, output, and event into slots 3–5 after the depth loop. `padded_path` is a different linear walk and is not this path. A depth-4 schedule is written in docs/DEPTH4.md and is not implemented. Raising depth changes the trace length or drops the pad slot, and it changes `LEAVES`, genesis, and existing proofs. Effective anonymity depends on how many notes are live, not on tree capacity.

**Owner decision required:** target depth and acceptable proving cost, or retain the eight-leaf limit for launch.
