# Version

**Current build: CIP-2026.3** (November 2026)

## CIP circuit v2

`crypto/cip/src/v2.rs` is the new circuit.

- Spend is bound to a Dilithium public key via a Poseidon2 commitment (`DOM_PK = 8`).
- `pk_commitment(pk)` hashes the key.
- `bind_spend` / `verify_binding` attach and check the binding.
- `bound_field_seed` derives the field spend seed from the Dilithium key.
- The node must check the Dilithium signature against the committed key.
- Full in-circuit Dilithium verification remains a later AIR. v2 is the binding circuit.

## Previous

- CIP-2026.2: 1-of-N infra, note encryption, 24-bit cap, peer list.
- CIP-2026.1: coinbase sk not serialized, 2-hour timestamp bound, opt-in recovery.
