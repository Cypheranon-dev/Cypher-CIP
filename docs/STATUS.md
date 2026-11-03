# Status

Updated November 2026. Current build: CIP-2026.6.

| Item | State |
| --- | --- |
| CIP AIR v1 | Spend proof generates. Degrees match Winterfell 0.13. |
| CIP circuit v2 | Dilithium key commitment. Required on the spend path. |
| Recursive aggregation | `aggregate` verifies spends and folds them into one commitment. |
| Envelope | M-of-N Dilithium5. Default M = 1. |
| User signature | Dilithium2, 1312-byte key required. |
| Note encryption | Present. |
| Equihash | (48, 5). Cap 24 bits. |
| Coinbase sk | Not serialized. |
| Timestamps | Caller-supplied. 1-day jump cap. |
| Full recursive AIR | Later. Aggregation layer is in. |
| In-circuit Dilithium verify | Later. |
| Audit | None. |
