# Status

Updated after the recheck of `809cc4d`.

| Item | State |
| --- | --- |
| Coinbase spend key | Private. Not derived from the miner public key. |
| Coinbase amount | Proof must verify, output must match the commitment, payload must equal the scheduled reward. The AIR does not yet constrain `amount_in == payload` for coinbase proofs. A dedicated selector constraint is the next circuit change. |
| Note encryption | Fresh nonce required. Fails closed if `/dev/urandom` is unavailable. Still a custom construction, not a vetted AEAD. |
| Wire limits | Indices capped at 10k. Extra envelope signatures capped at 64. |
| Infrastructure keys | Full key set is persisted and restored. |
| Peer sync | Requests blocks when remote work is greater, then selects by validated work. |
| Broadcast | Sends the block bytes over TCP with a signed id. Not a gossip network. |
| Block id | Commits to extra envelope signatures. |
| Recursive AIR | Not a verifier circuit. |
| Anonymity set | 8 leaves. |
| Audit | None. |

Not for real funds.
