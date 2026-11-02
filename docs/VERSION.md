# Version

**Current build: CIP-2026.1** (November 2026)

This is the version under active development.

## What changed from the research tree

- Coinbase field `sk` is no longer serialized or included in the PoW prefix.
- Timestamps more than 2 hours ahead of the previous block are rejected.
- Recovery is opt-in and user-authorized (not a validator power).
- CIP-2026 targets (privacy+compliance, quantum-safe opt-in recovery, offline mesh) are the design baseline.

## Still research debt

- 64-bit field spend key must move into a Dilithium key inside the STARK.
- Single infra Dilithium5 key must become a threshold key before any public network.
- Equihash(48, 5) and the 12-bit clamp are temporary.
- No public P2P, no note encryption, no recursive STARKs.

See `docs/CIP-2026.md` and `docs/STATUS.md`.
