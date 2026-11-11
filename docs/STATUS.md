# Status

**Private testnet / pre-mainnet audit candidate.** Mainnet is not launched. Production release remains gated on independent audit, sustained testnet validation, and explicit project-owner approval.

| Item | State |
| --- | --- |
| Coinbase spend key | Private. |
| Coinbase amount | AIR constrains `payload == amount_in` when the coinbase flag is 1. |
| Note encryption | XChaCha20-Poly1305. Fails closed without entropy. |
| Proof parameters | Verification accepts only the protocol-pinned options. Weaker options are rejected before Winterfell runs. |
| Recursive fold | Domain-separated Poseidon2 absorb. Reorder changes the commitment. Not a verifier circuit. |
| Announcements | Structure, id, and signature length checked. Application is the caller's job via `chain.consider`. Not a gossip network. |
| In-circuit ML-DSA | Not implemented. Signature checked outside the proof. Design in MISSING.md. |
| Note tree | Depth 3, 8 leaves. Boundary layout blocks a depth increase without redesign. |
| Peers | Hello with network ID, work-based sync, signed announcement with structure check. No Sybil or eclipse defense. |
| External audit | None. |

## Mainnet gates

All of the following are required before any mainnet decision:

1. Full ML-DSA verification inside the STARK, or an explicit owner decision to remove that property from launch scope.
2. A genuine Winterfell verifier circuit, or an explicit owner decision to remove recursion from launch scope.
3. Public gossip with a tested Sybil/eclipse threat model, or an explicit owner decision to keep the deployment private/static-peer.
4. Anonymity circuit redesign beyond depth 3, or an explicit owner decision to retain the eight-leaf limit.
5. Independent cryptography, consensus, network, and storage audits with critical/high findings resolved.
6. Sustained testnet validation and explicit project-owner release approval.
