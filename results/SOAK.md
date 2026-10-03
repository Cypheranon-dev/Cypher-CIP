# Protocol-logic soak, 2026-10-03

Stand-in. Not a CIP proof, not Dilithium5, not Equihash-512, not a testnet.

Command: `python3 sim/protocol_logic.py` for 600 wall-clock seconds. Seed 20261003. Commit of this file is the run that produced these counters.

## What ran

Hash stand-ins for the envelope and the nullifier. A transaction is sealed only after the miner re-checks the envelope. Rejects cover a flipped envelope, a repeated `H(sk, r_in)` nullifier, a swap price outside `[p_a, p_b]`, a bridge under 8 of 15 votes, and a DAO vote under 5% turnout or 67% yes. A foundation god-key is not installed; scan uses a per-key viewing key. No proof-of-work search ran, so height is a count of rule-check batches, not Equihash blocks. Difficulty cap 12 was not exercised.

## Counters

| Item | Value |
| --- | --- |
| Wall clock | 600.0 s |
| Batches sealed | 4944923 |
| Accepts | 3956188 |
| Rejects | 35603196 |
| Leaks | 0 |
| God-key decrypts | 0 |
| Exceptions | 0 |
| Light-client failures | 0 |
| Header | `a34b3c17f42ebf7149990f6c28f363abbe4c3d1c15e491b82b262002e52bba6c` |

Snapshots were written every 30 s (21 rows). First snapshot t=30.0 s, height 254489. Last snapshot t=600.0 s.

## Emission check, separate from the soak

See `results/EMISSION.md`. Floor of the five-year sum is 964,940,762,634, the stated total. Floor of each year then sum is 964,940,762,633. The printed column sums to 964,941,029,434.

## What this does not prove

CIP soundness, Poseidon2, Dilithium5, Equihash(n=512, k=9), recursion, or P2P. Those are still absent from the tree.
