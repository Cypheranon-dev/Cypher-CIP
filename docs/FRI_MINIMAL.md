# Minimal FRI Verification Circuit (Research Prototype)

**Status: design + skeleton only. Not a recursive verifier. Not production.**

This document describes a minimal outer AIR that constrains a single degree-respecting fold step and binds a layer commitment. It is a research prototype toward Milestone 4. It does **not** verify an inner STARK proof.

## What it is

- A standalone AIR (`crypto/cip/src/fri_minimal.rs`).
- Constrains that four evaluations fold correctly under a challenge \(\alpha\):
  \(\text{folded} = e_0 + \alpha\cdot e_1 + \alpha^2\cdot e_2 + \alpha^3\cdot e_3\).
- Public inputs bind the layer commitment (digest) and the expected folded value.
- Optional single-position Merkle opening can be added later.

## What it is not

- Not a FRI verifier.
- Does not parse a Winterfell `FriProof`.
- Does not check multiple layers, remainder degree, or transcript.
- Does not re-execute an inner AIR.
- Does not replace native `verify_bytes` or `verify_fold`.
- Passing this circuit is **not** evidence that an inner proof is valid.

## Parameters (pinned to current `proof_options`)

- Base field: Goldilocks
- Folding factor: 4
- Blowup: 8
- Hash: Rp64_256

## Trace layout (fixed)

| Cols | Purpose |
|------|---------|
| 0-3  | Layer commitment (public) |
| 4-7  | Evaluations \(e_0..e_3\) |
| 8-11 | Challenge \(\alpha\) (public) |
| 12-15| Folded result (constrained) |
| 16   | Boundary / checked-row flag |

## Constraints

- On the checked row: folded result equals the degree-4 linear combination of the evaluations under \(\alpha\).
- Assertions bind the commitment and the expected folded value as public inputs.

## Tests

- Satisfiable fold verifies.
- Wrong evaluation, wrong \(\alpha\), or wrong commitment fails.

## Next steps (not in this PR)

1. Add a real Merkle opening constraint for one position.
2. Chain two fold steps.
3. Document proving cost.
4. Keep full recursive verification as an open blocker.

**Mainnet is not approved. This is research only.**

## Implementation note

The skeleton AIR in `crypto/cip/src/fri_minimal.rs` is present and compiles.  
A non-constant step-counter column is now included so the composition polynomial has the expected degree. Tests pass in release mode.  
The skeleton now proves and verifies a satisfiable fold and rejects a mismatched public folded value.  
This is left as the immediate next step. The design and limitations remain as stated above.

