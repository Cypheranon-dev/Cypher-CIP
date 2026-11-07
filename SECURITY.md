# Security

**Private testnet / pre-mainnet audit candidate.** Mainnet is not launched. Production release remains gated on independent audit, sustained testnet validation, and explicit project-owner approval.

The source repository is public. No mainnet is deployed. A green CI run is not proof of protocol security.

## Material limitations

- Full ML-DSA verification inside the STARK is not implemented. The signature is checked outside the proof.
- The recursive AIR is a constrained fold, not a Winterfell verifier circuit.
- Peer protocol is hello, work-based sync, and a signed announcement. It is not a gossip network and has no Sybil or eclipse defense.
- Note tree is depth 3 (8 leaves). This is not a meaningful anonymity set.
- Equihash(48, 5) is used at demo difficulty.
- No independent audit has been completed.

## What this build enforces

- Spend proofs are accepted only under the protocol-pinned Winterfell parameters.
- Envelope requires M of N infrastructure signatures.
- Coinbase spend key is not serialized and is not derived from a public value.
- Coinbase amount is constrained in the AIR when the coinbase flag is 1.
- Note encryption is XChaCha20-Poly1305 and fails closed without entropy.
- Wire parser enforces frame and count caps.
- Hello rejects a mismatched network identifier.

## Reporting

Report issues against this repository. Do not treat this tree as suitable for real funds until the mainnet gates in `docs/STATUS.md` are evidenced.
