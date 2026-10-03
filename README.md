# Cypher-CIP

Research-stage Layer-1 design. One FRI STARK (the CIP circuit) checks a private transfer, with no trusted setup. Dilithium5 wraps the proof. This repository is the public spec drop plus the parts that actually run. It is not an audited protocol and it is not a public network.

## What runs

| Path | What it is |
| --- | --- |
| `crypto/cip` | Winterfell AIR. Goldilocks, blowup 8, 84 queries, grinding 20, quadratic extension: those four numbers are the §4.7 proposal. The in-circuit hash is `a³ + 3b³ + 7`. The paper's name for that role is Poseidon2. This gadget is not Poseidon2. |
| `crypto/dilithium` | FIPS 204. Dilithium5 is ML-DSA-87 (public key 2,592 bytes, signature 4,627). Dilithium2 is ML-DSA-44 (1,312 and 2,420). |
| `crypto/equihash` | Wagner / Blake2b. The paper's name Equihash-512 is Equihash(n=512, k=9). That pair is rejected: `k+1` does not divide `n`. The instance that solves is Equihash(48, 5). That is not Equihash-512. |
| `node` | In-process chain. Checks the proof, both signatures, the Equihash(48, 5) solution, and nullifier uniqueness. Longer work wins. Not a public P2P network. Timestamps are whatever the caller passes. |
| `sim/protocol_logic.py` | SHA-256 stand-in for rules the circuit does not cover (bridge quorum, swap range, DAO, oracle weights, viewing keys). |
| `sim/emission.py` | §12.3 derivation. The exact total matches; the printed column does not. |
| `docs/whitepaper-v1.4.pdf` | Design document. |
| `docs/STATUS.md` | What is done and what is not. |

`cargo test --release --workspace` runs the Rust checks. `python3 tests/test_protocol_logic.py` and `python3 sim/emission.py` run the rest.

## What this is not

- The in-circuit hash is `H(a, b) = a³ + 3b³ + 7`. The paper calls that role Poseidon2 (§4.4, §4.7). This gadget is not Poseidon2, and it is not collision-resistant. It is also not Rescue-Prime. Rescue-Prime is only the name §4.7.1 uses for a toy circuit standing in for Poseidon2. Winterfell's `Rp64_256` commitment hasher is a library hash, not this gadget.
- Equihash(48, 5) is not Equihash-512. Equihash-512, in the paper, is Equihash(n=512, k=9), and that pair is not a legal Equihash instance. A second illegal case, (512, 15), fails the 32-bit index word.
- The Dilithium5 signature is 4,627 bytes under FIPS 204. The paper's 4,595 is the round-3 draft.
- The field secret inside the circuit is not the Dilithium key. Dilithium signs the envelope outside the AIR.
- Nullifier uniqueness is a node check. One proof cannot see the chain.
- The chain in `node` is one process. It is not a public P2P testnet. It does not run 120-second blocks. The paper's block target of 120 seconds, and `T_target = 120` in the §7 retarget formula, are the paper's names. They are not what this process does. It is not cypheranon.com.
- No audit, no bounty, no mainnet.

## Verify a claim

- A circuit change needs a prove-and-verify test for the accept case and at least one reject case.
- A changed number needs the derivation in the PR.
- A benchmark needs the machine, and must name the hash it actually ran. Do not attach Poseidon2, Equihash-512, or a 120-second public testnet to a run that was not those things.

## Security

Pre-audit. No value should sit on this design. Report a forge, double-spend, or privacy break to Contact@Cypheranon.com, not as a public issue. See `SECURITY.md`.

## License

Apache-2.0 or MIT, at your option. See `LICENSE-APACHE-2.0` and `LICENSE-MIT`.
