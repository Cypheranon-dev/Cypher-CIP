# Security

**CIP-2026.9 is a hardened demo. It is not hackproof. Do not put funds on it.**

## What an attacker still can do

- Brute-force Equihash(48, 5) at the demo difficulty.
- Eclipse a single-peer or small-peer deployment.
- Correlate the depth-3 note tree.
- Exploit any bug in the AIR that an audit would have caught.

## What this build closes

- Spend without a Dilithium key that opens the Poseidon2 commitment.
- Envelope signed by fewer than M of N keys.
- Coinbase field key on the wire.
- Timestamp jumps that smash the retarget in production mode.
- Unsigned block announcements.

## Reporting

Email Contact@Cypheranon.com. No bounty yet.
