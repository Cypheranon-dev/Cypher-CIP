# Status

**Under development.** Private testnet / pre-mainnet audit candidate. Mainnet is not launched. Production use is blocked pending completion of protocol work, independent security audits, testnet validation, and explicit release approval.

A module is not landed until the production path calls it and a production-path test passes.

## Core idea

Cypher is a privacy-first Layer-1. Transfers are proved in a STARK with no trusted setup. Post-quantum signatures wrap the proof. The note tree, the fold, and the network are research-stage.

## Current progress

| Area | State |
| --- | --- |
| Spend proof | Working at the current tree depth. Production path is fixed-depth. |
| Fold | Poseidon2 absorb re-executed in the fold AIR, up to five digests. The proof binds the commitment and the count. The digest list is checked natively. Not a recursive verifier. Each spend is still verified natively. |
| Membership gadget | Production `CipAir` constrains a depth-3 Poseidon path (`boundaries` + `parent`). `padded_path` is a separate linear mux and is not that path. See docs/DEPTH4.md. |
| Lite routing AIR | Degree-2 selector routing, 8 levels. Not Poseidon2. Not the production spend. See docs/LITE.md. |
| Signatures | Outer signature verified outside the proof. In-circuit verification not implemented. |
| Network | Private-testnet announcements: version, network id, signature over network id and block id, one block, replay of a known id, then `append` or `consider`. No peer discovery and no Sybil or eclipse defense. |
| Encryption | Authenticated encryption on notes. |
| Audit | None. Candidate status only. |

## Blockers

These are under development. Closing them is required before any mainnet consideration.

1. **In-circuit signature verification.** The node verifies ML-DSA outside the proof. `mldsa_circuit_bind` only checks that a public key opens a Poseidon2 commitment. It does not implement NTT, the challenge polynomial, hints, or norm checks. Full FIPS 204 verification inside a STARK is not implemented.
2. **Recursive verification.** The fold AIR re-executes a Poseidon2 absorb and binds the final commitment and a constant count. It does not re-execute FRI or an inner AIR. The node still verifies each spend natively.
3. **Network defenses.** Announcements are versioned, bound to `cypher-testnet-0`, length-capped, and rejected when the block id is already on the chain. Scoring exists. Discovery, diversity, and eclipse resistance are not finished.
4. **Larger anonymity set.** Production tree is depth 3 (8 leaves). The depth-4 slot map is in docs/DEPTH4.md and is not the production AIR. `padded_path` is not a replacement for `parent()`.
5. **External audit.** No independent review has been completed.

## Mainnet gates

All blockers above, plus independent audit with critical and high findings resolved, sustained testnet, and explicit release approval. Passing CI is not proof of protocol security.

## Proof parameter policy (Milestone 1C)

`verify` rejects any proof whose Winterfell options do not exactly match `proof_options()`.
A real negative test constructs a proof with fewer queries and less grinding and confirms rejection.

