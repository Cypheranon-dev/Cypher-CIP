# Protocol-logic soak, 2026-10-03

Retired as a measurement. The 600 s loop in `cc32eb8` is not what `sim/protocol_logic.py` does now, and those counters were not the checks named in the headings.

What was wrong with that loop:

- A rejected bridge or swap was usually a missing note, so the quorum and range rules never ran.
- The injected "double spend" stored a new nullifier. It did not reuse `H(sk, r_in)`.
- Every reject was counted as an attack. Swap, oracle, bridge, and DAO totals counted submissions, including rejects.
- No proof-of-work ran. Height was how many times the loop called `seal`.

The historical counters are kept so the run is not relabeled as a pass:

| Item | Value |
| --- | --- |
| Wall clock | 600.0 s |
| Batches sealed | 4,944,923 |
| Accepts | 3,956,188 |
| Rejects | 35,603,196 |
| Leaks | 0 |
| Exceptions | 0 |
| Header | `a34b3c17f42ebf7149990f6c28f363abbe4c3d1c15e491b82b262002e52bba6c` |

Do not cite that table as evidence the rules hold. The check that runs is `python3 tests/test_protocol_logic.py`. Emission is `python3 sim/emission.py`, written up in `results/EMISSION.md`.
