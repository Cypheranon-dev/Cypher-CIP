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
| Membership gadget | Standalone. Not wired into the production spend path. |
| Lite routing AIR | Degree-2 selector routing, 8 levels. Not Poseidon2. Not the production spend. See docs/LITE.md. |
| Signatures | Outer signature verified outside the proof. In-circuit verification not implemented. |
| Network | Peer scoring and announcement checks present. Public discovery and Sybil defense not finished. |
| Encryption | Authenticated encryption on notes. |
| Audit | None. Candidate status only. |

## Blockers

These are under development. Closing them is required before any mainnet consideration.

1. **In-circuit signature verification.** Outer signature still checked outside the proof. A minimal research binding (`mldsa_circuit_bind`) now checks key commitment + a simple challenge byte. Full FIPS 204 verification inside a STARK is not implemented.
2. **Recursive verification.** A second-fold helper exists in the minimal FRI skeleton. The fold still does not re-execute an inner FRI/AIR. The node still verifies each spend natively.
3. **Network defenses.** Announcements are now versioned and length-bounded (1 MiB). Scoring exists. Discovery, diversity, and eclipse resistance are not finished.
4. **Larger anonymity set.** Production tree remains fixed at depth 3. An optional depth-4 membership check (`production_optional_depth4`) is callable but is not the production path.
5. **External audit.** No independent review has been completed.

## Mainnet gates

All blockers above, plus independent audit with critical and high findings resolved, sustained testnet, and explicit release approval. Passing CI is not proof of protocol security.

## Proof parameter policy (Milestone 1C)

`verify` rejects any proof whose Winterfell options do not exactly match `proof_options()`.
A real negative test constructs a proof with fewer queries and less grinding and confirms rejection.

