# Status

Updated on the open-items branch.

| Item | State |
| --- | --- |
| Coinbase spend key | Private. Not derived from a public value. |
| Coinbase amount | AIR constrains `payload == amount_in` when the coinbase flag is 1. Node requires the flag. |
| Note encryption | XChaCha20-Poly1305. Fresh 24-byte nonce. Fails closed without entropy. |
| Wire limits | Enforced. |
| Infrastructure keys | Full set persisted. |
| Peer sync | Heavier work wins. |
| Broadcast | Sends the block over TCP. Not a gossip network. |
| Recursive AIR | Not a verifier circuit. |
| In-circuit ML-DSA | Not implemented. Signature is checked outside the proof. |
| Anonymity set | 8 leaves. |
| Audit | None. |

Not for real funds.
