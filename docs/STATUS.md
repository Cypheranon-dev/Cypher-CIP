# Status

**Private testnet / pre-mainnet audit candidate.** Owner has approved this tree as an audit candidate. Mainnet is not launched. Production release remains gated on completion of the remaining circuit work, independent audit, sustained testnet validation, and explicit release approval.

| Item | State |
| --- | --- |
| Coinbase spend key | Private. |
| Coinbase amount | AIR constrains `payload == amount_in` when the coinbase flag is 1. |
| Note encryption | XChaCha20-Poly1305. Fails closed without entropy. |
| Proof parameters | Verification accepts only the protocol-pinned options. |
| Recursive fold | Domain-separated Poseidon2. Not a verifier circuit. |
| Announcements | Structure, id, and signature length checked. Peer scoring and ban threshold added. |
| Peer scoring | `note_good` / `note_bad` / `is_banned`. Cap 32 peers. |
| In-circuit ML-DSA | Not implemented. Signature checked outside the proof. |
| Note tree | Depth 3, 8 leaves. Boundary layout redesign required for depth 4+. |
| External audit | None. Candidate status approved by owner. |

## Remaining for full implementation

1. Full ML-DSA verification inside the STARK (NTT, challenge, hints, norms).
2. Winterfell verifier circuit (FRI, Merkle openings, inner AIR).
3. Public gossip: discovery, Sybil defense beyond scoring, eclipse resistance.
4. Anonymity circuit: boundary slot separation, depth 4 correctness, then higher.

## Mainnet gates

All remaining items above, plus independent audit with critical/high findings resolved, sustained testnet, and explicit release approval.
