# Local chain

One release run of `cargo test --release -p cypher-node -- --nocapture local_chain`. Not a public network, not 120-second blocks, not Equihash-512.

Machine: 2 cores, Intel Xeon Platinum 8481C at 2.70 GHz, 4,024,496 kB RAM. `rustc 1.98.1`.

| Counter | Value |
| --- | --- |
| FRI options | 84 queries, blowup 8, grinding 20, quadratic extension |
| STARK hash | Rescue-Prime (`Rp64_256`) |
| In-circuit hash | `a³ + 3b³ + 7`, not Poseidon2 |
| Prove | 8,329 ms |
| Proof | 16,409 bytes |
| Dilithium5 | public key 2,592, signature 4,627 |
| Dilithium2 | public key 1,312, signature 2,420 |
| Equihash | `(48, 5)`, target 4 leading zero bits |
| Miner root | 15109209295028550221 |
| Nullifier | 33282 |
| Event id | 913545492179214011 |
| Proof height | 1 |
| Honest chain | height 2, work 32 |
| Heavier reorg | height 3, work 48, nullifier set cleared |
| Equal work | a 2-block fork did not replace the 2-block chain |

Work per block is `2^4 = 16`. The retarget formula is unit-tested and was not applied; this chain is shorter than 500 intervals.

Rejects in the same run: a flipped Dilithium5 envelope, a proof whose public nullifier was incremented, a second spend of nullifier 33282, and an Equihash solution with the first two indices swapped. The CIP crate's own release test (accept, then verify with the nullifier incremented) finished in 18.88 seconds on the same machine.
