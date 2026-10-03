# Architecture map

What `sim/protocol_logic.py` checks. It is not the paper's cryptography.

| Paper | In this stand-in |
| --- | --- |
| §4 four statements | Membership of the input note, nullifier `H(sk, r_in)`, exact input value, height equal to the next block. SHA-256, not a STARK. |
| §5 envelope | `SHA-256` over a bind of amount, kind, height, and the rule fields. Re-checked in `seal`. Not Dilithium5. |
| §6 flow | Build, then miner re-check, then header hash. |
| §7 PoW | Not run. `work` is a block count, so heavier-chain is only a counter. |
| §8 viewing keys | `H("vk", sk)` opens that key. A foundation string does not. |
| §9 MaskSwap | A spend whose price is outside `[p_a, p_b]` is rejected. Fees are not moved. |
| §10 bridge | A mint. Rejected below 8 votes or above 15. |
| §11 oracle | A spend whose three weights do not sum to 1 is rejected. |
| §12 emission | `sim/emission.py`. |
| §17 DAO | A spend under 5% turnout or 67% yes is rejected. |
