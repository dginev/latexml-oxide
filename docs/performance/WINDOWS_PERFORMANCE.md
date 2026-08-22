# Windows performance profile — the kpathsea backend is everything

> First Windows-specific perf pass (2026-08-21), on latest `main`
> (`f0936ed04d`, the `0.7.6-rc1` line). The prior campaigns
> ([`PERFORMANCE.md`](PERFORMANCE.md), [`ARXIV_PERFORMANCE.md`](ARXIV_PERFORMANCE.md))
> only ever measured the Linux CI/benchmark host. This one asks: *does the same
> code have a Windows-specific bottleneck?*
>
> **Answer: yes, exactly one, and it is large — the file-lookup backend.** The
> subprocess-`kpsewhich` backend re-loads the TeX `ls-R` filename database on
> every spawn (~270–290 ms fixed cost), and file-heavy documents pay it per
> lookup, so `digest` swells to **86 %** of wall and a mid-size TikZ paper takes
> **21.7 s**. Rebuilding the *same commit* with the in-process linked-libkpathsea
> backend (`--features kpathsea-build-from-source`, which is what the shipped
> Windows distribution already uses) drops that paper **16×** to **1.35 s** and
> restores a broad, Linux-shaped phase budget. Nothing else on Windows is a
> bottleneck: process spawn, exe load, XSLT, serialize are all cheap.

---

## 1. Test bench

| | |
|---|---|
| CPU | AMD Ryzen Threadripper 1950X — 16C/32T, Zen 1 (2017), modest single-thread IPC |
| RAM | 64 GiB (≈53 GiB free during runs) |
| OS | Windows 10 Home 10.0.19045 |
| Defender | Real-time protection **ON**, behaviour monitor **ON** |
| TeX | TeX Live 2026 at `C:\texlive` (forced ahead of MiKTeX on `PATH`) |
| Toolchain | nightly-msvc, `CARGO_PROFILE_RELEASE_LTO=false`, LLVM `lld-link` |
| Binary | `target\release\latexml_oxide.exe`, built two ways for the A/B (below) |

Two binaries, same commit, same dumps ([`resources/dumps`](../../resources/dumps)
regenerated against TL2026 via `tools/make_formats.ps1`):

* **subprocess** — plain `cargo build --release`. No linked libkpathsea, so
  `select_kpaths` ([`latexml_core/src/util/pathname.rs`](../../latexml_core/src/util/pathname.rs))
  resolves files by spawning `kpsewhich`. **This is what a default local build
  gets**, and what any host without a usable linked libkpathsea falls back to
  (notably MiKTeX — see [Windows port notes]).
* **in-process** — `cargo build --release --features kpathsea-build-from-source`.
  Static libkpathsea linked in; `ls-R` is loaded once into the process. **This is
  what the release `.zip` ships** (per the Windows release leg).

Method: `tools/win_perf_bench.ps1` (committed alongside). It times each
conversion's *external* wall with a `Stopwatch` and reads the *in-process* wall
+ 17-phase budget from `--telemetry-out` ([`TELEMETRY.md`](TELEMETRY.md)). Warm
runs, min + median of 6. The external−telemetry gap is the spawn/init/IO the
pipeline clock never sees.

---

## 2. The headline A/B

External wall, median of 6, identical inputs, identical commit — the **only**
difference is the kpathsea backend:

| document | file resolutions¹ | subprocess | in-process | speed-up |
|---|--:|--:|--:|--:|
| `hello`          |   5 |    857 ms |    505 ms | 1.7× |
| `book`           |   5 |  1 036 ms |    657 ms | 1.6× |
| `equality_big`   |   5 |  3 555 ms |  3 261 ms | 1.09× |
| `si`             |  22 |  3 366 ms |  2 666 ms | 1.3× |
| `various_colors` | 102 | **21 717 ms** | **1 354 ms** | **16.0×** |

¹ `(Processing definitions …)` + `(Loading …)` lines in the conversion log — the
count of external files the engine had to resolve.

**The speed-up tracks the file-resolution count, not the document size.**
`equality_big` is a 170 KB, thousands-of-formulae math torture test but loads
only 5 files → the backend barely matters (1.09×). `various_colors` is a small
TikZ document that pulls in 102 files (pgf/TikZ libraries, colour files, fonts)
→ 16×. The subprocess penalty is `≈ (files needing kpsewhich) × ~272 ms`: for
`various_colors`, `21 717 − 1 354 = 20 363 ms ≈ 75 × 272 ms`.

The external−telemetry delta is small in **both** backends (≈ 0–120 ms), so this
is **not** `latexml_oxide` process spawn/teardown — the cost is *inside* the
pipeline, spent in `digest` waiting on child `kpsewhich` processes.

---

## 3. Why one `kpsewhich` costs ~270 ms (spawn microbench)

Min ms of 6, standalone processes:

| command | ms | what it isolates |
|---|--:|---|
| `cmd /c rem` | **8** | Windows `CreateProcess` floor |
| `latexml_oxide --version` | **10** | + load the 63–66 MB exe & static init |
| `kpsewhich --version` | **10** | spawn kpsewhich, **no** `ls-R` search |
| `kpsewhich cmr10.tfm` | **273** | + one `ls-R` filename-DB lookup |
| `kpsewhich` ×5 files, one process | **284** | five lookups in one process |

Two facts fall straight out:

1. **Process spawn is not the problem.** Windows `CreateProcess` is ~8 ms (vs
   sub-ms `fork` on Linux — real, but tiny here), and loading the big static exe
   adds only ~2 ms. `kpsewhich --version` is also ~10 ms.
2. **The ~270 ms is a per-*process* fixed cost, not per-file.** Five lookups in
   one process (284 ms) cost essentially the same as one (273 ms). That cost is
   kpathsea initialising and searching the `ls-R` database. The subprocess
   backend spawns a **fresh** kpsewhich per lookup, so it re-pays the ~270 ms
   `ls-R` init every single time. The in-process backend pays it **once** for
   the whole conversion.

(Defender real-time scanning of `kpsewhich.exe` + the kpathsea DLL on each spawn
plausibly contributes to the gap between the 8 ms floor and the 270 ms lookup,
but the dominant term is the per-process `ls-R` work — `--version`, which skips
the search, is only 10 ms.)

---

## 4. Phase budget: subprocess distorts it, in-process restores it

Aggregate `phase_us` share across the five documents:

| phase | subprocess | in-process | Linux 60k-arXiv² |
|---|--:|--:|--:|
| **digest** | **86.3 %** | **32.5 %** | 19.7 % |
| build | 7.5 % | 42.2 % | 18.1 % |
| xslt | 2.5 % | 10.2 % | 13.2 % |
| math_parse | 2.3 % | 9.5 % | 19.2 % |
| everything else | ≤0.3 % ea. | ≤1.6 % ea. | ≤8.9 % ea. |

² [`ARXIV_PERFORMANCE.md`](ARXIV_PERFORMANCE.md), containerised in-process worker
over 60 469 arXiv docs. Different corpus (real papers vs 5 repo fixtures), so
compare *shape*, not absolute cells.

Under the subprocess backend, `digest` eats everything because the per-file
`kpsewhich` waits are attributed there. Switch to in-process and the budget
becomes **broad** — `build` + `digest` + `xslt` + `math_parse` spread out — which
is the Linux shape. The residual `build`/`digest` weight (heavier than Linux) is
partly the fixture mix and partly this being a 2017 Zen 1 part running the
single-threaded digest/build engine; the 16C/32T box is idle during a single
conversion.

---

## 5. Findings & recommendations

**F1 — The kpathsea backend is the one Windows perf lever that matters.**
In-process linked libkpathsea is 16× on file-heavy docs and ~1.5× on trivial
ones, for zero output change. The shipped Windows `.zip` already uses it; this
pass **quantifies and validates** that architecture decision.
* *Local/dev builds:* a plain `cargo build --release` gets the **subprocess**
  backend and is misleadingly slow on TikZ/graphics-heavy inputs. Benchmark with
  `--features kpathsea-build-from-source`, or the numbers lie by up to 16×.
* *MiKTeX hosts:* in-process can't read MiKTeX's fndb, so `select_kpaths` falls
  back to subprocess **even in the shipped build** ([Windows port notes]) — and
  MiKTeX's kpsewhich is ~340 ms/spawn, worse than TL's ~270 ms. MiKTeX users are
  the population that still pays F1 in full.

**F2 — For the subprocess path, batch or cache `kpsewhich`.** The microbench
proves 5 lookups in one process ≈ 1 (284 vs 273 ms). Two concrete levers for
the fallback backend: (a) memoise per-file lookups within a conversion so a file
is never resolved twice; (b) resolve pending includes/fonts in **batched**
kpsewhich calls instead of one-per-file. Either shrinks the MiKTeX/fallback tax
by the batching factor. (Neither is needed on the in-process path.)

**F3 — Process/exe cost is a non-issue.** ~8 ms spawn, ~2 ms to load the 63–66 MB
static exe. Binary size and `+crt-static` are not perf problems; no action.

**F4 — Windows Defender is a background tax, not a headline.** Real-time
protection is on and scans each spawned exe and each freshly-written temp/output
file. With the in-process backend (≈0 child processes) its effect on conversion
is negligible. It *does* surface as flakiness elsewhere — see F5. Excluding
`C:\texlive\...\bin` and the scratch/temp dir would trim per-spawn scan time on
the subprocess path (needs admin; not verifiable here — this box can't read
exclusions without elevation).

**F5 — Swallowed transient file-lock error (`read_pdf_page_box`).** Unrelated to
throughput but found in passing: on Windows a just-written `.pdf` can be briefly
locked (sharing + Defender scan), and `read_pdf_page_box`
([`latexml_core/src/util/image.rs`](../../latexml_core/src/util/image.rs)) does
`std::fs::read(path).ok()?` — a transient lock becomes a silent `None` (a figure
reaches the engine at 0×0). It also makes the `read_pdf_page_box_prefers_cropbox…`
unit test flaky on Windows. Consider a short retry on transient read errors.

**F6 — Telemetry `max_rss_kb` is 0 on the Windows single-shot CLI.** Peak-RSS is
only wired through `cortex_worker`'s child accounting; the direct-CLI
`--telemetry-out` path reports 0 on Windows. Minor, but it blocks RSS profiling
of one-off conversions here.

---

## 6. Reproduce

```powershell
# TeX Live 2026 FIRST on PATH (never MiKTeX-first) — the fixtures assume TL.
$env:PATH = "C:\texlive\2026\bin\windows;" + $env:PATH

# regenerate the ambient-year dumps for the current engine
$env:PROFILE='release'; $env:CARGO_PROFILE_RELEASE_LTO='false'; tools\make_formats.ps1

# A: subprocess backend (default build)
cargo build --release --bin latexml_oxide
tools\win_perf_bench.ps1

# B: in-process backend (shipped Windows config)
cargo build --release --features kpathsea-build-from-source --bin latexml_oxide
tools\win_perf_bench.ps1
```

Confirm which backend a binary uses via the `Info:kpathsea:backend …` line the
harness echoes (`subprocess kpsewhich` vs `in-process (linked)`).

---

_See also:_ [`ARXIV_PERFORMANCE.md`](ARXIV_PERFORMANCE.md) (Linux baseline),
[`TELEMETRY.md`](TELEMETRY.md) (the phase instrument),
`docs/release/WINDOWS_COMPATIBILITY_PLAN.md` (the port + backend-selection
design). [Windows port notes]: the MiKTeX fallback and the ~340 ms MiKTeX
kpsewhich are from the port's backend-selection bring-up.
