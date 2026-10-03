# Cypher-CIP

Research-stage specification for a Layer-1 that proves two things in one STARK circuit, with no trusted setup:

- Post-quantum authorization under Dilithium5 (NIST FIPS 204).
- Mandatory, default-private transfer (nullifiers, binding, no transparent amounts).

The name CIP is the circuit, not a standards process. This repository is the public spec drop. It is not a node, a testnet, or an audited protocol.

## What is in this tree

| Path | Status |
| --- | --- |
| `docs/whitepaper-v1.4.pdf` | Design document. Numbers in §19 marked unreproduced are not claims. |
| `SECURITY.md` | How to report a forge, double-spend, or privacy break. |
| `CONTRIBUTING.md` | A result has to run. Accept-only circuits are not tests. |
| `docs/STATUS.md` | Gap list. This is the source of truth for "done". |
| `docs/RESEARCH-RESULTS.md` | 2026-10-03 attempt. No circuit, so no testnet. |
| `results/SOAK.md` | 600 s hash stand-in. Not a CIP proof. |

Not in this tree, despite being named in the docs: `cip-combined`, `cip-merkle-test`, the protocol-logic simulator, a node, genesis, or CI. Do not treat those as implemented.

## Open gaps

These block any network:

- Statement 4 (existence binding) is not folded into `cip-combined`.
- In-circuit hash is still f128 / Rescue128. The proposed parameters are Goldilocks / Poseidon2-GL64. Benchmarks on the stand-in are not Poseidon2 numbers.
- Whitepaper §19 entries marked unreproduced have not been independently checked.
- No public proof/verify tests, so the circuit cannot yet be reviewed.

File those as issues from `.github/ISSUE_TEMPLATE` rather than restating them in chat.

## Verify a claim

- A circuit change needs a prove-and-verify test for the accept case and at least one reject case.
- A changed number (emission, bits, throughput, sizes) needs the derivation in the PR.
- A benchmark needs hardware, and must say what is real versus a stand-in.

## Security

Pre-audit. No bounty. No value should sit on this design. Report forge, double-spend, and privacy breaks to Contact@Cypheranon.com, not as a public issue. See `SECURITY.md`.

## License

Apache-2.0 or MIT, at your option. See `LICENSE-APACHE-2.0` and `LICENSE-MIT`.
