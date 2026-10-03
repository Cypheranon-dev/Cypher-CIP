# Status

Updated for the public tree. "Named in the paper" is not "the paper's parameters".

| Item | State |
| --- | --- |
| Whitepaper v1.4 | `docs/whitepaper-v1.4.pdf`. §12.3 column still disagrees with its formula; see `results/EMISSION.md`. |
| CIP AIR | `crypto/cip`. Four statements, Winterfell, no trusted setup. Accept and wrong-nullifier reject. In-circuit hash is the cubic above, not Poseidon2. |
| Dilithium5 / Dilithium2 | `crypto/dilithium`, FIPS 204 via `fips204`. Signature 4,627 bytes, not the paper's 4,595. |
| Equihash | `crypto/equihash`. `(512, 9)` rejected (`NotDivisible`). `(512, 15)` rejected (`IndexWord`). Solving instance is `(48, 5)`. |
| Node | `node`. In-process. Proof, Dilithium2 user signature, Dilithium5 envelope, Equihash, nullifier set, heavier chain. |
| Local chain | A few blocks inside `node`'s tests. Fixed 4-bit target on `(48, 5)`. Not a 500-block retarget and not a public testnet. Counters from that run are in `results/TESTNET.md`. |
| Difficulty formula | `adjust_difficulty` is the paper's `D_new = D_old * (T_target * 500) / sum(t_i)`, unit-tested. It is not applied to the short local chain. |
| Protocol stand-in | `sim/protocol_logic.py`. SHA-256. Bridge, swap, DAO, oracle, viewing key. Not the AIR. |
| Emission | `sim/emission.py`. Floor of the five-year sum is 964,940,762,634. |
| CI | `.github/workflows/test.yml` runs the Python checks and `cargo test --release --workspace`. |
| Audit / bounty | None. |
| Not in this tree | Poseidon2, a legal Equihash-512, recursive STARKs, note encryption, P2P, a public testnet. |

Exit for the next milestone: replace the cubic hash with a collision-resistant gadget and label it only if the implementation is that gadget, and do not call `(48, 5)` Equihash-512.
