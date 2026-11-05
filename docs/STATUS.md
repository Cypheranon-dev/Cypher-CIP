# Status

| Item | State |
| --- | --- |
| Coinbase spend key | Private. |
| Coinbase amount | AIR constrains `payload == amount_in` when the coinbase flag is 1. |
| Note encryption | XChaCha20-Poly1305. Fails closed without entropy. |
| Note tree | Depth 3, 8 leaves. Boundary layout blocks a depth increase. |
| Recursive fold | Constrained absorb (`next = cur + digest`). Not a verifier circuit. |
| In-circuit ML-DSA | Not implemented. Design in MISSING.md. |
| Peers | Hello, work-based sync, signed block announcement. Not a gossip network. |
| Audit | None. Scope in AUDIT.md. |

Not for real funds.
