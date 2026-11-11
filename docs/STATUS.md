# Status

**Private testnet / pre-mainnet audit candidate.** Mainnet is not launched. Production release remains gated on completion of the remaining circuit work, independent audit, sustained testnet validation, and explicit release approval.

A module is not landed for a phase until the production path calls it and a production-path test passes.

| Item | State |
| --- | --- |
| Coinbase spend key | Private. |
| Coinbase amount | AIR constrains `payload == amount_in` when the coinbase flag is 1. |
| Note encryption | XChaCha20-Poly1305. Fails closed without entropy. |
| Proof parameters | Verification accepts only the protocol-pinned options. |
| Fold | `recursive.rs` proves a Poseidon2 absorb. Not a verifier circuit. Node still calls `verify_bytes` per spend. |
| `padded_path` | Standalone membership gadget. Not called by the production spend AIR. |
| Announcements | Structure, id, and signature length checked. Peer scoring and ban threshold added. |
| Peer scoring | `note_good` / `note_bad` / `is_banned`. Cap 32 peers. |
| In-circuit ML-DSA | Not implemented. Signature checked outside the proof. |
| Note tree | Depth 3, 8 leaves. Production AIR still hardcoded. |
| External audit | None. Candidate status approved by owner. |

## Remaining

1. Full ML-DSA verification inside the STARK.
2. Winterfell verifier circuit. The fold does not re-execute FRI. Node must still call `verify_bytes`.
3. Public gossip with Sybil and eclipse defense.
4. Wire `padded_path` into the production spend AIR, or redesign the production path so a larger tree is one circuit. Not done.

## Mainnet gates

All remaining items, plus independent audit with critical and high findings resolved, sustained testnet, and explicit release approval.
