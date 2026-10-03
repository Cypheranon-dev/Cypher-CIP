# Status

Updated for the public tree. A name in the paper is the paper's name. It is not the name of a stand-in unless the stand-in is that thing.

| Item | State |
| --- | --- |
| Whitepaper v1.4 | `docs/whitepaper-v1.4.pdf`. §12.3 column still disagrees with its formula; see `results/EMISSION.md`. |
| Protocol | `docs/PROTOCOL.md`. Spend, note tree, coinbase, retarget. |
| CIP AIR | `crypto/cip`. Poseidon2, Goldilocks, width 8. Digest is four lanes. Accept and wrong-nullifier reject. 32-bit amounts. Depth-3 note tree. |
| Dilithium5 / Dilithium2 | `crypto/dilithium`, FIPS 204 via `fips204`. Signature 4,627 bytes, not the paper's 4,595. |
| Equihash | `crypto/equihash`. The paper's Equihash-512 is (512, 9), rejected (`NotDivisible`). The instance that solves is Equihash(48, 5). That is not Equihash-512. |
| Node | `node`. In-process. Proof, both signatures, note root, event, nullifier set, output note, coinbase reward, heavier chain, retarget after 500 intervals. Bit target clamped at 12. |
| Local chain | Tests in `node`. Not a public P2P testnet. Not 120-second blocks. |
| Difficulty formula | `adjust_difficulty` is the paper's formula with `T_target = 120`. Applied after 500 intervals. The process does not wait 120 seconds. |
| App rules | `sim/protocol_logic.py`. SHA-256 stand-in for bridge, swap, DAO, oracle, viewing keys. Not consensus. |
| Emission | `block_reward` in the node. `sim/emission.py` checks the five-year total. |
| CI | `.github/workflows/test.yml` runs the Python checks and `cargo test --release --workspace`. |
| Audit / bounty | None. |
| Not in this tree | Equihash-512. A public P2P testnet. 120-second block production. Recursive STARKs. Note encryption. Bridge, swap, DAO, and oracle as consensus transactions. |

`results/RUN-2026-10-03.md` records the previous circuit, whose hash was `a³ + 3b³ + 7`. That run is not this tree.
