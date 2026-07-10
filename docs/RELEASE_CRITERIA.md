# Release Criteria — what must be true before a public 1.0

The externally-readable "what must be true before we ship" contract, kept
*separate* from [`SYNC_STATUS.md`](SYNC_STATUS.md) (the engine-sync /
parity log). Two non-competing missions:

- **Parity** (SYNC_STATUS): match Perl on the arXiv corpus, no
  error-downgrading. Compass: ~99.4% on the 100k warning subset.
- **Release-readiness** (here): speed, RAM, size, portability, licensing,
  safety, downstream tooling.

Origin: the 2026-05-24 codex "public-quality gaps" pass + a code-checked
review. Corrected positions are stated, not the original where it was wrong
(see §10).

## 1. Gates

Numbers are verified current state (2026-05-24) unless marked TODO. The
`cargo test` / `cargo clippy` rows were re-verified 2026-07-09
(`public-release-prep-week`); the OS/arch, toolchain, license, and safety rows
were also refreshed then. The corpus / tail-latency / size rows still carry
their 2026-05-24 values.

| Gate | Current | Target |
|---|---|---|
| `cargo test --tests` | 1533/0/0 | green |
| `cargo clippy --all-targets` | 0 | 0 |
| Corpus (100k warning subset) | ~99.39% / ~99.44% rerun-adj | no regression; gate cohorts separately (`no-problem`, warning subset, random full sample, hard package/class) |
| Tail latency / RSS | mean bands only ([`PERFORMANCE.md`](PERFORMANCE.md)); **P50/P90/P99 + RSS dashboard + growth gate built** (`tools/telemetry_dashboard.py`) | §5 — capture a fleet baseline + wire `--gate` into release (absolute red lines gate today) |
| Binary size (`maxperf`) | **45 MB / 14 MB tarball** | budget + growth alarm — **§2 DONE** (release.yml 64 MB gate) |
| OS/arch | `x86_64-linux-gnu` + **`aarch64-unknown-linux-gnu`** + `aarch64-apple-darwin` + `x86_64-apple-darwin` + **GHCR container (amd64/arm64)** | staged ladder — §3 (aarch64-linux + container DONE 2026-07-09; next rung: Windows/musl) |
| Toolchain | **nightly**, **deliberately floating** (`rust-toolchain.toml`, 2026-07-03) | keep floating; pin a dated nightly only if release-day reproducibility is needed (#143) |
| License inventory | **inventoried + gated** ([`LICENSE_INVENTORY.md`](LICENSE_INVENTORY.md)); NOTICE + README + release-workflow wiring landed | **§4/§7 DONE** (F4 landed; only cortex-only F1 remains, non-blocking) |
| Safety | local-CLI model ([`SAFETY.md`](SAFETY.md)); URI-passthrough posture documented | remaining §6 items (CSP/sandboxing/`--hardened`) — **1.0-scoped, not a 0.7.4 blocker** |

## 2. Binary size (issue #101)

Get attribution right first — it's **code, not data**:
- `.text` ≈ **36.7 MB of 45 MB** (`size -A`), dominated by the compile-time
  binding pool (`latexml_package::pool::*::load_definitions`; ~56% of
  `.text` in #101's 2023 `cargo bloat`, grown since).
- Dumps are **not** a driver: already gzip-embedded (DEP-12,
  `embedded_dumps.rs`), ~7.6 MB text → **~870 KB `.rodata`**, lazy-inflated
  at bootstrap. "Gzip the dumps" is *done* — don't re-propose it.

Levers: (1) re-run `cargo bloat` to refresh attribution before acting;
(2) attack binding-pool codegen density (relates to #93); (3) `maxperf`
already does fat-LTO + `codegen-units=1` + strip + `panic=abort` +
`--no-default-features`. Gate: CI prints size + top `.text` contributors,
fails on budget breach (§7).

## 3. Portability staging (issues #217, #143)

Current: **four** self-contained published artifacts — `x86_64-linux-gnu`
and `aarch64-unknown-linux-gnu` (both Ubuntu 22.04 / glibc 2.35),
`aarch64-apple-darwin` (macOS Apple Silicon), and `x86_64-apple-darwin`
(macOS Intel) — each embedding our XSLT/CSS/JS/schema/dumps, host TeX Live +
system libs ([`RELEASING.md`](RELEASING.md) → "Release asset strategy").
A native binary is never cross-OS (ELF vs Mach-O), so it is one artifact
per `(OS, arch)` triple, built on its own native runner — not
cross-compiled. Ladder — each stage needs a smoke corpus + size gate +
dependency check:

1. Debian/Ubuntu x86_64 (current).
2. aarch64 Linux — **DONE 2026-07-09**: `release.yml`'s `build-linux-arm64`
   leg (`ubuntu-22.04-arm`) publishes a tarball + `arm64` `.deb`, a full
   build+gate peer of the x86_64 leg (static libxml2/libxslt/kpathsea, `ldd`
   self-contained check, conversion + embedded-resource smokes, 64 MB size
   budget).
3. Container image (reproducible TeX Live + graphics) — **DONE 2026-07-09**:
   `.github/workflows/docker.yml` publishes two images from ONE unified root
   `Dockerfile` (`--target cli`/`--target worker`, DRY shared deps): the
   general-purpose CLI `ghcr.io/dginev/latexml-oxide` (multi-arch amd64+arm64,
   native runners) and the CorTeX fleet worker
   `ghcr.io/dginev/latexml-oxide/cortex-worker` (amd64), on `release: published`.
   Each BUILDS its own binary + regenerates dumps against its own TeX Live; the
   CLI embeds them behind a runtime-stage self-test.
4. macOS (#217) — **DONE 2026-06-08**
   ([`archive/PORTABILITY_MACOS_PROBE_2026-06-07.md`](archive/PORTABILITY_MACOS_PROBE_2026-06-07.md)):
   the full `cargo test --tests --workspace` suite is **green on `macos-15`
   arm64** (brew-texlive gating leg: 1390 passed / 0 failed / 0 crashes,
   43 binaries). MacTeX ships NO libkpathsea → covered by **kpathsea 0.3.0
   (crates.io)** subprocess-`kpsewhich` fallback. The macOS-only
   worker-thread Node corruption was a **use-after-free of a
   libxml2-merged text node** — detected via a read of the freed node
   (benign on glibc, exposed by macOS libmalloc); fixed in
   `open_text_internal` with a pointer-identity merge check (WISDOM #58)
   and audited for sibling sites. crates.io release + dep swap, README
   install matrix, and the gating CI job all **done**. **Release asset
   automated 2026-06-08**: `release.yml`'s `build-macos` job builds the
   `aarch64-apple-darwin` tarball natively on `macos-15` (subprocess-
   `kpsewhich`, host brew libxml2/libxslt, same embedded TL-window dumps)
   and the Linux `release` job publishes it alongside the Linux assets.
   **Intel macOS (`x86_64-apple-darwin`) published** via the
   `build-macos-intel` leg (`macos-15-intel`, `MACOSX_DEPLOYMENT_TARGET=10.13`
   for older Intel Macs) — arm64 binaries don't run on Intel, so it is a
   separate native leg (not a cross-compile / `lipo` universal, which is
   revisited only when GitHub sunsets the Intel runner ~Fall 2027).
5. Windows / musl — deferred. Known blockers: `libmarpa-sys`
   `./configure && make` (needs a cc-crate port; tarball is vendored),
   `lsp_server` unix sockets, `graphics*.rs` cfg(unix) paths,
   vcpkg-sourced libxml2/libxslt. The subprocess-`kpsewhich` fallback
   already removes the kpathsea blocker (MiKTeX's kpsewhich.exe
   delegates to MiKTeX's own resolver — better than linking could do).

**Self-contained libxml2/libxslt — DONE (shipped 0.7.1).** 0.7.0 dynamically
linked the build host's libxml2/libxslt SONAME (`libxml2.so.2` on the
ubuntu-22.04 runner), which loads on libxml2-2.x systems but **NOT** on
libxml2 ≥ 2.14 (SONAME bumped `.so.2` → `.so.16`). 0.7.1 **static-links
libxml2 + libxslt + libexslt** (PIC, source-built) on top of libkpathsea —
the kpathsea playbook: `tools/build_static_libxml.sh` +
`tools/build_static_kpathsea.sh` build the PIC `.a` archives (libxml2-dev's
`libxml2.a` plus source-built `libxslt`/`libexslt`, which ship no packaged
`.a`), and the `LIBXML2_STATIC` / `LIBXSLT_STATIC` build.rs branches in the
`libxml`/`libxslt` forks emit the `static=` link. `release.yml` runs both
scripts on the Linux and macOS legs, and a CI step asserts the binary carries
**no** dynamic libxml2/libxslt/kpathsea; transitive `-lz`/`-lgcrypt` stay
dynamic (stable SONAMEs). Net: only the glibc family + zlib + libgcrypt remain
dynamic → "any glibc-2.35+ Linux, any libxml/libxslt version," and the `.deb`
declares no libxml2 SONAME dependency (RELEASING.md). **Portability gate** (a
static `latexml_oxide --version` running on this dev box, which is on
libxml2.so.16 / 2.15.2 where a 0.7.0 `.so.2` binary fails to load): met by the
0.7.1 release build. The **default dev build stays dynamic** (`cargo build`
with no static env) — static is the release-only path.

**Nightly (#143):** required (`thread_local`). For a long-lived tool, a
reproducibility risk — pin a known-good nightly, track stabilization.
"Carries our resources" ≠ "portable across platforms."

**Editor-distributed binary is a stricter bar than the CLI.** The ladder
above allows "host system libs" — fine for the CLI/sandbox, where the user
(or the .deb deps) provides libxml2/libxslt. A binary shipped *inside a
VSCode extension* cannot assume that, especially on macOS/Windows: it must
be **self-contained** re: libxml2/libxslt (static/vendored). That stricter
bar — and the editor distribution model it gates — is §11.

## 4. License audit (blocker)

Crates are `CC0`, but the binary ships more. Full inventory:
[`LICENSE_INVENTORY.md`](LICENSE_INVENTORY.md) (living). **Analysis complete
2026-07-09**; posture is clean, three outward-facing items remain:

- **Rust deps — DONE (gated).** `deny.toml` allow-list + cargo-deny CI;
  `cargo deny --all-features check licenses` → *licenses ok*. Distributed
  feature set clean too (the `pericortex` no-license warning is cortex-only,
  absent from the shipped binary — inventory F1).
- **Embedded assets — DONE.** CSS/XSLT/RelaxNG/DTD/Profiles + one JS are Perl
  LaTeXML (NIST public domain ≈ CC0); the other JS is ours (CC0). No notice
  burden.
- **Graphics tools — DONE (confirmed subprocess-only).** `gs`/`mutool`/
  `pdftocairo`/`convert` are `Command::new` only, never linked → their (A)GPL
  does not propagate.
- **Dumps (TeX-Live-derived) — POSITION APPROVED + LANDED 2026-07-09.**
  Gitignored (repo ships none); embedded in the release binary at build time,
  derived from LaTeX kernel (LPPL 1.3c) + plain TeX (Knuth). Owner-approved:
  CC0 scoped to our source; `THIRD-PARTY-NOTICES` (assembled by
  `tools/gen_notices.sh` = hand-authored §1-4 + cargo-about §5) attributes the
  kernel/plain-TeX + linked libs + Rust crates; README License section scopes
  the claim (inventory §C).
- **Remaining:** wire `tools/gen_notices.sh` into the release workflow so the
  artifact ships the assembled notices, + the CI asset-inventory gate (F4 → §7);
  `pericortex` upstream `license` field (F1, cortex-only, non-blocking).

## 5. Tail latency & RSS

The public-quality risk is outliers, not the mean: 60s+ timeout/fatal tail;
math-bocage ambiguity explosions
([`archive/MATH_AMBIGUITY_AUDIT_2026-05-21.md`](archive/MATH_AMBIGUITY_AUDIT_2026-05-21.md));
4 GiB alloc failures; high-RSS package loads; ar5iv limit creep hiding
over-evaluation. Build a rolling dashboard from `telemetry.jsonl.gz`
(schema exists, [`TELEMETRY.md`](TELEMETRY.md)): P50/P90/P99 wall+RSS, top
fatal/timeout/ambiguity witnesses. Gate "no unbounded growth" *separately*
from "mean beats Perl."

**Status (2026-07-09) — dashboard + gate BUILT.**
[`tools/telemetry_dashboard.py`](../tools/telemetry_dashboard.py) reads a
`telemetry.jsonl.gz` (or a dir of output ZIPs) and reports P50/P90/P99/max
wall + peak-RSS, the phase that drives the P99, and the top slowest / highest-RSS
/ fatal-timeout-error / math-ambiguity witnesses. `--gate` enforces the
"no unbounded growth" contract — **absolute red lines** (peak RSS → the 4 GiB
alloc wall via `--rss-redline-mb`; P99 wall → the timeout via
`--wall-redline-frac`; any hard-timeout job) plus an **optional regression check**
vs a committed baseline (`--baseline`, `--update-baseline`, `--tolerance`) — and
it is deliberately independent of any mean/Perl comparison. Complements the
phase-attribution rollup [`tools/perf_phase_summary.py`](../tools/perf_phase_summary.py).
**Remaining:** capture a representative fleet-run baseline
(`--update-baseline`), commit it, and wire `--gate --baseline` into the release
job. The absolute red lines already gate today without a baseline.

**Live aggregation view — hosted in cortex (2026-07-10).** The standalone
`telemetry_dashboard.py` is the CI/release *gate*; the *live monitoring* twin
now lives in the cortex frontend (its natural home), reading each run's
`telemetry.json` straight out of the persisted result archives: `GET
/telemetry/<corpus>/<service>` (+ `/api/telemetry/...`) — outcome mix, wall/RSS
P50/P90/P99/max, per-phase P99, and slowest/highest-RSS witnesses (cortex PR
#399). Fed by the containerized worker built from the unified `Dockerfile`
(`--target worker`); validated 2026-07-10 over the sandbox-arxiv-2605/2606
reruns (baked dumps drop the bootstrap phase P99 to 0; math_parse + digest are
the phase tails).

**Empirical "no unbounded growth" evidence (2026-07-10, 60,469 docs).** The §5
worry — the 60s+ timeout tail and the 4 GiB alloc wall — is **measured, not
assumed**, and controlled: **zero timeouts across 60k docs** (wall median 3.07s,
P99 34.9s, max 149s; only 120 papers / 0.20% over 60s, 4 over 120s, none at
180s), and **peak RSS median 0.94 / P99 2.05 / max 4.52 GiB — exactly one doc
over 4 GiB**, 53 (0.09%) over 3 GiB. The residual worst-case tail is a *reliability*
cluster (digest-runaway fatals, `STABILITY_WITNESSES.md` Cluster H), not
unbounded growth. This is direct go-signal evidence for the release gate; capture
it as the committed `--gate --baseline` snapshot (the "Remaining" item above).
Full phase budget + tail analysis: `ARXIV_PERFORMANCE.md` "Corpus-wide phase
budget 2026-07-10".

## 6. Safety: distribution profile

[`SAFETY.md`](SAFETY.md)'s local-CLI batch-compiler model is honest but not
the public-deployment story (arXiv-scale = hostile input, *published*
HTML/SVG). Don't change converter behavior during parity work — *document*
what's safe where:
- **URIs:** `\href{javascript:…}` / data URLs pass through today. Keep
  sanitization out of the parity converter; offer an optional output pass /
  downstream responsibility.
- CSP for published HTML/SVG; archive/path-traversal + temp-dir
  invariants; subprocess sandboxing beyond timeout+pgroup-kill; whether a
  `--hardened`/`--public-html` profile should diverge from parity (cf. §8).

## 7. CI must prove artifact properties

`CI.yml` is RAM-bounded for the test suite (correct). Release-artifact
properties are proven in `release.yml` (release-only, so per-PR CI cost is
unaffected). Status:

**Landed (`release.yml`, Linux job):**
- **Size budget** (§2) — `binary size budget` step, 64 MB cap, runaway-growth
  alarm.
- **Embedded-resource smoke** — `embedded-resource smoke` step re-runs a
  conversion with `resources/` renamed away (dumps/XSLT/RelaxNG/CSS must come
  from the embedded tables); complements `tests/001_single_binary_smoke.rs`
  (which only isolates cwd) and the functional conversion smoke inside `verify
  self-contained binary`.
- **License / notices** (§4 F4) — `assemble THIRD-PARTY-NOTICES` runs
  `tools/gen_notices.sh`; the notices are bundled in the tarball + `.deb`
  (`/usr/share/doc`) and published as a release asset.
- **No dynamic C-lib** — `verify self-contained binary` (`ldd` gate, since
  0.7.1).

**Already in `CI.yml` `lint` job (per-PR, cheap):** `cargo clippy
--all-targets`; `cargo-deny` (license allow-list + RUSTSEC advisories).

**Deferred / out of scope:**
- `strace` no-own-disk-read — the rename-`resources/` smoke covers the
  functional equivalent; the structural `strace` proof is lower-value, deferred.
- Graphics-tool matrix (with/without `gs`/`mutool`/`pdftocairo`, asserting
  graceful degradation via the missing-tool hint) — future.
- **Corpus smoke + telemetry — OUT OF SCOPE for CI** (too expensive, decided
  2026-07-09). Run out-of-band on the sandbox fleet, not the release workflow.

## 8. Surpass-Perl policy

Many open clusters are *shared* failures, not Rust regressions. Rule:
- Both fail on malformed/UB source → **SHARED-FAILURE**, no fix.
- Both fail only because a binding could read an arg under a more correct
  parameter type without harming valid content → documented **surpass-Perl**
  fix allowed.
- Visible output-shape change beyond error recovery → needs an
  [`OXIDIZED_DESIGN.md`](OXIDIZED_DESIGN.md) entry + witness comparison.
- Report the opportunity upstream where practical.

"Do not downgrade errors" stays non-negotiable. Existing cases:
`memory/project_rust_supersedes_perl.md` + SYNC_STATUS "Permanent ignores."

## 9. Source provenance — the beyond-Perl showcase (issues #47, #92)

**Prioritized showcase**, designed in
[`SOURCE_PROVENANCE.md`](SOURCE_PROVENANCE.md). Live source ↔ preview over a
shared locator substrate, with **two clients**: the **ar5iv-editor**
(CodeMirror + live HTML preview web UI) and a **VSCode extension** (webview
preview), both syncing identically on every edit. The same substrate gives
accurate linting (#47) and Rust-compiler-grade author error messages (#92)
for free. Perl chased this for a decade (brucemiller/LaTeXML#101) and never
cracked the accuracy; Rust's data model removes the blocker (provenance
out-of-band, `Token` stays 8 bytes).

- **Tier A** (near-term, parity-neutral): plumb the existing box-level
  `Locator` to DOM nodes behind `--source-map` → the editor sync + better
  error locators. `Locator::to_attribute()` already emits the right form.
- **Tier B** (the linting payoff): out-of-band token/char expansion
  provenance — **do not widen `Token`**.
- **Process model:** a persistent server/LSP with warm state + debounced
  incremental reconversion — the editor backend, not optional.

Design pull on current work: don't discard locator info where keeping it is
cheap and parity-neutral. #199 (HTML-dialect RelaxNG) gives the preview a
validation contract.

## 10. Corrections to the codex pass

- **#191 (CLI) is PARTIAL, not closeable** — `clap` 4 derive adopted; 2026-07-09
  wired every flag whose engine feature already works (`--strict`,
  `--includestyles`, `--debug`, `--navtoc`, plus batch 2: `--timestamp`,
  `--icon`, `--nographicimages`, `--numbersections`, `--mathparse` +
  `--invisibletimes`/`--defaultresources`). `--validate` postponed to the next
  release (gated on a rust-libxml RelaxNG publish). Remaining = `--profile`
  (+`--mode`) and the feature-tied long tail (mathimages, svg, jats,
  crossref/index, daemon) — kept as **hard parse errors, not stubbed** (option C).
  `--output` is an intentional non-goal. Full detail in
  [`ISSUE_AUDIT.md`](ISSUE_AUDIT.md).
- **Single-binary smoke test exists** (`tests/001_single_binary_smoke.rs`) —
  §7 is promote/extend, not create.
- **BibTeX is ported** (Phases 1–8, [`archive/BIBTEX_PORT_PLAN_2026-06-20.md`](archive/BIBTEX_PORT_PLAN_2026-06-20.md));
  the stale SYNC_STATUS "unported" line is fixed. B1–B6 / Phase 4–5 polish is
  product correctness; `--nobibtex` is not the default escape hatch.
- **#47 is not purely post-1.0** — Tier A is near-term.

## 11. Editor distribution — the rust-analyzer model (issues #47, #92, #217, #143)

§9's "persistent server/LSP" is the editor backend; this section is **how it
reaches every VSCode user (and beyond)**, modeled on rust-analyzer: a native
LSP server binary + a thin client extension. The bundling mechanism is the
easy part — **Stage 1 (a self-contained cross-platform binary) is the gate**,
an engine/release-pipeline effort, not extension code.

**Architecture (correct, already in place).** `latexml_oxide --server` — the
warm-preamble, fork-isolated, resource-guarded JSON-RPC/LSP server — *is* the
server; the extension only spawns + supervises it. Two invariants to keep:
- **Editor-agnostic.** Diagnostics/linting (#47) and author errors (#92) must
  ride **standard LSP** (`publishDiagnostics`), so the same server serves
  Neovim/Emacs/Helix/Zed, not just VSCode. Only the *preview* is a custom
  request (`latexml/convert`) + a VSCode webview — rust-analyzer likewise
  mixes standard LSP with custom requests.
- **Supervised subprocess, never in-process.** A runaway must not take down
  the editor, so the per-conversion **timeout + RAM (`--timeout` /
  `--max-memory`) + fork-reap + same-document preemption guards are mandatory**
  (built; shared `latexml_core::watchdog::Watchdog`). This is *why* the engine
  stays an out-of-process binary rather than an in-process N-API `.node`
  addon — an addon in the extension host can't be killed without killing
  VSCode. (`.node` ≠ WASM; WASM is Stage 4.)

**Stage 1 — self-contained, cross-platform binary (the pacing item).**
Nothing reaches macOS/Windows until `latexml_oxide` *builds and runs* there
**without the user installing system libs**:
- **Static/vendor the C deps** (libxml2, libxslt) so the shipped binary is
  self-contained — the real work; §3's "host system libs" allowance does not
  hold for an editor-bundled binary.
- **CI release matrix**: {linux, macOS, windows} × {x64, arm64} (a subset of
  §3's ladder), each emitting a self-contained `latexml_oxide`; per-target
  size + smoke + *no-host-lib-dependency* gate (extend §7).
- **kpathsea / TeX Live stays out of scope** (CLAUDE.md) and is *expected* on
  the user's machine; when it is absent the editor must **degrade with a clear
  diagnostic, not crash** — we do not vendor a TeX tree.

**Stage 2 — turnkey distribution (once Stage 1 exists).**
- **Platform-specific VSIXes** (`vsce package --target linux-x64 | win32-x64 |
  darwin-arm64 | …`), each bundling the matching self-contained binary; the
  Marketplace auto-serves the right one → single install, works immediately,
  no download, no path. The turnkey end-state.
- **Download-on-activation fallback** for unlisted platforms / a universal
  VSIX — fetch the matching release asset into extension global storage,
  checksum-verified, cached (mirrors the ar5iv-editor `managedServer`
  pattern). The near-term interim while the platform-VSIX matrix is built.
- **PATH / explicit-path override** for devs (works today via
  `ar5iv.latexmlOxidePath`).

**Stage 4 — WASM (web-only).** The native model fails *only* in the web
extension host (vscode.dev, github.dev): no subprocess. That alone needs a
`wasm32` server build — hard for the same libxml2/libxslt reason — and is a
separate, later track. **Not** required for "all desktop VSCode users."

| Stage | Deliverable | Reaches |
|---|---|---|
| 0 (done) | `--server` + guards + thin client | linux desktop (PATH / download) |
| 1 | self-contained cross-platform binary matrix | all desktop OSes — **pacing item** |
| 2 | platform-specific VSIXes (+ download fallback) | all desktop VSCode, turnkey |
| 3 | standard-LSP diagnostics/linting (#47/#92) | all LSP editors |
| 4 | WASM server | web / browser editors |
