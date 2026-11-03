# Version

**Current build: CIP-2026.5** (November 2026)

## Next step: M-of-N envelope

The infrastructure envelope is no longer only 1-of-N.

- `Chain::set_threshold(m)` requires M valid Dilithium5 signatures from the N listed keys.
- A transfer carries `envelope_sig` plus `extra_envelope_sigs`.
- Verification counts distinct keys that produce a valid signature. Fewer than M is rejected.
- Default remains 1, so existing single-key chains still work.
- M must be between 1 and N.

This is the step toward a threshold infrastructure key before any public network. It is still Dilithium5, not a light signature.

## Chain plan (unchanged)

- FRI STARK transfer, no trusted setup
- Dilithium5 envelope, Dilithium2 user signature
- Poseidon2
- Finite emission
- Equihash(48, 5) as the legal instance
- Recursive STARKs and in-circuit Dilithium are the following circuit steps

## Previous

- CIP-2026.4: degree fix, full Dilithium2 key required, 1-day timestamp cap
- CIP-2026.3: circuit v2 key binding
- CIP-2026.2: 1-of-N, note encryption, 24-bit cap
