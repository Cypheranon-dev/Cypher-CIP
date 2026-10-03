"""Accept and reject cases for the hash stand-in. Not a CIP proof."""
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from sim.emission import BLOCKS, PAPER_ISSUED, PAPER_TOTAL, row
from sim.protocol_logic import Chain, heavier, make_tx, opens, viewing_key


def spend(chain: Chain, note: str, amount: int, kind: str = "transfer", **extra):
    return make_tx(chain.height + 1, kind, "sk", amount, note, f"e-{chain.height}-{kind}-{amount}", **extra)


def test_emission_matches_stated_total():
    rows = [row(y) for y in range(5)]
    assert int(sum(rows, start=0)) == PAPER_TOTAL
    assert sum(int(item) for item in rows) == PAPER_TOTAL - 1
    assert sum(PAPER_ISSUED) != PAPER_TOTAL
    assert row(1) == PAPER_ISSUED[1]
    assert BLOCKS == 262_800


def test_double_spend_and_same_block():
    chain = Chain()
    minted = make_tx(1, "mint", "sk", 10, "fresh", "m1")
    assert chain.seal([minted])[0] == 1
    note = minted.nullifier
    first = spend(chain, note, 10)
    second = spend(chain, note, 10)
    second.event_id = "e-replay"
    from sim.protocol_logic import sign
    sign(second)
    ok, bad, reasons = chain.seal([first, second])
    assert (ok, bad, reasons) == (1, 1, {"double-spend": 1})
    assert chain.seal([spend(chain, note, 10)])[2] == {"double-spend": 1}


def test_envelope_binds_the_fields():
    chain = Chain()
    minted = make_tx(1, "mint", "sk", 10, "fresh", "m1")
    chain.seal([minted])
    tx = spend(chain, minted.nullifier, 10)
    tx.sig = "flipped"
    ok, bad, reasons = chain.seal([tx])
    assert (ok, reasons) == (0, {"envelope": 1})
    assert minted.nullifier in chain.notes


def test_conservation_membership_and_height():
    chain = Chain()
    minted = make_tx(1, "mint", "sk", 10, "fresh", "m1")
    chain.seal([minted])
    short = spend(chain, minted.nullifier, 4)
    assert chain.seal([short])[2] == {"conservation": 1}
    assert chain.notes[minted.nullifier] == 10
    missing = spend(chain, "absent", 10)
    assert chain.seal([missing])[2] == {"membership": 1}
    early = make_tx(9, "mint", "sk", 1, "other", "m9")
    assert chain.seal([early])[2] == {"height-bind": 1}


def test_swap_bridge_dao_oracle():
    chain = Chain()
    minted = make_tx(1, "mint", "sk", 5, "fresh", "m1")
    chain.seal([minted])
    wide = spend(chain, minted.nullifier, 5, "swap", pa=1, pb=100, p=200)
    assert chain.seal([wide])[2] == {"range": 1}
    low = make_tx(chain.height + 1, "bridge", "sk", 5, "bridge-in", "b1", votes=7)
    high = make_tx(chain.height + 1, "bridge", "sk", 5, "bridge-in-2", "b2", votes=16)
    assert chain.seal([low, high])[2] == {"bridge-quorum": 2}
    good = make_tx(chain.height + 1, "bridge", "sk", 5, "bridge-in-3", "b3", votes=8)
    assert chain.seal([good])[0] == 1
    minted = make_tx(chain.height + 1, "mint", "sk", 3, "fresh-2", "m2")
    chain.seal([minted])
    dao = spend(chain, minted.nullifier, 3, "dao", turnout=0.04, yes=0.7)
    assert chain.seal([dao])[2] == {"dao": 1}
    minted = make_tx(chain.height + 1, "mint", "sk", 3, "fresh-3", "m3")
    chain.seal([minted])
    oracle = spend(chain, minted.nullifier, 3, "oracle", w=(0.5, 0.4, 0.2))
    assert chain.seal([oracle])[2] == {"oracle": 1}


def test_heavier_chain_and_no_god_key():
    left, right = Chain(), Chain()
    left.seal([make_tx(1, "mint", "a", 1, "la", "el")])
    right.seal([make_tx(1, "mint", "b", 1, "ra", "er")])
    assert heavier(left, right) == "tie"
    right.seal([make_tx(2, "mint", "b", 1, "rb", "er2")])
    assert heavier(left, right) == "b"
    assert opens(viewing_key("sk"), "sk")
    assert not opens("foundation", "sk")
    assert not opens(viewing_key("other"), "sk")


if __name__ == "__main__":
    test_emission_matches_stated_total()
    test_double_spend_and_same_block()
    test_envelope_binds_the_fields()
    test_conservation_membership_and_height()
    test_swap_bridge_dao_oracle()
    test_heavier_chain_and_no_god_key()
    print("accept and reject cases passed")
