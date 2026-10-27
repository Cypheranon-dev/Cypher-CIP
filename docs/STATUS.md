# Status

Updated October 2026. CIP-2026 research targets are in `docs/CIP-2026.md`.

| Item | State |
| --- | --- |
| Whitepaper v1.4 | `docs/whitepaper-v1.4.pdf` |
| CIP-2026 | `docs/CIP-2026.md`. Privacy+compliance, quantum recovery, offline mesh. |
| Protocol | `docs/PROTOCOL.md` |
| CIP AIR | `crypto/cip`. Poseidon2, Goldilocks, width 8. 32-bit amounts. Depth-3 note tree. |
| Dilithium5 / Dilithium2 | `crypto/dilithium`, FIPS 204. |
| Equihash | `crypto/equihash`. Equihash(48, 5). Not Equihash-512. |
| Node | `node`. Local chain, mempool, coinbase, retarget. Not public P2P. |
| Viewing keys | Design in CIP-2026. Optional threshold compliance key. Not yet in consensus. |
| Recovery window | Design in CIP-2026. Supermajority validator recovery inside challenge window. Not yet in consensus. |
| Offline / mesh | Design in CIP-2026. Self-contained proofs relayable over QR/BLE/mesh. Wallet QR signing exists; mesh relay not yet in node. |
| Emission | `sim/emission.py` and node `block_reward`. |
| CI | `.github/workflows/test.yml` |
| Audit / bounty | None. |

## Not yet implemented (CIP-2026 targets)

- Threshold view-key registration and decryption path
- Challenge-window recovery transaction type
- Mesh/BLE/LoRa relay of signed STARK proofs
- Recursive STARKs
- Public P2P testnet
- Note encryption
- Bridge, swap, DAO, oracle as consensus transactions

These are research targets. The current tree proves the base STARK spend and Dilithium envelope.
