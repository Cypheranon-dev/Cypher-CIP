# Status

**Final research build: CIP-2026.7** (November 2026)

| Item | State |
| --- | --- |
| CIP AIR v1 | Spend proof generates. Wrong nullifier rejected. |
| CIP circuit v2 | Dilithium key commitment on the spend path. |
| Dilithium binding check | `dilithium_bind::check_spend_key`. |
| Recursive aggregation | `aggregate` verifies and folds spends. |
| Aggregate check | `check_aggregate` on the node. |
| Envelope | M-of-N Dilithium5. |
| User signature | Dilithium2, 1312 bytes. |
| Note encryption | Present. |
| Equihash | (48, 5). Cap 24 bits. |
| Coinbase sk | Not serialized. |
| Timestamps | Caller-supplied. 1-day cap. |
| Emission | Integer §12.2. |
| Full recursive AIR | Later research. |
| Full in-circuit ML-DSA | Later research. |
| Public P2P | Later. |
| Audit | None. |

This is the final update for the CIP-2026 research plan.
