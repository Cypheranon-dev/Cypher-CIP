# Status

Updated November 2026. Current build: CIP-2026.2.

| Item | State |
| --- | --- |
| CIP AIR | `crypto/cip`. Poseidon2. 32-bit amounts. Depth-3. |
| Dilithium | FIPS 204. Envelope is 1-of-N (`open_threshold`). |
| Field spend seed | `field_seed_from_pk` derives seed from Dilithium pk. |
| Note encryption | `encrypt_note` / `decrypt_note`. |
| Equihash | (48, 5). Difficulty cap 24 bits. |
| Coinbase sk | Not serialized. |
| Timestamp bound | 2 hours. |
| Peers | `Node::add_peer` list. Not full gossip. |
| Recursive STARKs | Path in `docs/RECURSIVE.md`. Not in AIR. |
| Recovery | Opt-in design. Not in consensus. |
| Audit | None. |

## Remaining

- In-circuit Dilithium
- M-of-N threshold signatures
- Public P2P testnet
- Recursive block proofs
