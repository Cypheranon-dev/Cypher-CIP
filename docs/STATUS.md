# Status

Reviewed `main` at `7ca407c`. Branch `cip-2026-review-fixes`.

| Item | State |
| --- | --- |
| Coinbase | Publicly determined. No skip when secret is absent. |
| Spend key binding | Payload (STARK-constrained) must match Dilithium key-commitment lane. Not in-circuit ML-DSA. |
| Note encryption | Authenticated custom stream. Fresh nonce. Not a vetted AEAD. |
| Chain selection | Heavier work via `consider`. |
| Wire limits | 10k blocks, 1k transfers, 1 MiB blobs. |
| Block id | Commits to proof, public inputs, and both signatures. |
| Production config | Persisted. |
| Recursive AIR | Relabeled. Not a verifier circuit. |
| Broadcast | TCP push of a signed block id. Not a gossip network. |
| Anonymity set | 8 leaves. |
| Audit | None. |

Not for real funds.
