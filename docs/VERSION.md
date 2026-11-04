# Version

**CIP-2026.10 — review fixes** (November 2026)

Addresses the static review of `3295bbb`. Hardened demo, not mainnet. Do not put funds on this.

## Fixed

| Review finding | Fix |
| --- | --- |
| Coinbase skipped when `sk == 0` | Commitment must equal `note(reward, field_seed(coinbase_pk), height)`. Publicly checkable. Arbitrary mint rejected. |
| Key binding verified against itself | Payload (constrained by the STARK) must equal the first lane of the Dilithium key commitment. |
| Unauthenticated XOR encryption | Nonce + Blake2b keystream + 16-byte tag. Tamper rejected. |
| Peer sync appended by height | `exchange` builds a candidate chain and calls `consider`. Heavier work wins. |
| Unbounded wire allocation | Caps: 10k blocks, 1k transfers/block, 1 MiB per byte blob. |
| Production config not persisted | Threshold and production flag are in the chain file and restored on load. |

## Still not done

- Full ML-DSA equations inside the STARK
- Recursive AIR that constrains the Poseidon fold (current constraints are copies)
- Live socket broadcast (announcement is signed; transport is a stub)
- External audit
- Anonymity set larger than 8

Do not put funds on this build.
