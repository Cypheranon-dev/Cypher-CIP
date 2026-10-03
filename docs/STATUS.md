# Status

Updated for the public tree. "Named in docs" is not "shipped".

| Item | State | Blocks |
| --- | --- | --- |
| Whitepaper v1.4 | Published as `docs/whitepaper-v1.4.pdf` | §12.3 column still disagrees with its formula; see `results/EMISSION.md` |
| Statement 4, existence binding | Not in `cip-combined` | Soundness of the combined circuit |
| Hash gadget | Not in this repo. Proposed Goldilocks / Poseidon2-GL64. The in-tree stand-in is SHA-256 | Parameter freeze |
| `cip-combined`, `cip-merkle-test` | Not in this repo | External review |
| Protocol simulator | Hash stand-in in `sim/protocol_logic.py`. `python3 tests/test_protocol_logic.py` | CIP soundness, real PoW |
| Node, consensus, genesis | Absent | Any deployment |
| Third-party audit | None | Mainnet |
| Bounty | None | — |
| Testnet | Not run | Any deployment claim |
| CI | `.github/workflows/test.yml` runs the stand-in tests | A circuit proof |

Exit for the next milestone: those two circuits are in-tree, statement 4 is folded, accept and reject tests pass, and the hash gadget is the proposed Goldilocks / Poseidon2-GL64 parameters or the README still calls it a stand-in.
