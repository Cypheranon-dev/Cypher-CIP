# Version

**Current build: CIP-2026.4** (November 2026)

This is the full CIP chain, not a light version: FRI STARK spend, Dilithium2 user signature, Dilithium5 envelope, Poseidon2, no trusted setup.

## Fixed and hardened in CIP-2026.4

- STARK constraint degrees now match Winterfell 0.13. The spend proof generates and rejects a wrong nullifier.
- Spend path requires a full Dilithium2 public key (1312 bytes) and a v2 Poseidon2 key commitment. Empty or short keys are rejected.
- Timestamp cap is 1 day. Timestamps stay caller-supplied, as in the paper. Only absurd jumps are rejected.
- Protocol doc states this is the full chain.

## Chain plan (unchanged)

- FRI STARK transfer, no trusted setup
- Dilithium5 / ML-DSA-87 envelope, Dilithium2 user signature
- Poseidon2 note, nullifier, event
- Finite emission: 1,522,069 then floor(previous * 5 / 8) per year of 262,800 blocks
- Equihash(48, 5) is the legal instance. The paper's (512, 9) does not construct.
- Path to recursive STARKs and in-circuit Dilithium is documented, not dropped

## Previous

- CIP-2026.3: circuit v2 Dilithium key binding
- CIP-2026.2: 1-of-N infra, note encryption, 24-bit cap, peer list
- CIP-2026.1: coinbase sk not serialized, opt-in recovery
