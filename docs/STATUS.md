# Status

**Private testnet / pre-mainnet audit candidate.** Mainnet is not launched. Production release remains gated on independent audit, sustained testnet validation, and explicit project-owner approval. A green CI run is not proof of protocol security.

| Item | State |
| --- | --- |
| Coinbase spend key | Private. Not derived from a public value. |
| Coinbase amount | AIR constrains `payload == amount_in` when the coinbase flag is 1. Node requires the flag. |
| Note encryption | XChaCha20-Poly1305. Fresh 24-byte nonce. Fails closed without entropy. |
| Proof parameters | Verification accepts only the protocol-pinned options (84 queries, blowup 8, grinding 20). |
| Note tree | Depth 3, 8 leaves. Boundary layout blocks a depth increase without redesign. |
| Recursive fold | Constrained absorb (`next = cur + digest`). Not a Winterfell verifier circuit. |
| In-circuit ML-DSA | Not implemented. Design in MISSING.md. Signature checked outside the proof. |
| Peers | Hello with network ID, work-based sync, signed block announcement. Not a gossip network. No Sybil or eclipse defense. |
| External audit | None. Scope in AUDIT.md. |

## Mainnet gates (all required)

1. Consensus and protocol versioning frozen and documented.
2. Required cryptographic relations sound and independently validated, including full ML-DSA-in-STARK and a genuine recursive verifier, or those requirements explicitly removed from launch scope.
3. Independent cryptography, consensus, network, and storage audits complete.
4. Critical and high findings fixed and re-tested.
5. Sustained private/public testnet validation and incident response demonstrated.
6. Explicit project-owner release approval.

Until those gates are evidenced, deployment is testnet-only.
