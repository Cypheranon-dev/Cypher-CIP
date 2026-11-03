# Status

Updated November 2026. Current build: CIP-2026.4.

| Item | State |
| --- | --- |
| CIP AIR v1 | Spend proof generates. Degrees match Winterfell 0.13. Wrong nullifier rejected. |
| CIP circuit v2 | Dilithium key commitment. Required on the spend path. |
| Dilithium envelope | 1-of-N. Full Dilithium5, not a stand-in. |
| User signature | Dilithium2, 1312-byte key required. |
| Note encryption | `encrypt_note` / `decrypt_note`. |
| Equihash | (48, 5). Cap 24 bits. Paper (512, 9) rejected. |
| Coinbase sk | Not serialized. |
| Timestamps | Caller-supplied. Strictly increasing. 1-day jump cap. |
| Emission | Integer form of §12.2. |
| Recursive STARKs | Path in `docs/RECURSIVE.md`. Not dropped. |
| In-circuit Dilithium verify | Not yet. v2 commits to the key; signature check is outside the STARK. |
| Audit | None. |

This tree is the full CIP chain. `sim/` is not consensus.
