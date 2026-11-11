# Phase 2 — Recursion spike

**Status: comparison only. No production verifier.**

## Paths

### A. Port the AIR to Plonky2 or Plonky3

Plonky2 and Plonky3 already use Goldilocks and a Poseidon-family hash. A port would rewrite the CIP spend constraints in their gate system and gain their recursive verifier for free.

Cost: a full AIR rewrite, a new proving stack, and a migration of the node verifier. Benefit: recursion is a library feature, not a custom circuit. Risk: the existing Winterfell proofs, the pinned proof options, and the node integration are thrown away.

### B. Custom Winterfell verifier AIR

A genuine recursive verifier constrains the inner Winterfell verification: FRI layer checks, Merkle openings, the fold, the final degree check, and DEEP/OOD consistency. This stays in the current stack.

Cost: a large new AIR, estimated in the hundreds of thousands of constraints, plus a cross-check against Winterfell's native verifier. Benefit: no stack migration. Risk: long implementation, easy to get the FRI relation subtly wrong.

## Recommendation

**Stay on Winterfell and do not schedule a full verifier circuit yet.** The current Poseidon2 fold plus per-spend `verify_bytes` is the correct interim. A Plonky port is only justified if recursion becomes a launch requirement and the Winterfell verifier AIR is estimated to cost more than the migration. That estimate does not exist yet.

## Prototype

No prototype is included. A minimal "verify one proof inside another" loop requires either a Plonky dependency or a partial FRI circuit. Neither is small enough to land as a spike without becoming the production verifier by accident. The comparison above is the gate artifact.

## Gate

Comparison written. Path chosen: stay on Winterfell, recursion remains out of launch scope until an owner decision. No further recursion work is scheduled.
