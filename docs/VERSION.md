# Version

**CIP-2026.11** — private testnet / pre-mainnet audit candidate.

Changes from CIP-2026.10:

- Proof verification accepts only the protocol-pinned Winterfell options.
- Hello handshake carries and checks a network identifier (`cypher-testnet-0`).
- Status, security, and audit documents use release-gated language and match the current source.

Unchanged blockers: full ML-DSA inside the STARK, Winterfell verifier circuit, public gossip with Sybil defense, anonymity set larger than 8, external audit.
