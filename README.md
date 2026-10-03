# Cypher-CIP

Research code for a FRI STARK transfer check (CIP), Dilithium signatures, and an in-process chain. Not an audited protocol and not a public network.

What is missing, and which paper names this tree does not implement, is [docs/STATUS.md](docs/STATUS.md). The design document is [docs/whitepaper-v1.4.pdf](docs/whitepaper-v1.4.pdf).

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
| `crypto/equihash` | Solves Equihash(48, 5). The paper's Equihash-512, `(512, 9)`, is rejected. |
| `node` | In-process chain. Proof, both signatures, note tree, coinbase, retarget. Not a public P2P testnet and not 120-second blocks. Equihash(48, 5), not Equihash-512. |
| `sim/` | SHA-256 stand-in for bridge, swap, DAO, oracle, and viewing keys. Not consensus. Emission check for §12. |
| `docs/PROTOCOL.md` | The rules the Rust crates enforce. |

## Security

Pre-audit. Report a forge, double-spend, or privacy break to Contact@Cypheranon.com. See `SECURITY.md`.

## License

Apache-2.0 or MIT, at your option. See `LICENSE-APACHE-2.0` and `LICENSE-MIT`.
