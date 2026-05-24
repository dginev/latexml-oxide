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
| OS/arch | `x86_64-linux-gnu` only | staged ladder — §3 |
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

Current: one self-contained `x86_64-linux-gnu` artifact (Ubuntu 22.04 /
glibc 2.35), embedding our XSLT/CSS/JS/schema/dumps, host TeX Live +
system libs ([`RELEASING.md`](RELEASING.md)). Ladder — each stage needs a
smoke corpus + size gate + dependency check:

1. Debian/Ubuntu x86_64 (current).
2. aarch64 Linux.
3. Container image (reproducible TeX Live + graphics).
4. macOS (#217) — blocker is `libkpathsea-dev`; needs `rust-kpathsea` to
   find MacTeX headers via `pkg-config` + README + CI sanity job.
5. Windows / musl — deferred.

**Nightly (#143):** required (`thread_local`). For a long-lived tool, a
reproducibility risk — pin a known-good nightly, track stabilization.
"Carries our resources" ≠ "portable across platforms."

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
([`MATH_AMBIGUITY_AUDIT_2026-05-21.md`](MATH_AMBIGUITY_AUDIT_2026-05-21.md));
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
inkscape present-or-not); `cargo clippy --all-targets`.

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

## 9. Source provenance & tooling (issues #47, #92) — product north star

#47 (linting) and #92 (error reporting) are one capability and the
foundation for a **VSCode plugin with synced source↔preview**. One track,
two tiers, plus a process-model prerequisite.

**Already in the code (don't rebuild):** `Locator` (24-byte `Copy`) already
emits LaTeXML's `range(from='l;c',to='l;c')` / `point('l;c')` via
`common/locator.rs::to_attribute()`; box/whatsit/error nodes already carry
`Locator`.

- **Tier A — element-level provenance (near-term, enables synced preview):**
  plumb the existing box-level `Locator` to DOM nodes behind `--source-map`.
  Mostly wiring. Gives click-element↔source-line (and inverse via
  `querySelector`). Opt-in so default HTML stays compact / leaks no paths.
- **Tier B — token/char expansion provenance (post-1.0, accurate linting):**
  the `\def\au{au}\au{}tor`→`autor` case needs sub-element provenance across
  macro boundaries. **Constraint: do not widen `Token`** (deliberately 8
  bytes; `common/locator.rs:7` notes token-locators were tried and abandoned
  as a hot-path regression). Carry provenance out-of-band (side table keyed
  by mouth+offset). Distinguish literal spans from expansion spans.
- **Process model (the gap codex missed):** #47 wants "near-instant"
  conversion, but the binary cold-starts and re-parses ~24k dump entries
  every run. The real risk is a **persistent server / LSP mode** (warm
  state, debounced incremental reconversion) — also the natural host for #92.

Design pull on current work: don't discard locator info where keeping it is
cheap and parity-neutral. Related: #199 (HTML-dialect RelaxNG) gives the
preview a validation contract — [`SCHEMA_DOCUMENTATION.md`](SCHEMA_DOCUMENTATION.md).

## 10. Corrections to the codex pass

- **#191 (CLI) essentially done** — all binaries use `clap` 4 derive; only
  Perl `Config.pm` option-coverage parity remains.
- **Single-binary smoke test exists** (`tests/001_single_binary_smoke.rs`) —
  §7 is promote/extend, not create.
- **BibTeX is ported** (Phases 1–8, [`BIBTEX_PORT_PLAN.md`](BIBTEX_PORT_PLAN.md));
  the stale SYNC_STATUS "unported" line is fixed. B1–B6 / Phase 4–5 polish is
  product correctness; `--nobibtex` is not the default escape hatch.
- **#47 is not purely post-1.0** — Tier A is near-term.
