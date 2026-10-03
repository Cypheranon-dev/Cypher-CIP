"""Accept and reject cases for the hash stand-in. Not a CIP proof."""
import random
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from sim.protocol_logic import Chain, make_tx


def test_accept_then_reject_double_spend():
    rng = random.Random(1)
    chain = Chain()
    mint = make_tx(rng, 1, "mint", "sk", 10)
    ok, bad = chain.seal([mint])
    assert (ok, bad) == (1, 0)
    again = make_tx(rng, 2, "transfer", "sk", 1)
    again.r_in = mint.nullifier
    again.sig = __import__("sim.protocol_logic", fromlist=["h"]).h("env", again.bind, again.event_id)
    # rebuild a spend of the minted note, then replay it
    spend = make_tx(rng, 2, "transfer", "sk", 10)
    spend.r_in = mint.nullifier
    spend.amount = 10
    spend.bind = __import__("sim.protocol_logic", fromlist=["h"]).h("bind", spend.sk, spend.r_in, spend.amount, spend.kind)
    spend.sig = __import__("sim.protocol_logic", fromlist=["h"]).h("env", spend.bind, spend.event_id)
    ok, bad = chain.seal([spend])
    assert ok == 1 and bad == 0
    replay = make_tx(rng, 3, "transfer", "sk", 10)
    replay.r_in = spend.nullifier
    replay.bind = __import__("sim.protocol_logic", fromlist=["h"]).h("bind", replay.sk, replay.r_in, replay.amount, replay.kind)
    replay.sig = __import__("sim.protocol_logic", fromlist=["h"]).h("env", replay.bind, replay.event_id)
    first = replay
    second_r = first.nullifier
    # same nullifier twice in one batch
    twin = make_tx(rng, 4, "transfer", "sk", 1)
    twin.r_in = spend.nullifier
    twin.bind = __import__("sim.protocol_logic", fromlist=["h"]).h("bind", twin.sk, twin.r_in, twin.amount, twin.kind)
    twin.event_id = first.event_id
    twin.sig = "flipped"
    ok, bad = chain.seal([twin])
    assert ok == 0 and bad == 1


def test_bridge_and_swap_reject():
    rng = random.Random(2)
    chain = Chain()
    low = make_tx(rng, 1, "bridge", "sk", 1, votes=7)
    wide = make_tx(rng, 1, "swap", "sk", 1, pa=1, pb=100, p=200)
    ok, bad = chain.seal([low, wide])
    assert ok == 0 and bad == 2


if __name__ == "__main__":
    test_accept_then_reject_double_spend()
    test_bridge_and_swap_reject()
    print("accept and reject cases passed")
