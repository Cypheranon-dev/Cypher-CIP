# Version

**Final research build: CIP-2026.7** (November 2026)

This closes the research plan for the current tree.

## What this build finishes

| Plan item | State |
| --- | --- |
| FRI STARK spend, no trusted setup | Done. Degrees match Winterfell 0.13. |
| Dilithium5 M-of-N envelope | Done. Default M = 1. |
| Dilithium2 user signature | Done. 1312-byte key required. |
| Poseidon2 notes, nullifiers, events | Done. |
| Dilithium key binding (circuit v2) | Done. Spend commits to the key. |
| In-circuit Dilithium binding check | Done. `dilithium_bind::check_spend_key`. Full ML-DSA-in-STARK is a later AIR. |
| Recursive aggregation | Done. `aggregate` folds verified spends into one commitment. Full verifier-in-circuit is a later AIR. |
| Note encryption | Done. |
| Coinbase key not on chain | Done. |
| Timestamp bounds | Done. Caller-supplied, 1-day jump cap. |
| Finite emission | Done. Integer form of §12.2. |
| Equihash(48, 5) | Done. Paper (512, 9) does not construct. |

## Explicitly later (not this tree)

- Full Winterfell verifier circuit (recursive AIR)
- Full ML-DSA verification inside the STARK
- Public P2P testnet with authenticated gossip
- Third-party audit and bounty

The current tree is the full CIP chain for this research stage. `sim/` is not consensus.
