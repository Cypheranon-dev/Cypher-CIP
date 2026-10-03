# Protocol

What the Rust crates enforce. A name here is the name of the code. The paper's Equihash-512 and a public 120-second network are not this protocol.

## Hash

Poseidon2 over Goldilocks, width 8, alpha 7, four initial full rounds, 22 partial rounds, four final full rounds. A digest is the first four lanes (256 bits). The permutation test vector is the Plonky3 Goldilocks vector in `crypto/cip`.

`Rp64_256` is only Winterfell's commitment hasher. It is not the note hash.

## Notes

Eight leaves, depth 3. An all-zero leaf is empty.

A note is `Poseidon2(amount, sk, rho, domain, 0, 0, 0, 0)`. The domain separates a note from a nullifier and from an event. `amount` is under 2^32. `sk` is a field element, not a Dilithium key.

The nullifier is `Poseidon2(sk, rho, nullifier-domain, ...)`. The node stores nullifiers. The proof cannot see that set.

## Spend proof

One FRI proof, Winterfell, no trusted setup. Blowup 8, 84 queries, grinding 20, quadratic extension.

The proof binds all of the following. The node checks the same public values against its state.

- The spent note is a leaf of the public note root.
- The nullifier opens to that note's `sk` and `rho`.
- The output note opens to `amount_out`, the same `sk`, and a fresh `rho`.
- `amount_in = amount_out + fee`, each under 2^32, so the field equation cannot wrap.
- The event opens to `payload`, the block height, the previous block id (four 32-bit words), and the fee.

A transfer also carries a Dilithium2 signature over the proof and the public inputs, and a Dilithium5 envelope over the same bytes. The envelope key is the chain's infrastructure key. Neither key is `sk`.

## Block

Equihash(48, 5) over the header, then `commit(header, indices)` must have `difficulty_bits` leading zero bits. Equihash(48, 5) is not Equihash-512. The paper's `(512, 9)` does not construct.

The miner signs the header with Dilithium2.

Transfers in one block are checked against the root from the start of the block. Outputs are appended into empty leaves after every transfer in the block has been checked. A note created in a block cannot be spent in that block.

Coinbase is optional. If present, its commitment must be the note for `block_reward(height)`, the miner's field `sk`, and `rho = height`. The reward is `1,522,069` in year 0 and `floor(previous * 5 / 8)` each later year of 262,800 blocks. That is the integer form of §12.2. The printed §12.3 column is still not this schedule; see `results/EMISSION.md`.

## Forks and difficulty

Heavier total work wins. Equal work does not replace the chain. Work per block is `2^bits`.

After 500 intervals, bits are retargeted with `D_new = D_old * (120 * 500) / sum(t_i)` and then stored as `floor(log2(D))`, clamped at 12. Timestamps are caller-supplied. This process does not produce a block every 120 seconds, and it is not a public P2P network.

## Not in consensus

`sim/protocol_logic.py` checks bridge quorum, swap range, DAO, oracle weights, and viewing keys with SHA-256. Those rules are not transactions on the node. Notes are not encrypted. There is no recursive block proof, no audit, and no bounty.
