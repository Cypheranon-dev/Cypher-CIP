# Research results

Date: 2026-10-03. Tree: `dbbd8b2` (`main`).

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
