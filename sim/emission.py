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


def issued(year: int) -> int:
    return int(R0 * (K ** year) * BLOCKS)


def main() -> None:
    rows = [issued(y) for y in range(5)]
    total = sum(rows)
    print(f"exact total {total}")
    print(f"paper total {PAPER_TOTAL} match {total == PAPER_TOTAL}")
    print(f"printed column sum {sum(PAPER_ISSUED)}")
    for y, row in enumerate(rows):
        print(f"year {y + 1} exact {row} printed {PAPER_ISSUED[y]}")


if __name__ == "__main__":
    main()
