# Status

Updated for the public tree. A name in the paper is the paper's name. It is not the name of a stand-in unless the stand-in is that thing.

| Item | State |
| --- | --- |
| Whitepaper v1.4 | `docs/whitepaper-v1.4.pdf`. §12.3 column still disagrees with its formula; see `results/EMISSION.md`. |
| CIP AIR | `crypto/cip`. Four statements, Winterfell, no trusted setup. Accept and wrong-nullifier reject. In-circuit hash is `a³ + 3b³ + 7`. The paper's name for that role is Poseidon2. This gadget is not Poseidon2. |
| Dilithium5 / Dilithium2 | `crypto/dilithium`, FIPS 204 via `fips204`. Signature 4,627 bytes, not the paper's 4,595. |
| Equihash | `crypto/equihash`. The paper's Equihash-512 is (512, 9), rejected (`NotDivisible`). (512, 15) is rejected (`IndexWord`). The solving instance is Equihash(48, 5). That is not Equihash-512. |
| Node | `node`. In-process. Proof, Dilithium2 user signature, Dilithium5 envelope, Equihash(48, 5), nullifier set, heavier chain. |
| Local chain | A few blocks inside `node`'s tests. Fixed 4-bit target on Equihash(48, 5). Not a public P2P testnet. Not 120-second blocks. Not a 500-block retarget. Counters from that run are in `results/LOCAL_CHAIN.md`. |
| Difficulty formula | `adjust_difficulty` is the paper's `D_new = D_old * (T_target * 500) / sum(t_i)`. The paper's `T_target` is 120 seconds. The function is unit-tested with that number. It is not applied to the short local chain, and that chain does not run 120-second blocks. |
| Protocol stand-in | `sim/protocol_logic.py`. SHA-256. Bridge, swap, DAO, oracle, viewing key. Not the AIR. |
| Emission | `sim/emission.py`. Floor of the five-year sum is 964,940,762,634. |
| CI | `.github/workflows/test.yml` runs the Python checks and `cargo test --release --workspace`. |
| Audit / bounty | None. |
| Not in this tree | Poseidon2. Equihash-512. A public P2P testnet. 120-second block production. Recursive STARKs. Note encryption. |

Exit for the next milestone: replace `a³ + 3b³ + 7` with a collision-resistant gadget and use the paper's name Poseidon2 only if the gadget is Poseidon2. Do not call Equihash(48, 5) Equihash-512. Do not call the in-process chain a public P2P testnet or a 120-second chain.
