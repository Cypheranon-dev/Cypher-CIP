# Research results

## Current

There is still no CIP circuit, Dilithium5 envelope, Equihash-512 miner, node, or testnet in this tree. `sim/protocol_logic.py` is a SHA-256 stand-in. Its accept and reject cases run with `python3 tests/test_protocol_logic.py`. The 600 s counters in `results/SOAK.md` are a retired loop, not a result. Emission is derived in `results/EMISSION.md`.

## At dbbd8b2

Date: 2026-10-03. Tree: `dbbd8b2`.

Request: run a testnet on the CIP circuit and publish the result.

## What was run

Nothing. There is no circuit in this commit to prove, and no node to start.

Checked paths:

| Path | Present |
| --- | --- |
| `cip-combined` | No |
| `cip-merkle-test` | No |
| Protocol-logic simulator | No |
| Node, consensus, genesis | No |
| Prove/verify tests | No |
| CI | No |

`docs/whitepaper-v1.4.pdf` is a design document. It is not an executable circuit.

## Result

No testnet. No proof. No verify. No throughput, proof size, or security-bit number is reported here, because none was measured.

This is the result. A later file that claims a run has to name the commit that contains the circuit, the accept case, the reject case, the field, the hash gadget, and the hardware.

## What blocks a real run

- Statement 4 (existence binding) is not folded into `cip-combined`, and `cip-combined` is not in the tree.
- The in-circuit hash is still the f128 / Rescue128 stand-in. Proposed parameters are Goldilocks / Poseidon2-GL64. A stand-in benchmark would not be a Poseidon2 result.
- Whitepaper §19 entries marked unreproduced have not been independently checked.

Exit for the next results file: those two circuits are in-tree, statement 4 is folded, an accept test and a reject test both run, and the hash gadget is the proposed parameters or this file still calls it a stand-in.

## Unreproduced prior session

An earlier chat described a 600 s mainnet-style soak of a three-layer node (`mainnet/node.py`, `apps.py`, `emission.py`) and a live snapshot at 421 s: height 5,635, 4,437 CIP accepts, 794 rejects, 1,832 attacks blocked, 0 leaks. That process and its log are not in this repository and are not on this machine. The figures are not a result of `a8a8d9a` or of any commit on `main`.

Checked against `docs/whitepaper-v1.4.pdf`, the same write-up also used stand-ins the paper does not specify:

- Envelope in that run was WOTS+ over `(bind || event_id)`. §5 specifies CRYSTALS-Dilithium5 over `(CIP_proof || event_id)`.
- Consensus in that run was Equihash-lite with a 12-bit soak cap. §7 specifies Equihash(n=512, k=9).
- Recursive block STARKs and public P2P were absent there, as they are here.

Until that code is committed and the accept case, the reject case, and the 600 s run are repeated from the committed tree, the 421 s snapshot stays unreproduced.
