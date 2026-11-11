# Status

**Under development.** Private testnet / pre-mainnet audit candidate. Mainnet is not launched. Production use is blocked pending completion of protocol work, independent security audits, testnet validation, and explicit release approval.

A module is not landed until the production path calls it and a production-path test passes.

## Core idea

Cypher is a privacy-first Layer-1. Transfers are proved in a STARK with no trusted setup. Post-quantum signatures wrap the proof. The note tree, the fold, and the network are research-stage.

## Current progress

| Area | State |
| --- | --- |
| Spend proof | Working at the current tree depth. Production path is fixed-depth. |
| Fold | Domain-separated absorb. Not a recursive verifier. Each spend is still verified natively. |
| Membership gadget | Standalone. Not wired into the production spend path. |
| Signatures | Outer signature verified outside the proof. In-circuit verification not implemented. |
| Network | Peer scoring and announcement checks present. Public discovery and Sybil defense not finished. |
| Encryption | Authenticated encryption on notes. |
| Audit | None. Candidate status only. |

## Blockers

These are under development. Closing them is required before any mainnet consideration.

1. **In-circuit signature verification.** The outer signature is checked outside the proof. An in-circuit check is not implemented.
2. **Recursive verification.** The fold does not re-execute the inner proof. The node still verifies each spend natively.
3. **Network defenses.** Scoring exists. Discovery, diversity, and eclipse resistance are not finished.
4. **Larger anonymity set.** The production tree is fixed at the current depth. A padded circuit exists as a standalone gadget and is not the production path.
5. **External audit.** No independent review has been completed.

## Mainnet gates

All blockers above, plus independent audit with critical and high findings resolved, sustained testnet, and explicit release approval. Passing CI is not proof of protocol security.
