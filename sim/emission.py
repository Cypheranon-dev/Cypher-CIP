"""§12.3 derivation. R_y = 1,522,069 * (5/8)^y, times 262,800 blocks."""
from fractions import Fraction

R0 = 1_522_069
K = Fraction(5, 8)
BLOCKS = 262_800
PAPER_TOTAL = 964_940_762_634
PAPER_ISSUED = (
    400_000_000_000,
    249_999_833_250,
    156_249_895_781,
    97_656_184_863,
    61_035_115_540,
)


def row(year: int) -> Fraction:
    return R0 * (K ** year) * BLOCKS


def main() -> None:
    rows = [row(y) for y in range(5)]
    floored = [int(r) for r in rows]
    print(f"floor each year {sum(floored)}")
    print(f"floor of sum {int(sum(rows))}")
    print(f"paper total {PAPER_TOTAL}")
    print(f"printed column sum {sum(PAPER_ISSUED)}")
    for y, exact in enumerate(rows):
        print(f"year {y + 1} floor {int(exact)} printed {PAPER_ISSUED[y]}")


if __name__ == "__main__":
    main()
