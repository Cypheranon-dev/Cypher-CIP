#!/usr/bin/env python3
"""Hash stand-in for the protocol rules. Not a STARK, not Dilithium5, not Equihash.

Envelope is SHA-256 over the bind. The bind covers the fields the rule
checks, so a flipped vote or price fails the envelope. A miner re-checks
that envelope inside seal. Nullifier is H(sk, r_in).
"""
from __future__ import annotations

import hashlib
from dataclasses import dataclass, field

BRIDGE_QUORUM = 8
BRIDGE_N = 15
DAO_QUORUM = 0.05
DAO_SUPER = 0.67
SPEND_KINDS = frozenset({"transfer", "swap", "oracle", "dao"})
MINT_KINDS = frozenset({"mint", "bridge"})


def h(*parts: object) -> str:
    return hashlib.sha256("|".join(map(str, parts)).encode()).hexdigest()


def canonical_extra(extra: dict) -> str:
    return ",".join(f"{key}={extra[key]}" for key in sorted(extra))


def viewing_key(sk: str) -> str:
    return h("vk", sk)


def opens(vk: str, sk: str) -> bool:
    return vk == viewing_key(sk)


@dataclass
class Tx:
    kind: str
    sk: str
    r_in: str
    amount: int
    height: int
    event_id: str
    extra: dict = field(default_factory=dict)
    bind: str = ""
    sig: str = ""

    @property
    def nullifier(self) -> str:
        return h(self.sk, self.r_in)


def bind_of(tx: Tx) -> str:
    return h("bind", tx.sk, tx.r_in, tx.amount, tx.kind, tx.height, canonical_extra(tx.extra))


def sign(tx: Tx) -> Tx:
    tx.bind = bind_of(tx)
    tx.sig = h("env", tx.bind, tx.event_id)
    return tx


def make_tx(height: int, kind: str, sk: str, amount: int, r_in: str, event_id: str, **extra) -> Tx:
    return sign(Tx(kind, sk, r_in, amount, height, event_id, extra))


class Chain:
    def __init__(self) -> None:
        self.height = 0
        self.spent: set[str] = set()
        self.notes: dict[str, int] = {}
        self.work = 0
        self.header = "genesis"

    def accept(self, tx: Tx, block_spent: set[str]) -> str | None:
        if tx.sig != h("env", tx.bind, tx.event_id) or tx.bind != bind_of(tx):
            return "envelope"
        if tx.nullifier in self.spent or tx.nullifier in block_spent:
            return "double-spend"
        if tx.height != self.height + 1:
            return "height-bind"
        if tx.amount <= 0:
            return "conservation"
        if tx.kind in SPEND_KINDS:
            if tx.r_in not in self.notes:
                return "membership"
            if self.notes[tx.r_in] != tx.amount:
                return "conservation"
        elif tx.kind not in MINT_KINDS:
            return "kind"
        if tx.kind == "swap":
            lo, hi, px = tx.extra.get("pa"), tx.extra.get("pb"), tx.extra.get("p")
            if lo is None or hi is None or px is None or not (lo <= px <= hi):
                return "range"
        if tx.kind == "bridge":
            votes = tx.extra.get("votes", 0)
            if not (BRIDGE_QUORUM <= votes <= BRIDGE_N):
                return "bridge-quorum"
        if tx.kind == "dao":
            if tx.extra.get("turnout", 0) < DAO_QUORUM or tx.extra.get("yes", 0) < DAO_SUPER:
                return "dao"
        if tx.kind == "oracle":
            weights = tx.extra.get("w", ())
            if len(weights) != 3 or abs(sum(weights) - 1) > 1e-9:
                return "oracle"
        return None

    def seal(self, txs: list[Tx]) -> tuple[int, int, dict[str, int]]:
        block_spent: set[str] = set()
        reasons: dict[str, int] = {}
        ok = 0
        for tx in txs:
            reason = self.accept(tx, block_spent)
            if reason:
                reasons[reason] = reasons.get(reason, 0) + 1
                continue
            block_spent.add(tx.nullifier)
            self.spent.add(tx.nullifier)
            if tx.kind in SPEND_KINDS:
                del self.notes[tx.r_in]
            self.notes[tx.nullifier] = tx.amount
            ok += 1
        self.height += 1
        self.work += 1
        self.header = h(self.header, self.height, len(block_spent))
        return ok, sum(reasons.values()), reasons


def heavier(a: Chain, b: Chain) -> str:
    if a.work > b.work:
        return "a"
    if b.work > a.work:
        return "b"
    return "tie"
