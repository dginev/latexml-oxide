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

Numbers are verified current state (2026-05-24) unless marked TODO.

| Gate | Current | Target |
|---|---|---|
| `cargo test --tests` | 1334/0/0 | green |
| `cargo clippy --all-targets` | 14 (all `latexml_math_parser`) | 0 |
| Corpus (100k warning subset) | ~99.39% / ~99.44% rerun-adj | no regression; gate cohorts separately (`no-problem`, warning subset, random full sample, hard package/class) |
| Tail latency / RSS | mean bands only ([`PERFORMANCE.md`](PERFORMANCE.md)) | P50/P90/P99 dashboard; "no unbounded growth" gate — §5 |
| Binary size (`maxperf`) | **45 MB / 14 MB tarball** | budget + growth alarm — §2 |
| OS/arch | `x86_64-linux-gnu` + `aarch64-apple-darwin` | staged ladder — §3 |
| Toolchain | **nightly** (`#![feature(thread_local)]`) | pin nightly; track stabilization (#143) |
| License inventory | crates `CC0`; embedded assets uninventoried | blocker — §4 |
| Safety | local-CLI model ([`SAFETY.md`](SAFETY.md)) | + distribution profile — §6 |

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

Current: **two** self-contained published artifacts — `x86_64-linux-gnu`
(Ubuntu 22.04 / glibc 2.35) and `aarch64-apple-darwin` (macOS Apple
Silicon) — each embedding our XSLT/CSS/JS/schema/dumps, host TeX Live +
system libs ([`RELEASING.md`](RELEASING.md) → "Release asset strategy").
A native binary is never cross-OS (ELF vs Mach-O), so it is one artifact
per `(OS, arch)` triple, built on its own native runner — not
cross-compiled. Ladder — each stage needs a smoke corpus + size gate +
dependency check:

1. Debian/Ubuntu x86_64 (current).
2. aarch64 Linux.
3. Container image (reproducible TeX Live + graphics).
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
   *Not yet published:* Intel macOS (`x86_64-apple-darwin`) — needs a
   `macos-13` leg or a `lipo` universal; arm64 binaries don't run on
   Intel.
5. Windows / musl — deferred. Known blockers: `libmarpa-sys`
   `./configure && make` (needs a cc-crate port; tarball is vendored),
   `lsp_server` unix sockets, `graphics*.rs` cfg(unix) paths,
   vcpkg-sourced libxml2/libxslt. The subprocess-`kpsewhich` fallback
   already removes the kpathsea blocker (MiKTeX's kpsewhich.exe
   delegates to MiKTeX's own resolver — better than linking could do).

**v0.7.1 target — self-contained libxml2/libxslt (the SONAME-portability
completion).** 0.7.0 dynamically links the build host's libxml2/libxslt
SONAME (`libxml2.so.2` on the ubuntu-22.04 runner). That binary loads on
every libxml2-2.x system (Ubuntu 22.04/24.04 LTS, Debian 12 — ~20 years
of a stable `.so.2`) but **NOT** on libxml2 ≥ 2.14, which bumped its
SONAME to `libxml2.so.16`. A SONAME change is, by the rules of shared
libraries, the loader's "not ABI-compatible" token — so no symlink
bridges `.so.2` ↔ `.so.16` by design; that's the mechanism working, not
failing. Fix = static-link libxml2 + libxslt + libexslt into the binary
(the kpathsea playbook: PIC static `.a`, needed because the proc-macro
cdylib links libxml via `latexml_core`; `libxml2-dev` ships `libxml2.a`
but `libxslt`/`libexslt` have no packaged `.a` → source-build; transitive
`-lz`/`-lgcrypt` stay dynamic — stable SONAMEs, no churn). After kpathsea
(done) + these three, the only dynamic deps left are the glibc family +
zlib + libgcrypt — all stable-SONAME, so the binary becomes "any
glibc-2.35+ Linux, any libxml/libxslt version." **Test bed: this dev box
runs libxml2.so.16 (2.15.2)** — the 0.7.0 `.so.2` binary fails to load
here, so a successful `latexml_oxide --version` on the dev box is the
portability gate for the static-linked build.

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

Crates are `CC0`, but the binary ships more. Before any public-domain claim:
- **Embedded-asset inventory** (origin + license): XSLT, RelaxNG schema,
  CSS/JS, and the sharp edge — `resources/dumps/`.
- **Dumps are TeX-Live-derived** (`tools/make_formats.sh`) → *not*
  automatically CC0; needs a written position.
- Rust dep license report (`cargo deny`/`cargo about`).
- Confirm GPL/AGPL graphics tools stay subprocess-only (never linked).
- CI checks release-artifact contents against the inventory.

## 5. Tail latency & RSS

The public-quality risk is outliers, not the mean: 60s+ timeout/fatal tail;
math-bocage ambiguity explosions
([`archive/MATH_AMBIGUITY_AUDIT_2026-05-21.md`](archive/MATH_AMBIGUITY_AUDIT_2026-05-21.md));
4 GiB alloc failures; high-RSS package loads; ar5iv limit creep hiding
over-evaluation. Build a rolling dashboard from `telemetry.jsonl.gz`
(schema exists, [`TELEMETRY.md`](TELEMETRY.md)): P50/P90/P99 wall+RSS, top
fatal/timeout/ambiguity witnesses. Gate "no unbounded growth" *separately*
from "mean beats Perl."

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

`CI.yml` is RAM-bounded for the test suite (correct). `release.yml` only
proves "it built." Add (release or beefier scheduled runner): size budget
gate (§2); embedded-resource smoke — *promote/extend* existing
`tests/001_single_binary_smoke.rs` + the "rename `resources/` away" check;
`strace` no-own-disk-read assertion; license-inventory check (§4); corpus
smoke + telemetry upload; graphics-tool matrix (pdftocairo / mutool /
gs present-or-not); `cargo clippy --all-targets`.

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

- **#191 (CLI) is PARTIAL, not done** (audited 2026-05-24) — `clap` is
  adopted and core options work, but only ~47 of ~95 Perl omni flags exist.
  Missing routing/parity options (`--profile`, `--strict`, `--includestyles`,
  `--validate`, `--mode`, `--debug`, `--navtoc`, `--mathml`) and a
  feature-tied long tail (mathimages/svg, jats, crossref/index, daemon mode).
  `--output` is an intentional non-goal. Detail in
  [`ISSUE_AUDIT.md`](ISSUE_AUDIT.md). (Earlier "essentially done" was wrong.)
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
