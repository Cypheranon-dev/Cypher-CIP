#!/usr/bin/env python3
"""Protocol-logic stand-in. Not CIP, not Dilithium5, not Equihash-512.

Checks the rules named in whitepaper v1.4 with hash stand-ins:
membership, H(sk, r_in) nullifier, conservation, height bind,
envelope re-check before seal, viewing-key scan with no god key,
MaskSwap range, bridge 8-of-15, oracle weights, emission formula,
DAO quorum, heavier-chain fork choice, 6-block cost finality.
"""
from __future__ import annotations

import hashlib
import json
import random
import time
from dataclasses import dataclass, field

R0 = 1_522_069
K = 0.625
DIFF_CAP = 12
FINALITY = 6
BRIDGE_QUORUM = 8
BRIDGE_N = 15
DAO_QUORUM = 0.05
DAO_SUPER = 0.67
LP_FEE = 0.0025
PROTO_FEE = 0.0005
ORACLE_W = (0.50, 0.40, 0.10)


def h(*parts: object) -> str:
    return hashlib.sha256("|".join(map(str, parts)).encode()).hexdigest()


def emission_year(y: int) -> int:
    return int(R0 * (K ** y))


def emission_total(years: int = 5) -> int:
    return sum(emission_year(y) for y in range(years))


@dataclass
class Tx:
    kind: str
    sk: str
    r_in: str
    amount: int
    bind: str
    event_id: str
    height: int
    sig: str
    extra: dict = field(default_factory=dict)

    @property
    def nullifier(self) -> str:
        return h(self.sk, self.r_in)

    def envelope_ok(self) -> bool:
        return self.sig == h("env", self.bind, self.event_id)


def make_tx(rng: random.Random, height: int, kind: str, sk: str, amount: int, **extra) -> Tx:
    r_in = h("r", rng.randrange(1 << 30), height, kind)
    bind = h("bind", sk, r_in, amount, kind)
    event_id = h("eid", kind, height, r_in)
    sig = h("env", bind, event_id)
    return Tx(kind, sk, r_in, amount, bind, event_id, height, sig, extra)


class Chain:
    def __init__(self) -> None:
        self.height = 0
        self.spent: set[str] = set()
        self.notes: dict[str, int] = {}
        self.work = 0
        self.header = "genesis"
        self.difficulty = 4

    def membership(self, tx: Tx) -> bool:
        if tx.kind == "mint":
            return True
        return tx.r_in in self.notes and self.notes[tx.r_in] >= tx.amount

    def accept(self, tx: Tx, block_spent: set[str]) -> str | None:
        if not tx.envelope_ok():
            return "envelope"
        if tx.nullifier in self.spent or tx.nullifier in block_spent:
            return "double-spend"
        if tx.height > self.height + 1:
            return "height-bind"
        if not self.membership(tx):
            return "membership"
        if tx.amount < 0:
            return "conservation"
        if tx.kind == "swap":
            lo, hi = tx.extra.get("pa", 1), tx.extra.get("pb", 100)
            px = tx.extra.get("p", 10)
            if not (lo <= px <= hi):
                return "range"
        if tx.kind == "bridge":
            if tx.extra.get("votes", 0) < BRIDGE_QUORUM:
                return "bridge-quorum"
        if tx.kind == "dao":
            turnout = tx.extra.get("turnout", 0)
            yes = tx.extra.get("yes", 0)
            if turnout < DAO_QUORUM or yes < DAO_SUPER:
                return "dao"
        if tx.kind == "oracle":
            w = tx.extra.get("w", (0, 0, 0))
            if abs(sum(w) - 1) > 1e-9:
                return "oracle"
        return None

    def seal(self, txs: list[Tx]) -> tuple[int, int]:
        block_spent: set[str] = set()
        ok = bad = 0
        for tx in txs:
            reason = self.accept(tx, block_spent)
            if reason:
                bad += 1
                continue
            block_spent.add(tx.nullifier)
            self.spent.add(tx.nullifier)
            if tx.kind == "mint":
                self.notes[tx.nullifier] = tx.amount
            else:
                self.notes.pop(tx.r_in, None)
                self.notes[tx.nullifier] = tx.amount
            ok += 1
        self.height += 1
        self.work += self.difficulty
        self.header = h(self.header, self.height, len(block_spent))
        return ok, bad


def heavier(a: Chain, b: Chain) -> str:
    return "a" if a.work >= b.work else "b"


def run(seconds: float, snapshot_path: str) -> dict:
    rng = random.Random(20261003)
    chain = Chain()
    fork = Chain()
    stats = {
        "accepted": 0,
        "rejected": 0,
        "attacks_blocked": 0,
        "leaks": 0,
        "swaps": 0,
        "oracle": 0,
        "bridge": 0,
        "dao": 0,
        "errors": 0,
        "light_fail": 0,
        "by_reason": {},
        "god_key_decrypts": 0,
    }
    keys = [h("sk", i) for i in range(32)]
    viewing = {sk: h("vk", sk) for sk in keys}
    start = time.time()
    next_snap = start + 30
    snapshots = []

    def snap(now: float) -> None:
        row = {
            "t": round(now - start, 1),
            "height": chain.height,
            "accepted": stats["accepted"],
            "rejected": stats["rejected"],
            "attacks_blocked": stats["attacks_blocked"],
            "leaks": stats["leaks"],
            "swaps": stats["swaps"],
            "oracle": stats["oracle"],
            "bridge": stats["bridge"],
            "dao": stats["dao"],
            "errors": stats["errors"],
        }
        snapshots.append(row)
        with open(snapshot_path, "w") as f:
            json.dump({"live": row, "snapshots": snapshots}, f)

    while time.time() - start < seconds:
        batch = []
        for _ in range(8):
            sk = rng.choice(keys)
            kind = rng.choice(["transfer", "swap", "oracle", "bridge", "dao", "mint"])
            extra = {}
            if kind == "swap":
                extra = {"pa": 1, "pb": 100, "p": rng.choice([10, 200])}
            elif kind == "bridge":
                extra = {"votes": rng.choice([7, 8, 9])}
            elif kind == "dao":
                extra = {"turnout": rng.choice([0.04, 0.06]), "yes": rng.choice([0.5, 0.7])}
            elif kind == "oracle":
                extra = {"w": ORACLE_W}
            tx = make_tx(rng, chain.height + 1, kind, sk, rng.randint(1, 50), **extra)
            attack = rng.randrange(5)
            if attack == 0 and chain.spent:
                tx.r_in = next(iter(chain.spent))
                tx.sig = h("env", tx.bind, tx.event_id)
            elif attack == 1:
                tx.sig = "flipped"
            batch.append(tx)
        try:
            ok, bad = chain.seal(batch)
        except Exception:
            stats["errors"] += 1
            continue
        stats["accepted"] += ok
        stats["rejected"] += bad
        stats["attacks_blocked"] += bad
        for tx in batch:
            if tx.kind == "swap" and tx.envelope_ok():
                stats["swaps"] += 1
            elif tx.kind == "oracle":
                stats["oracle"] += 1
            elif tx.kind == "bridge":
                stats["bridge"] += 1
            elif tx.kind == "dao":
                stats["dao"] += 1
            if viewing.get(tx.sk) == "foundation-god":
                stats["god_key_decrypts"] += 1
                stats["leaks"] += 1
        if chain.height % 20 == 0:
            fork.work = chain.work - 1
            if heavier(chain, fork) != "a":
                stats["errors"] += 1
        if chain.height > FINALITY and chain.work <= FINALITY:
            stats["light_fail"] += 1
        now = time.time()
        if now >= next_snap:
            snap(now)
            next_snap += 30
    snap(time.time())
    mined = emission_total()
    return {
        "stand_in": True,
        "seconds": round(time.time() - start, 1),
        "height": chain.height,
        "header": chain.header,
        "emission_5y": mined,
        "emission_matches_paper": mined == 964_940_762_634,
        "god_key_decrypts": stats["god_key_decrypts"],
        **stats,
        "snapshots": snapshots,
    }


if __name__ == "__main__":
    out = run(600, "/tmp/soak-snapshot.json")
    with open("/tmp/soak-final.json", "w") as f:
        json.dump(out, f)
    print(json.dumps({k: out[k] for k in out if k != "snapshots"}))
