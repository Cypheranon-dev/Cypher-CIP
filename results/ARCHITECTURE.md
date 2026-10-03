# Architecture map

Two layers. The Rust crates are the cryptography that runs. The Python module is still a rule stand-in for the parts with no circuit.

Names below are the names of the thing that runs. The paper's names are named as the paper's, and are not reused for a different object.

## Rust

| Paper | In this tree |
| --- | --- |
| §4 four statements | `crypto/cip`. Membership of `pk = H(sk, 1)` in a depth-4 miner root, nullifier `H(sk, r_in)`, `amount_in = amount_out + fee`, `event_id = H(H(payload, height), chain_state)`. `H` is `a³ + 3b³ + 7` over Goldilocks. The paper's name for `H` is Poseidon2. This `H` is not Poseidon2, and it is not Rescue-Prime. |
| §4.7 FRI | Winterfell. Blowup 8, 84 queries, grinding 20, quadratic extension. Those four numbers are the §4.7 proposal. The commitment hasher is the library's `Rp64_256`. That hasher is not the in-circuit hash and not Poseidon2. §4.7.1's Rescue-Prime was a toy circuit standing in for Poseidon2. This AIR is not that toy. |
| §5 envelope | Dilithium5 over `proof \|\| event_id`. User authorization is Dilithium2 over `proof \|\| public inputs`. Neither key is the field `sk`. |
| §7 PoW | `crypto/equihash`. Blake2b, Zcash verification order. The paper's name Equihash-512 means Equihash(n=512, k=9). That pair does not construct. The in-process chain solves Equihash(48, 5), then requires leading zero bits on `commit(header, indices)`. Equihash(48, 5) is not Equihash-512. |
| §7 retarget | `node::adjust_difficulty`. The paper's formula uses `T_target = 120` seconds. Tested on a synthetic 500-interval window. Not run against a live 500-block history, and not a 120-second block loop. |
| Chain | `node`. Nullifier set, coinbase Dilithium2, heavier chain replaces. One process. Not a public P2P testnet. |

## Python stand-in

What `sim/protocol_logic.py` checks. It is not the paper's cryptography.

| Paper | In this stand-in |
| --- | --- |
| §4 four statements | Membership of the input note, nullifier `H(sk, r_in)`, exact input value, height equal to the next block. SHA-256, not a STARK. |
| §5 envelope | `SHA-256` over a bind of amount, kind, height, and the rule fields. Re-checked in `seal`. Not Dilithium5. |
| §6 flow | Build, then miner re-check, then header hash. |
| §7 PoW | Not run. `work` is a block count, so heavier-chain is only a counter. Not Equihash-512. |
| §8 viewing keys | `H("vk", sk)` opens that key. A foundation string does not. |
| §9 MaskSwap | A spend whose price is outside `[p_a, p_b]` is rejected. Fees are not moved. |
| §10 bridge | A mint. Rejected below 8 votes or above 15. |
| §11 oracle | A spend whose three weights do not sum to 1 is rejected. |
| §12 emission | `sim/emission.py`. |
| §17 DAO | A spend under 5% turnout or 67% yes is rejected. |
