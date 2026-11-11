# Gossip and peer scoring

**Status: partial. Not a public gossip network.**

## Implemented

- Hello with network identifier.
- Work-based block sync.
- Announcement structure and id validation.
- Peer score: `note_good` increments (cap 100), `note_bad` decrements by 5.
- Ban threshold: score below -10.
- Connection cap: 32 peers.

## Not implemented

- Peer discovery or bootstrap.
- Eclipse resistance (peer diversity by ASN/subnet).
- Sybil defense beyond the score (no identity cost).
- Bandwidth or proof-verification rate limits.
- Announcement application inside `serve_one` (caller must call `chain.consider`).

## Threat model

An attacker who can create many peers can still eclipse a node. The score and cap slow this but do not prevent it. A public network requires identity cost or diversity rules that are not present.

## Owner decision

Public gossip with Sybil/eclipse defense is not yet a launch property. The current deployment is a private/static-peer testnet.
