# Security Policy

Cypher is a research-stage, pre-audit project. Nothing here has been formally
reviewed by a third party yet. Treat everything in this repository as
experimental, and please tell us if you find a problem — cryptographic,
economic, or otherwise.

## Reporting a vulnerability

Email **Contact@Cypheranon.com** with:

- what you found and why it matters (a broken assumption, a forgeable proof,
  a double-spend path, an economic exploit — whatever it is)
- the smallest reproduction you can manage (a test case, a script, a trace)
- your assessment of severity, if you have one

Please don't open a public GitHub issue for anything that could let someone
steal funds, forge a proof, or break privacy before we've had a chance to
look at it. Everything else — typos, broken links, documentation issues — is
fine as a normal public issue.

## What's actually in scope right now

This is early. The things most worth attacking:

- the CIP circuit logic in `cip-combined` and `cip-merkle-test` (does it
  accept something it shouldn't, or reject something valid?)
- the emission/tokenomics math in the whitepaper — we've already found and
  fixed one real bug here (see the whitepaper's changelog) by just running
  the numbers; there may be more
- the protocol-logic simulator's attack scenarios — are there cases it
  doesn't cover?

We don't have a bug bounty program yet. If that changes, it'll be announced
here and at cypheranon.com.

## Response time

No formal SLA at this stage — we're a small, pseudonymous team. We'll
acknowledge reports as fast as we can and credit you (by whatever name you
choose, including staying anonymous) once a fix ships, if you want credit.
