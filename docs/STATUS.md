# Status

Updated for the public tree as of the spec drop. "Named in docs" is not "shipped".

| Item | State | Blocks |
| --- | --- | --- |
| Whitepaper v1.4 | Published as `docs/whitepaper-v1.4.pdf` | Independent check of §19 |
| Statement 4, existence binding | Not in `cip-combined` | Soundness of the combined circuit |
| Hash gadget | Rescue128 / f128 stand-in | Parameter freeze, honest benchmarks |
| `cip-combined`, `cip-merkle-test` | Not in this repo | External review |
| Protocol simulator | `sim/protocol_logic.py`, hash stand-in, 600 s soak in `results/SOAK.md` | CIP soundness, real PoW |
| Node, consensus, genesis | Absent | Any deployment |
| Third-party audit | None | Mainnet |
| Bounty | None | — |
| Testnet | Not run. See `docs/RESEARCH-RESULTS.md` | Any deployment claim |

Exit for the next milestone: those two circuits are in-tree, statement 4 is folded, accept and reject tests pass, and the hash gadget is the proposed Goldilocks / Poseidon2-GL64 parameters or the README still calls it a stand-in.
