# Release Criteria — what must be true before a public 1.0

The release-readiness contract: speed, RAM, size, portability, licensing, safety and downstream tooling. Parity with
Perl LaTeXML on the arXiv corpus, without downgrading errors, is the other mission; its worklist is
[`SYNC_STATUS.md`](../SYNC_STATUS.md). This file states each criterion and where it stands; §10 records positions
that correct the original review this list grew from.

## 1. Gates

The test count is not copied here (it changes with every PR): the gate is a green `cargo nextest run --workspace` on
the release commit, which the tag's `ci` and `windows-ci` jobs rerun on every platform before `release` publishes.

| Gate | Current | Target |
|---|---|---|
| `cargo nextest run --workspace` | green on the release commit (Linux and macOS through `release.yml`'s `ci`, Windows through `windows-ci`) | green |
| `cargo clippy --all-targets` | 0 warnings | 0 |
| Corpus | full arXiv, November 2026: 98.1 % of 2,962,950 articles error-free; 100k warning subset, 2026-05-24: ~99.39 % (~99.44 % rerun-adjusted) | no regression; gate cohorts separately (no-problem, warning subset, random full sample, hard package/class) |
| Tail latency / RSS | P50/P90/P99 + RSS dashboard and growth gate built (`tools/telemetry_dashboard.py`); its absolute red lines gate today | §5: commit a fleet baseline and wire `--gate --baseline` into the release |
| Binary size (`maxperf`) | x86_64 Linux tarball 24.1 MB at 0.8.0-rc5 (GitHub asset size; 20.7 MB at 0.7.6); both Linux legs check the stripped binary against a 64 MB budget | budget + growth alarm (§2) |
| OS/arch | `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`, `aarch64-apple-darwin`, `x86_64-apple-darwin`, `x86_64-pc-windows-msvc`, and the GHCR images (CLI amd64/arm64, worker amd64) | the §3 ladder; next rungs: a macOS universal binary, then musl |
| Toolchain | nightly, floating in `rust-toolchain.toml`; each release pins one dated nightly and one `Cargo.lock` for every leg and attaches both (`BUILD-INPUTS`, `Cargo.lock`) | reproducible per release (#143) — met |
| License inventory | inventoried and gated ([`LICENSE_INVENTORY.md`](LICENSE_INVENTORY.md)); notices assembled by the release workflow | §4 — met |
| Safety | local-CLI model ([`SAFETY.md`](SAFETY.md)); Lua sandbox, archive and stylesheet containment in place | the remaining §6 items, scoped to 1.0 |

## 2. Binary size (issue #101)

The size is **code, not data**:
- `.text` was ≈ 36.7 MB (`size -A`) when the binary was 45 MB; re-derive it with `cargo bloat` before acting. The
  compile-time binding pool (`latexml_package::pool::*::load_definitions`) dominates: ~56 % of `.text` in #101's
  2023 `cargo bloat`, and grown since.
- The dumps are not a driver: they are embedded gzipped (DEP-12, `embedded_dumps.rs`), ~7.6 MB of text in ~870 KB
  of `.rodata`, inflated lazily at bootstrap. Do not re-propose gzipping them.

Levers: (1) re-run `cargo bloat` to refresh the attribution; (2) make the binding pool's code denser (relates to
#93). `maxperf` already does fat LTO, `codegen-units=1`, strip and `panic=abort`. Gate: the release legs print the
size and fail past the 64 MB budget (§7).

## 3. Portability (issues #217, #143)

Five self-contained platform builds ship, each embedding our XSLT/CSS/JS/schema/dumps and needing only TeX Live on the
host ([`RELEASING.md`](RELEASING.md) → "Release asset strategy"):

| Target | Runner | Notes |
|---|---|---|
| `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu` | `ubuntu-22.04`, `ubuntu-22.04-arm` | glibc 2.35+; tarball and `.deb` each |
| `aarch64-apple-darwin` | `macos-15` | the arch CI's macOS suite runs on |
| `x86_64-apple-darwin` | `macos-15-intel` | `MACOSX_DEPLOYMENT_TARGET=10.13`; GitHub retires the Intel runner ~Fall 2027 |
| `x86_64-pc-windows-msvc` | `windows-latest` | one fully static `.exe` (`+crt-static`, in-process libkpathsea built from source); [`WINDOWS_COMPATIBILITY_PLAN.md`](WINDOWS_COMPATIBILITY_PLAN.md) |
| GHCR images | native amd64/arm64 | `ghcr.io/dginev/latexml-oxide` (CLI) and `…/cortex-worker`, from one root `Dockerfile` |

A native binary is never cross-OS, so each `(OS, arch)` triple is its own artifact, built on its own native runner,
not cross-compiled. Each rung needs a smoke conversion, the size gate and a dependency check.

**Self-contained linkage.** Every leg statically links libxml2, libxslt, libexslt and libkpathsea (source-built PIC
archives: `tools/build_static_libxml.sh`, `tools/build_static_kpathsea.sh`; on Windows vcpkg's static libxml2/libxslt
and `kpathsea_sys`'s `build_from_source`), and a release step asserts with `ldd`/`otool`/`dumpbin` that none of them
stays dynamic. libxml2 is built `--without-zlib --without-lzma --without-icu` and libxslt `--without-crypto`, so only
the platform runtime stays dynamic (glibc and libm on Linux; libSystem and libiconv on macOS). The binary therefore
runs on any glibc-2.35+ Linux whatever its libxml2 SONAME (2.14 moved `.so.2` → `.so.16`), and the `.deb` declares no
libxml2 dependency. At runtime `select_kpaths` may still choose the subprocess `kpsewhich` backend (MiKTeX, or MacTeX,
which ships no libkpathsea); the library stays linked either way. The default dev build (`cargo build`) links
dynamically; static linking is the release path only.

**Nightly (#143).** Required (`#![feature(thread_local)]`). Each release resolves one dated nightly in its `prepare`
job and builds every leg with it; `rust-toolchain.toml` keeps floating for development.

Deferred: musl; a macOS `lipo` universal binary (due when the Intel runner retires). The editor-distribution bar
(§11) is met by these same assets.

## 4. License audit

Our crates are CC0, but the binary ships more. The living inventory is [`LICENSE_INVENTORY.md`](LICENSE_INVENTORY.md);
every item below is in place.

- **Rust dependencies:** the `deny.toml` allow-list and cargo-deny in CI (`cargo deny --all-features check licenses`).
- **Embedded assets:** the Perl LaTeXML CSS/XSLT/RelaxNG and JavaScript are public domain (NIST, ≈ CC0); the
  exceptions (the W3C/Mozilla SVG schema, rustdoc's Ayu palette, the bundled `ar5iv.css`, MIT) carry their notices.
- **Graphics and TeX tools:** run as subprocesses only (`Command::new`), never linked, so their (A)GPL terms do not
  propagate.
- **Dumps (TeX Live-derived):** not in git; embedded at release build time, derived from the LaTeX kernel (LPPL 1.3c)
  and plain TeX (Knuth). The CC0 claim covers our source; `THIRD-PARTY-NOTICES` (`tools/gen_notices.sh`: the
  hand-authored sections plus the cargo-about appendix) attributes the kernel, plain TeX, the linked libraries and the
  Rust crates, and the README scopes the claim.
- **Release wiring:** the `notices` job assembles and content-gates the notices once for every leg (§7).

## 5. Tail latency and RSS

The public-quality risk is outliers, not the mean: the 60 s+ timeout and Fatal tail, math-parse ambiguity explosions
([`archive/MATH_AMBIGUITY_AUDIT_2026-05-21.md`](../archive/MATH_AMBIGUITY_AUDIT_2026-05-21.md)), 4 GiB allocation
failures, high-RSS package loads, and ar5iv limits creeping to hide over-evaluation. Gate "no unbounded growth"
separately from "the mean beats Perl".

**Built.** [`tools/telemetry_dashboard.py`](../../tools/telemetry_dashboard.py) reads a `telemetry.jsonl.gz` (or a
directory of output ZIPs; schema in [`TELEMETRY.md`](../performance/TELEMETRY.md)) and reports P50/P90/P99/max wall
time and peak RSS, the phase that drives the P99, and the slowest, highest-RSS, fatal/timeout/error and
math-ambiguity witnesses. `--gate` enforces absolute red lines (peak RSS against the 4 GiB wall,
`--rss-redline-mb`; P99 wall against the timeout, `--wall-redline-frac`; any hard timeout) and an optional
regression check against a committed baseline (`--baseline`, `--update-baseline`, `--tolerance`). The phase rollup is
[`tools/perf_phase_summary.py`](../../tools/perf_phase_summary.py). The live view runs in the cortex frontend
(`GET /telemetry/<corpus>/<service>` and `/api/telemetry/...`), reading each run's `telemetry.json` from its result
archives.

**Measured (2026-07-10, 60,469 documents).** No timeouts; wall median 3.07 s, P99 34.9 s, max 149 s (120 papers,
0.20 %, over 60 s; 4 over 120 s; none at 180 s); peak RSS median 0.94, P99 2.05, max 4.52 GiB (one document over
4 GiB, 53 or 0.09 % over 3 GiB). The worst tail is a reliability cluster (digest-runaway Fatals,
[`STABILITY_WITNESSES.md`](../performance/STABILITY_WITNESSES.md)), not unbounded growth. Phase budget:
[`ARXIV_PERFORMANCE.md`](../performance/ARXIV_PERFORMANCE.md), "Corpus-wide phase budget 2026-07-10".

**Remaining:** capture a representative fleet baseline (`--update-baseline`), commit it, and wire `--gate --baseline`
into the release job.

## 6. Safety: the distribution profile

[`SAFETY.md`](SAFETY.md) describes a local batch compiler; a public deployment (arXiv-scale hostile input, published
HTML/SVG) needs more. In place: document Lua runs sandboxed; archive entries, `00README` main files, image copies and
stylesheet copies stay inside their directories; `cortex_worker` never loads a runtime binding. Do not change converter
behaviour for this during parity work; document what is safe where. Open, for 1.0:
- **URIs:** `\href{javascript:…}` and data URLs pass through. Keep sanitising out of the parity converter: an optional
  output pass, or the downstream consumer's responsibility.
- A CSP for published HTML/SVG; sandboxing the image converters beyond a timeout and a process-group kill; whether a
  `--hardened` / `--public-html` profile should diverge from parity (cf. §8).

## 7. CI proves the artifact's properties

`CI.yml` stays RAM-bounded for the test suite. The artifact's properties are proven in `release.yml`, so per-PR CI
cost is unaffected:
- **Size budget** (§2): 64 MB, a runaway-growth alarm, on both Linux legs.
- **Self-contained:** `ldd`/`otool`/`dumpbin` assert that no bundled C library stays dynamic.
- **Embedded resources and a real conversion:** on all five legs, `tools/release_smoke.sh` converts `hello.tex` with
  the shipped binary and `resources/` moved aside; `cluster_cli::single_binary_smoke` isolates the working directory.
- **Version and signature:** `--version` prints the tag and commit; both macOS legs verify the ad-hoc signature.
- **Notices:** the `notices` job runs `tools/gen_notices.sh` and `tools/audit_vendored_natives.py`; the notices ship in
  every archive, both `.deb`s (`/usr/share/doc`) and as a release asset.

Per-PR in `CI.yml`'s `lint` job: `cargo clippy --all-targets`, and cargo-deny (license allow-list and RUSTSEC
advisories).

Deferred: an `strace` proof of no own-resource disk reads (the moved-aside smoke covers it functionally); a
graphics-tool matrix (with and without `gs`/`mutool`/`pdftocairo`, asserting graceful degradation). Out of scope for
CI: the corpus smoke and telemetry, which run on the sandbox fleet (too expensive for a workflow).

## 8. Surpass-Perl policy

Many open clusters are failures Perl shares, not Rust regressions:
- Both fail on malformed or undefined source → shared failure, no fix.
- Both fail only because a binding could read an argument with a more correct parameter type without harming valid
  content → a documented surpass-Perl fix is allowed.
- A visible change of output shape beyond error recovery → an [`OXIDIZED_DESIGN.md`](../parity/OXIDIZED_DESIGN.md)
  entry and a witness comparison.
- Report the opportunity upstream where practical.

"Do not downgrade errors" is non-negotiable. The surpass-Perl cases taken so far are entries in
[`OXIDIZED_DESIGN_DIVERGENCES.md`](../parity/OXIDIZED_DESIGN_DIVERGENCES.md).

## 9. Source provenance — the beyond-Perl showcase (issues #47, #92)

Designed in [`SOURCE_PROVENANCE.md`](../performance/SOURCE_PROVENANCE.md): live source ↔ preview over a shared locator
substrate, with two clients, the ar5iv-editor (CodeMirror and a live HTML preview) and a VSCode extension (a webview
preview), both syncing on every edit. The same substrate gives accurate linting (#47) and compiler-grade author error
messages (#92). Perl pursued this for a decade (brucemiller/LaTeXML#101); keeping provenance out of band, with
`Token` at 8 bytes in the default build, removes the blocker.

- **Tier A** (near-term, parity-neutral): carry the box-level `Locator` to DOM nodes behind `--source-map`, for the
  editor sync and better error locators. `Locator::to_attribute()` already emits the right form.
- **Tier B** (the linting payoff): out-of-band token and character expansion provenance. The default `Token` stays
  8 bytes; the opt-in `token-locators` feature widens it.
- **Process model:** a persistent server with warm state and debounced incremental reconversion — the editor
  backend.

On current work: do not discard locator information where keeping it is cheap and parity-neutral. #199 (the
HTML-dialect RelaxNG) gives the preview a validation contract.

## 10. Positions that correct the original review

- **CLI (#191, closed by #276).** `clap` 4 derive; every flag whose engine feature works is wired. `--validate` waits
  on a rust-libxml RelaxNG release. `--profile`/`--mode` and the feature-tied long tail (math images, SVG, JATS,
  crossref/index, daemon) are hard parse errors, not stubs; `--output` is a non-goal
  ([`WISDOM.md`](../parity/WISDOM.md) #83).
- **A single-binary smoke test exists** (`cluster_cli::single_binary_smoke`); §7 extends it.
- **BibTeX is ported** ([`archive/BIBTEX_PORT_PLAN_2026-06-20.md`](../archive/BIBTEX_PORT_PLAN_2026-06-20.md));
  `--nobibtex` is not the default escape hatch.
- **#47 is not purely post-1.0:** Tier A (§9) is near-term.

## 11. Editor distribution — the rust-analyzer model (issues #47, #92, #217, #143)

§9's persistent server reaches editors the way rust-analyzer does: a native server binary and a thin client
extension.

**Architecture (in place).** `latexml_oxide --server`, the warm-preamble, fork-isolated, resource-guarded
JSON-RPC/LSP server, *is* the server; the extension only spawns and supervises it. Two invariants:
- **Editor-agnostic.** Diagnostics and linting (#47) and author errors (#92) ride standard LSP
  (`publishDiagnostics`), so the same server serves Neovim, Emacs, Helix and Zed. Only the preview is a custom request
  (`latexml/convert`) and a VSCode webview.
- **A supervised subprocess, never in-process.** A runaway must not take the editor down, so the per-conversion
  timeout and memory limit (`--timeout`, `--max-memory`), fork-reaping and same-document preemption are mandatory
  (built, on the shared `latexml_core::watchdog::Watchdog`). This is why the engine stays an out-of-process binary and
  not an N-API `.node` addon, which cannot be killed without killing VSCode.

| Stage | Deliverable | Reaches | Status |
|---|---|---|---|
| 0 | `--server` + guards + thin client | Linux desktop (PATH or download) | done |
| 1 | a self-contained binary per platform: static C libraries, per-target size, smoke and no-host-library gates | all desktop OSes | met by the five release assets (§3); a Windows arm64 build is not made |
| 2 | platform-specific VSIXes (`vsce package --target linux-x64 \| win32-x64 \| darwin-arm64 \| …`), with a download-on-activation fallback (fetch the matching release asset into the extension's storage, checksum-verified, cached) and a PATH or explicit-path override (`ar5iv.latexmlOxidePath`, works today) | all desktop VSCode, turnkey | open |
| 3 | standard-LSP diagnostics and linting (#47, #92) | all LSP editors | open |
| 4 | a `wasm32` server, for the web extension host (vscode.dev, github.dev), which has no subprocess | web editors | separate track: [`WASM_COMPATIBILITY_AUDIT_2026-09-03.md`](WASM_COMPATIBILITY_AUDIT_2026-09-03.md), [`WASM_COMPATIBILITY_PLAN.md`](WASM_COMPATIBILITY_PLAN.md) |

TeX Live stays out of scope and is expected on the user's machine; where it is absent the editor must degrade with a
clear diagnostic, not crash. We do not bundle a TeX tree.
