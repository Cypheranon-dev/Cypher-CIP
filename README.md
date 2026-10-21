# Cypher-CIP

**Protocol foundation for Cypher — private by default, post-quantum ready.**

Research and implementation of the Cypher Improvement Proposal (CIP): a FRI STARK transfer system, Dilithium signatures, and an in-process chain. Designed to solve the memory and performance limits of current shielded protocols, starting with Equihash(48, 5) and moving toward scalable zk-STARK privacy without a trusted setup.

This is the core protocol repo. User-facing clients are [Mask Wallet](https://github.com/Cypheranon-dev/Maskwallet) and [Mask Swap](https://github.com/Cypheranon-dev/Maskswap).

## What this builds toward

- Memory-efficient shielded transactions via zk-STARKs (replacing heavy Equihash 48,5 constraints)
- Post-quantum signatures (Dilithium5 / ML-DSA-87)
- Finite emission, GPU-first decentralized mining, and DAO governance
- Private transfers and swaps with no trusted setup
- Compatibility path with Zcash while upgrading the proof system

## Current implementation

What is implemented, what is missing, and which paper names this tree does not yet implement is in [docs/STATUS.md](docs/STATUS.md). The design document is [docs/whitepaper-v1.4.pdf](docs/whitepaper-v1.4.pdf).

## Build

```bash
cargo test --release --workspace -- --test-threads=1
python3 tests/test_protocol_logic.py
python3 sim/emission.py
```

The chain check, including rejects and a heavier-chain replacement:

```bash
cargo test --release -p cypher-node -- --nocapture local_chain
```

Proving is release-mode and takes about half a minute. CI runs the same commands.

## Tree

| Path | Role |
| --- | --- |
| `crypto/cip` | Winterfell spend AIR. The hash is Poseidon2 over Goldilocks, width 8. A digest is four lanes. |
| `crypto/dilithium` | FIPS 204. Dilithium5 is ML-DSA-87. Dilithium2 is ML-DSA-44. |
| `crypto/equihash` | Solves Equihash(48, 5). The paper's Equihash-512, `(512, 9)`, is rejected. This is the starting point for memory optimization. |
| `node` | Chain process. `cypher-node init` and `cypher-node run` mine, store, and hand blocks to one peer. Equihash(48, 5) today; path to lighter proofs. |
| `sim/` | SHA-256 stand-in for bridge, swap, DAO, oracle, and viewing keys. Emission check for §12. |
| `docs/PROTOCOL.md` | The rules the Rust crates enforce. |

## Related repos

- [Mask Wallet](https://github.com/Cypheranon-dev/Maskwallet) — Zodl-inspired Cypher wallet with Zcash integrated
- [Mask Swap](https://github.com/Cypheranon-dev/Maskswap) — Private swap interface for the Cypher ecosystem

## Security

Pre-audit. Report a forge, double-spend, or privacy break to Contact@Cypheranon.com. See `SECURITY.md`.

## License

Apache-2.0 or MIT, at your option. See `LICENSE-APACHE-2.0` and `LICENSE-MIT`.

---

Private by default. Provable by choice.
