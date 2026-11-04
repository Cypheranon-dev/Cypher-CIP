# Version

**CIP-2026.9 — hardened production demo** (November 2026)

This is a hardened demo, not a mainnet. Every listed hole has a practical fix. Claims match the code.

## Fixes against the audit list

| Hole | Fix in this build |
| --- | --- |
| ML-DSA not inside the AIR | Every spend now requires `mldsa::verify_key_binding` on the Dilithium2 key that signed. The FIPS 204 signature and the Poseidon2 key commitment must match. Full NTT-in-AIR is not in this demo. |
| Recursive AIR does not prove spends | `aggregate` still verifies each proof via `verify_bytes` before folding. The node rejects a fold that was not built that way. |
| 64-bit field spend key | Spend acceptance requires the Dilithium key binding. New wallets should use `field_seed_from_pk`. The field element alone is no longer sufficient at the node. |
| Weak Equihash | Production mode enforces an 8-bit difficulty floor. (48, 5) remains the legal instance. (512, 9) does not construct. |
| No P2P | `Node::broadcast` signs the block id with Dilithium2 and announces to the peer list. `verify_announcement` checks it. |
| Caller timestamps | Production mode rejects jumps larger than 4× the recent median interval (and still rejects > 1 day). |
| Depth-3 tree / no encryption | Depth stays 3 in this AIR. `encrypt_note` is the wire format for note payloads. Anonymity set is not mainnet-grade. |
| Single infra key | `enable_production` requires at least 2 infrastructure keys and sets threshold to 2. |
| No audit | No third-party audit. SECURITY.md states that. Do not put funds on this. |
| Recovery / mesh design-only | Recovery stays opt-in and off. No validator reverse power was added. |

## Still not mainnet

- Full ML-DSA equations inside the STARK
- Full Winterfell verifier circuit
- Public authenticated gossip network
- External audit
- Anonymity set larger than 8

Do not put funds on this build.
