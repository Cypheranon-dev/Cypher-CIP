# Version

**Current build: CIP-2026.2** (November 2026)

Research debt from CIP-2026.1 is addressed in this build.

## Completed in CIP-2026.2

| Debt | Resolution |
| --- | --- |
| 64-bit field spend key | `field_seed_from_pk` derives the field seed from the Dilithium public key. New wallets should use this. Full in-circuit Dilithium remains a future circuit revision. |
| Single infra key | `Chain::open_threshold` accepts 1-of-N infrastructure keys. Any listed key may sign the envelope. |
| Equihash(48,5) + 12-bit clamp | Difficulty cap raised to 24 bits. Equihash(48,5) remains the solved instance; a stronger instance is a parameter change. |
| No note encryption | `encrypt_note` / `decrypt_note` (Blake2b keystream) added. |
| Single peer | `Node::add_peer` holds a peer list. Gossip over the list is the next step. |
| Recursive STARKs | Documented path in `docs/RECURSIVE.md`. Not in the current AIR. |

## Still future work

- Dilithium verification inside the STARK circuit
- M-of-N threshold signatures (current is 1-of-N)
- Public P2P testnet with authenticated gossip
- Recursive block proofs

See `docs/CIP-2026.md` and `docs/STATUS.md`.
