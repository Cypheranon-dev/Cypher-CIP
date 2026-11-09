# Audit package

**Status: not audited.** This document prepares an independent review. It is not an audit. An implementing agent cannot certify an external audit.

**Project framing:** Private testnet / pre-mainnet audit candidate. Mainnet is not launched. Production use remains blocked pending independent review and explicit release approval.

## Threat model (summary)

- Adversary can generate blocks, craft proofs, and connect as a peer.
- Adversary cannot break Winterfell FRI under the pinned parameters, Poseidon2, or FIPS 204 Dilithium as used outside the proof.
- Single-process and small-peer deployments can be eclipsed. No Sybil defense is claimed.
- Depth-3 note tree does not provide a meaningful anonymity set.

## Consensus rules a reviewer should check

- Spend: Winterfell STARK binds note, nullifier, output, fee, height. See `crypto/cip/src/lib.rs` (`CipAir`).
- Coinbase: private spend key; AIR forces `payload == amount_in` when coinbase flag is 1; node checks flag, height, output, and reward. See `node/src/lib.rs` coinbase validation.
- Dilithium: FIPS 204 verification outside the proof. Key commitment is bound into the statement. This is not in-circuit signature verification. See `crypto/cip/src/mldsa.rs` and `dilithium_bind.rs`.
- Proof parameters: `verify` rejects any proof whose options differ from `proof_options()` (84 queries, blowup 8, grinding 20).
- Encoding: `pow_prefix` and `block_id` commit to transfers, coinbase, indices, coinbase key and signature.
- Chain selection: cumulative work, not height alone.
- Wire: frame and count caps in `node/src/wire.rs`.
- Note encryption: XChaCha20-Poly1305, fails closed without entropy.
- Handshake: hello carries `NETWORK_ID = b"cypher-testnet-0"`. Mismatch is rejected.

## Known limitations

- No full ML-DSA verification inside the STARK.
- No Winterfell verifier circuit. The recursive AIR is an absorb fold.
- No public gossip, peer discovery, scoring, or Sybil/eclipse defense.
- Anonymity set is 8 leaves. Capacity is not effective anonymity.
- Equihash(48, 5) at demo difficulty.
- No independent review.

## Reproducible checks

```
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace -- --test-threads=1
python3 tests/test_protocol_logic.py
python3 sim/emission.py
```

Deterministic vectors: Poseidon2 known-answer test in `crypto/cip/src/poseidon2.rs`; emission floors in `sim/emission.py`.
