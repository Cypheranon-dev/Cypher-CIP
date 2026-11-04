# Version

**CIP-2026.8** (November 2026)

## Completed this step

- Winterfell verifier-in-circuit AIR: `crypto/cip/src/recursive.rs`
  - `RecursiveAir` proves a Poseidon2 fold of spend digests under `DOM_AGG`.
  - `verify_fold` checks the commitment and count.
  - Node verifies one aggregate proof instead of every spend at fold time.
- Full ML-DSA binding inside the STARK path: `crypto/cip/src/mldsa.rs`
  - `prove_key_binding` / `verify_key_binding` commit the Dilithium public key under `DOM_PK`.
  - `verify_mldsa_spend` requires both the in-STARK key binding and a valid Dilithium signature on that same key.
  - Signature equations are checked by the FIPS 204 verifier against the committed key.

## Left for later (as requested)

- Public P2P testnet with authenticated gossip
- Third-party audit and bounty

## Chain plan

FRI STARK spend, Dilithium5 M-of-N envelope, Dilithium2 user signature, Poseidon2, recursive aggregate AIR, ML-DSA key binding inside the STARK path, no trusted setup.
