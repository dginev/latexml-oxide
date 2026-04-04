# arxiv-examples Conversion Catalog

Generated 2026-04-05. 47 papers tested with latexml-oxide (release build).

## Summary

- **38/47 OK** (81%) — produce meaningful HTML output
- **3 EMPTY** — produce minimal/empty output (cascading errors)
- **6 FAIL** — timeout or crash (no output)
- **5 zero-error** papers (perfect conversion)

## Results

| Paper ID | Status | Size | Errors | Time (ms) | Notes |
|----------|--------|------|--------|-----------|-------|
| 0710.2281 | OK | 361KB | 18 | 4505 | |
| 1312.5845 | OK | 42KB | 1 | 899 | |
| 1502.04955 | OK | 608KB | 11 | 7456 | |
| 1706.03762 | OK | 4KB | 3 | 77 | Attention paper (multi-file) |
| 1907.08050 | FAIL | 0 | 9 | 30s | Timeout |
| 1910.06709 | OK | 69KB | 0 | 438 | Zero errors |
| 2005.13625 | OK | 82KB | 122 | 1011 | minted/code errors |
| 2008.08932 | OK | 2KB | 3 | 69 | |
| 2101.00726 | OK | 770KB | 2 | 20385 | Large paper |
| 2103.01205 | OK | 34KB | 7 | 187 | |
| 2209.14198 | EMPTY | 39B | 9 | 6236 | Cascading errors |
| 2306.00809 | OK | 160KB | 4 | 7747 | |
| 2306.06628 | OK | 247KB | 0 | 3239 | Zero errors |
| 2308.06254 | OK | 2KB | 2 | 1707 | |
| 2310.18318 | OK | 343KB | 2 | 1014 | |
| 2401.08110 | FAIL | 0 | 4 | 30s | Timeout |
| 2401.18036 | OK | 200KB | 0 | 1807 | Zero errors |
| 2401.18052 | OK | 169KB | 0 | 3466 | Zero errors |
| 2402.03300 | OK | 4KB | 26 | 67 | Custom class |
| 2402.10301 | FAIL | 0 | 444 | 30s | pgf arrow errors |
| 2403.07652 | OK | 122KB | 1 | 517 | |
| 2403.15796 | OK | 4KB | 1 | 495 | |
| 2405.17032 | OK | 2KB | 2 | 72 | |
| 2405.19425 | FAIL | 0 | 2 | 30s | Timeout |
| 2406.06608 | OK | 667KB | 28 | 9867 | |
| 2408.11158 | OK | 69KB | 3 | 1464 | |
| 2408.13687 | OK | 118KB | 1 | 1250 | |
| 2410.10068 | OK | 8KB | 8 | 118 | |
| 2410.12896 | OK | 464KB | 35 | 3570 | |
| 2502.04134 | OK | 131KB | 3 | 2256 | |
| 2503.08256 | OK | 2KB | 3 | 78 | |
| 2506.03074 | OK | 18KB | 20 | 259 | |
| 2507.23241 | OK | 1.1MB | 22 | 27453 | Largest output |
| 2508.15260 | OK | 486KB | 2113 | 6991 | tcolorbox cascading |
| 2508.18544 | EMPTY | 39B | 10001 | 114 | MAX_ERRORS hit |
| 2509.18103 | OK | 238KB | 1 | 808 | |
| 2511.03798 | EMPTY | 39B | 70 | 1536 | |
| 2511.11713 | OK | 98KB | 0 | 1449 | Zero errors (IEEE) |
| 2511.14458 | OK | 218KB | 1 | 2700 | |
| 2511.15304 | OK | 116KB | 2 | 1944 | |
| 2512.09456 | OK | 112KB | 9 | 800 | |
| 2512.16911 | OK | 709KB | 13 | 18925 | |
| 2602.18719 | OK | 42KB | 182 | 775 | |
| 2602.23324 | FAIL | 0 | 4 | 30s | Timeout |
| 2603.14602 | OK | 5KB | 19 | 231 | |
| 2603.15617 | OK | 37KB | 38 | 3110 | |
| 2603.19312 | FAIL | 0 | 6 | 30s | Timeout |

## Failure Analysis

**Timeouts (4):** 1907.08050, 2401.08110, 2405.19425, 2602.23324, 2603.19312 — likely tikz-heavy or complex package loading

**pgf arrows (1):** 2402.10301 — `Unknown arrow tip kind 'Computer Modern Rightarrow'`

**Cascading errors (3):** 2209.14198, 2508.18544, 2511.03798 — tcolorbox/minted cascading failures
