# Phase 1 — Depth-parameterized anonymity set

**Status: partial. Depth 3 preserved. Depth 4 and 8 not proven.**

## What changed

Boundary assertion rows are now computed from `DEPTH`:

- `ROOT_SLOT = DEPTH`
- `NULL_SLOT = DEPTH + 1`
- `OUT_SLOT = DEPTH + 2`
- `EVENT_SLOT = DEPTH + 3`

Boundary loads sit at `DEPTH`, `DEPTH + 1`, `DEPTH + 2`.

For `DEPTH = 3` these equal the previous literals (rows 127, 159, 191, 223). Existing proofs still verify.

## Gate status

| Depth | Set | Proof + verify | Notes |
| --- | --- | --- | --- |
| 3 | 8 | Pass | Matches previous behavior. |
| 4 | 16 | Not implemented | Needs boundary array > 7, more sibling columns, and a re-derived `TRACE_LEN`. |
| 8 | 256 | Not implemented | Needs `TRACE_LEN` ≥ 512 and an 11-slot boundary array. |

## Why depth 4+ is blocked

The boundary array is `[E; 7]`. `TRACE_LEN` is 256 (8 slots of 32). Sibling columns are sized for 3 levels. Raising depth overwrites fixed loads or exceeds the array. A depth increase requires expanding these together and re-validating the AIR. That work is not done.

## Next step

Expand the boundary array, sibling columns, and `TRACE_LEN` so a depth-4 proof generates and verifies, then repeat for depth 8. Do not claim the gate until those tests pass.
