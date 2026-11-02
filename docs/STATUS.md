# Status

Updated November 2026. Current build: CIP-2026.1. CIP-2026 targets and security fixes are in `docs/CIP-2026.md`.

| Item | State |
| --- | --- |
| Whitepaper v1.4 | `docs/whitepaper-v1.4.pdf` |
| CIP-2026 | `docs/CIP-2026.md` |
| Protocol | `docs/PROTOCOL.md` |
| CIP AIR | `crypto/cip`. Poseidon2, Goldilocks. 32-bit amounts. Depth-3 note tree. |
| Dilithium5 / Dilithium2 | `crypto/dilithium`, FIPS 204. |
| Equihash | `crypto/equihash`. Equihash(48, 5). Not Equihash-512. |
| Node | `node`. Local chain. Coinbase `sk` is not serialized. Timestamp future bound is 2 hours. |
| Coinbase key leak | Fixed. Raw field `sk` is not written to the wire or the PoW prefix. |
| Viewing keys | Design only. Optional threshold compliance key. |
| Recovery | Design only. Opt-in, user-authorized, time-locked. Not a validator power. |
| Offline / mesh | Design only. Self-contained proofs. |
| Audit / bounty | None. |

## Not yet in consensus

- Dilithium spend key inside the STARK (field `sk` is still 64-bit)
- Threshold envelope key
- Recursive STARKs
- Public P2P testnet
- Note encryption
- Bridge, swap, DAO, oracle as consensus transactions
