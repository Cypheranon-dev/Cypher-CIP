# Architecture map

This is the map the stand-in implements. It is not the paper's cryptography.

| Paper | In `sim/protocol_logic.py` |
| --- | --- |
| §4 four statements | Membership, `H(sk, r_in)` nullifier, non-negative amount, height bind. Hash checks, not a STARK. |
| §5 envelope | `SHA-256(bind \|\| event_id)` re-checked in `seal`. Not Dilithium5. |
| §6 flow | Local build, then miner re-check, then header hash. |
| §7 PoW | Not run. Height is a batch counter. |
| §8 viewing keys | Per-key `H(vk, sk)`. No foundation key. |
| §9 MaskSwap | Price must sit in `[p_a, p_b]`. Fees not moved. |
| §10 bridge | Mint-style reject under 8 votes. |
| §11 oracle | Weights 0.50 / 0.40 / 0.10 must sum to 1. |
| §12 emission | Checked beside the soak, not inside it. See `results/SOAK.md`. |
| §17 DAO | Reject under 5% turnout or 67% yes. |
