# Contributing to Cypher

Thanks for even considering it. A few things worth knowing before you do.

## The one rule that actually matters

**If you say something works, it has to actually run.** This project's whole
history so far is a series of corrections: an emission table that didn't
match its own formula, signature sizes cited from a draft standard instead
of the final one, a security figure that counted bits it hadn't earned yet.
Every one of those got caught by someone actually computing the thing
instead of trusting how it read. That's the standard here.

Concretely:

- A PR claiming a circuit "works" needs a test that actually proves and
  verifies — both the accept case *and* at least one reject case. A circuit
  that only ever accepts isn't tested, it's decorated.
- A PR changing a number in the whitepaper (emission, security bits,
  throughput, anything) needs the derivation, not just the new number.
- If you're reporting a benchmark, say what hardware and name the function
  that ran. Equihash(48, 5) is not Equihash-512. An in-process chain is not a public P2P testnet and does
  not run 120-second blocks. Don't round in a way that hides what happened.
- If something doesn't work yet, say so in the PR instead of leaving it
  for someone else to discover. "This passes 3 of 4 test cases, here's the
  one that doesn't and why" is a genuinely useful contribution.

## What's useful to work on right now

Check open issues first, but broadly:

- Note encryption, and a viewing key that opens a note. The node stores commitments only.
- A legal Equihash parameter set with a real cost model. The paper's Equihash-512 is `(512, 9)`, and that pair is rejected on purpose. Do not call Equihash(48, 5) Equihash-512.
- Moving bridge, swap, DAO, and oracle out of `sim/protocol_logic.py` and into the node, with the same accept and reject tests.
- Independent verification of whitepaper §19 entries marked unreproduced.


## How to submit a change

1. Fork, branch, make the change
2. Include the test / derivation / reproduction that backs it up
3. Open a PR describing what you checked, not just what you changed
4. Be patient — this is a small, pseudonymous team; review isn't instant

## Code style

Rust: `cargo fmt` before committing, `cargo clippy` shouldn't complain.
Python: no strict formatter enforced yet, just keep it readable.

## Reporting issues vs. security issues

Normal bugs, docs problems, unclear writing → open a GitHub issue.
Anything that could let someone forge a proof, double-spend, or break
privacy → see `SECURITY.md`, don't open a public issue first.
