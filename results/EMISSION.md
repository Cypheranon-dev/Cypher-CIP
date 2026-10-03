# Emission derivation

Source of truth: §12.2, \(R_y = 1{,}522{,}069 \times (5/8)^y\), times 262,800 blocks. Run `python3 sim/emission.py`.

| Year | Exact issued | Printed in §12.3 |
| --- | --- | --- |
| 1 | 399,999,733,200 | 400,000,000,000 |
| 2 | 249,999,833,250 | 249,999,833,250 |
| 3 | 156,249,895,781 | 156,249,895,781 |
| 4 | 97,656,184,863 | 97,656,184,863 |
| 5 | 61,035,115,539 | 61,035,115,540 |

Exact total: 964,940,762,634. That matches the total stated in §12.3. The printed column sums to 964,941,029,434. The gap is the year-1 display, which is rounded up by 266,800, plus one unit on year 5. The formula, not the printed column, is the figure to cite.
