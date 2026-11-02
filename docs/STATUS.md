# Status

Updated November 2026. Current build: CIP-2026.3.

| Item | State |
| --- | --- |
| CIP AIR v1 | `crypto/cip`. Poseidon2 spend. Field `sk`. |
| CIP circuit v2 | `crypto/cip/src/v2.rs`. Dilithium key commitment binding. |
| Dilithium envelope | 1-of-N (`open_threshold`). |
| Note encryption | `encrypt_note` / `decrypt_note`. |
| Equihash | (48, 5). Cap 24 bits. |
| Coinbase sk | Not serialized. |
| Recursive STARKs | Path in `docs/RECURSIVE.md`. Not in AIR. |
| In-circuit Dilithium verify | Not yet. v2 commits to the key; signature check is outside the STARK. |
| Audit | None. |
