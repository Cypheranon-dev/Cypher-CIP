# Status

**CIP-2026.9 hardened production demo** (November 2026)

| Item | State |
| --- | --- |
| Spend proof | FRI STARK. Generates. Wrong nullifier rejected. |
| Dilithium key binding | Required on every spend. |
| ML-DSA in AIR | Binding + FIPS 204 on the committed key. Not full NTT-in-circuit. |
| Recursive aggregate | Verifies proofs, then folds. |
| Envelope | M-of-N. Production mode requires 2-of-N. |
| PoW | Equihash(48, 5). Production floor 8 bits. |
| Timestamps | Median check in production mode. 1-day cap always. |
| Peers | Signed broadcast to the peer list. |
| Note encryption | Available. Not a large anonymity set. |
| Coinbase sk | Not serialized. |
| Audit | None. Do not put funds on this. |
