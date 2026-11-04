# Audit scope

**Status: not audited.** This document prepares a review. It is not an audit.

## What a reviewer would examine

- `crypto/cip` AIR: constraint degrees, public inputs, nullifier binding, conservation
- Dilithium binding: payload equals key-commitment lane; signature checked outside
- Coinbase: publicly determined by reward, `field_seed(coinbase_pk)`, height
- Chain selection: `consider` by cumulative work
- Wire parser: frame and count limits
- Note encryption: custom authenticated stream; not a vetted AEAD
- Persistence: threshold and production flag round-trip

## Known limitations to confirm

- No in-circuit ML-DSA verification
- No recursive verifier circuit
- No public gossip with Sybil defense
- Depth-3 note tree
- Equihash(48, 5)
- No independent review

## Build

```
cargo test -p cypher-cip --lib
cargo test -p cypher-node --lib
cargo clippy --all-targets -- -D warnings
```

Contact: Contact@Cypheranon.com
