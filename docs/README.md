# latexml-oxide documentation

The front door to the internal docs. Files are grouped into themed subdirectories
matching the project's two mission targets; this page is the multi-level table of
contents. **Resuming work? Start with [`SYNC_STATUS.md`](SYNC_STATUS.md).**

> **What this project is.** latexml-oxide is a faithful Perl→Rust translation of
> [LaTeXML](https://github.com/brucemiller/latexml). Two co-equal targets drive
> the work: **(1) faithful parity** with the original Perl (the Perl source is
> ground truth), and **(2) beyond-Perl improvement runs** over the ~2.8M-doc arXiv
> corpus (levers Rust affords that single-threaded, libxslt-bound Perl cannot).
> The doc themes below mirror that split.

---

## 🧭 Start here — worklists & contracts

The live worklists and the ship contract. Read these first when resuming.

| Doc | What it is |
|-----|------------|
| [`SYNC_STATUS.md`](SYNC_STATUS.md) | **The brief actionable worklist for both targets.** Opens with *How to read this file* + a **ranked worklist (R1…R9)** — take the top unblocked row. Then: current status, per-row detail, standing policies, parked-family pointers, stable reference. Completed logs lift to `archive/`. |
| [`parity/BIBLIOGRAPHY_WORKLIST.md`](parity/BIBLIOGRAPHY_WORKLIST.md) | **R5** — surveyed missing-references targets + the MakeBibliography full-parity re-port. |
| [`performance/BEYOND_PERL_LEVERS.md`](performance/BEYOND_PERL_LEVERS.md) | **R7** — BP-1…BP-6 levers from the 60k-doc telemetry; POST-RELEASE. |
| [`math/CONTENT_MATHML_GAPS.md`](math/CONTENT_MATHML_GAPS.md) | **R8** — content-MathML / math-parser gaps; deferred by user directive, do not pick off in isolation. |
| [`parity/DEFERRED_FAMILIES.md`](parity/DEFERRED_FAMILIES.md) | **R9** — parked deep families (`.bst`, xy-pic, mode-frame, …); several carry explicit "do NOT start". |
| [`release/RELEASE_CRITERIA.md`](release/RELEASE_CRITERIA.md) | The "what must be true before a public 1.0" contract: gates, binary-size budget, portability, license audit, tail-latency/RSS signals. |
| [`release/RELEASING.md`](release/RELEASING.md) | Tag-driven release procedure; the self-contained-binary requirement. |
| [`release/CRATES_IO_PUBLISH.md`](release/CRATES_IO_PUBLISH.md) | `cargo publish` + docs.rs + library-use story: bottom-up publish order, open blockers (workspace-`resources/` packaging, `pericortex` git dep), docs.rs metadata, `latexml::api` entrypoint. |
| [`release/LICENSE_INVENTORY.md`](release/LICENSE_INVENTORY.md) | Living license inventory for the redistributable binary (scopes the CC0 claim). |
| [`release/ISSUE_AUDIT.md`](release/ISSUE_AUDIT.md) | Local mirror of open GitHub issues with status + interpretation. |
| [`release/SAFETY.md`](release/SAFETY.md) | Threat model and `unsafe` inventory. |
| [`AR5IV_DIAGNOSTICS.md`](AR5IV_DIAGNOSTICS.md) | The ar5iv issue-tracker sweep: every open "Improve article X" report screened against the current binary and classified vs same-host Perl, plus the ranked worklist. Re-measured 2026-07-20 on top of the 2026-07-18 snapshot. |

## 🎯 Target 1 — faithful Perl translation (`parity/`)

Strict parity at the dump/format boundary plus corpus-driven parity mining.

### Design & orientation
| Doc | What it is |
|-----|------------|
| [`parity/OXIDIZED_DESIGN.md`](parity/OXIDIZED_DESIGN.md) | Public-facing design **index + overview** (principles, architecture). Links the themed family below. |
| [`parity/OXIDIZED_DESIGN_DIVERGENCES.md`](parity/OXIDIZED_DESIGN_DIVERGENCES.md) | The numbered **intentional Perl divergences** that `.rs` comments cite as `OXIDIZED_DESIGN #N`. |
| [`parity/OXIDIZED_DESIGN_TYPES.md`](parity/OXIDIZED_DESIGN_TYPES.md) | Type-system improvements + tactical pitfalls. |
| [`parity/OXIDIZED_DESIGN_FUTURE_WORK.md`](parity/OXIDIZED_DESIGN_FUTURE_WORK.md) | Future-work backlog. |
| [`parity/ORGANIZATION.md`](parity/ORGANIZATION.md) | Maps Perl engine files (`Engine/*.pool.ltxml`) → Rust (`latexml_engine/src/*.rs`); loading hierarchy. |

### Engine internals & known issues
| Doc | What it is |
|-----|------------|
| [`parity/WISDOM.md`](parity/WISDOM.md) | Tactical insights about system internals — check here to avoid re-introducing known bugs. |
| [`parity/KNOWN_PERL_ERRORS.md`](parity/KNOWN_PERL_ERRORS.md) | Upstream Perl LaTeXML issues; check first when investigating a test failure. |
| [`parity/DUMP_DESIGN.md`](parity/DUMP_DESIGN.md) | Kernel dump precompilation (strict LoadFormat mutual exclusivity, unconditional apply). |
| [`parity/BINDING_DSL_ARCHITECTURE.md`](parity/BINDING_DSL_ARCHITECTURE.md) | Binding-definition DSL: shared `ConstructorBuilder` spine, compile-time + runtime front-ends. |
| [`parity/script_bindings_plan.md`](parity/script_bindings_plan.md) | The runtime (Rhai) script-bindings front-end reference (the `runtime-bindings` feature; on by default). |

### Open dated diagnostics (`parity/diagnostics/`)
Point-in-time studies with pending halves.
| Doc | What it is |
|-----|------------|
| [`parity/diagnostics/EXPECTED_ID_XMREF_DESIGN_2026-06-08.md`](parity/diagnostics/EXPECTED_ID_XMREF_DESIGN_2026-06-08.md) | `expected:id` dangling-XMRef cluster: container-id half landed; MathFork reconciliation pending. |
| [`parity/diagnostics/EXPL3_CATCODE_GAP_2026-06-08.md`](parity/diagnostics/EXPL3_CATCODE_GAP_2026-06-08.md) | expl3 catcode-gap study — **largely closed** (re-measured 2026-07-20; `2110.12034` the lone regression at 8). Third member fixed 2026-07-27 (`\c` cedilla clobber). Kept for its four reverted attempts as settled dead-ends. |

## ➗ Math parser (`math/`) — serves both targets

The Marpa-style highly-ambiguous grammar that replaced Perl's Parse::RecDescent.

| Doc | What it is |
|-----|------------|
| [`math/MATH_PARSER_AND_ASF.md`](math/MATH_PARSER_AND_ASF.md) | **Canonical:** three-stage ambiguity pipeline vs the Marpa ASF traversal. Read before touching `parser.rs::parse_string` / `semantics.rs::Actions`. |
| [`math/MATH_PARSER_ASF_TIEBREAKING.md`](math/MATH_PARSER_ASF_TIEBREAKING.md) | ASF tie-breaking rules, in detail. |
| [`math/MATH_GRAMMAR_FIRST_PRINCIPLES.md`](math/MATH_GRAMMAR_FIRST_PRINCIPLES.md) | Design rationale for the Marpa grammar. |
| [`math/MATH_OVERPARSE_DEEP_DIVE_2026-06-30.md`](math/MATH_OVERPARSE_DEEP_DIVE_2026-06-30.md) | Measured and-node counts per ambiguity pattern; ranked open levers. |
| [`math/OXIDIZED_DESIGN_MATH.md`](math/OXIDIZED_DESIGN_MATH.md) | Marpa math-parser + grammar-rule design (part of the OXIDIZED_DESIGN family). |

## 🚀 Target 2 — beyond-Perl (`performance/`)

The levers Rust affords that single-threaded, libxslt-bound Perl cannot: **performance
& reliability** over the arXiv corpus, the **fleet / telemetry** infrastructure that
drives it, and the **surpass-Perl feature showcases** (source-provenance, schema doc
site) that have no Perl equivalent.

| Doc | What it is |
|-----|------------|
| [`performance/ARXIV_PERFORMANCE.md`](performance/ARXIV_PERFORMANCE.md) | Living empirical performance campaign over arXiv: slowest-100 testbed, phase rollups, optimization log. |
| [`performance/PERFORMANCE.md`](performance/PERFORMANCE.md) | Timeless optimization principles, open/closed lever state, dated audit log. |
| [`performance/STABILITY_WITNESSES.md`](performance/STABILITY_WITNESSES.md) | Living worklist of reliability witnesses (timeout/OOM/peak-RSS/hang) with current + Perl baselines. |
| [`performance/STREAMING_POST_DESIGN_2026-07-06.md`](performance/STREAMING_POST_DESIGN_2026-07-06.md) | Very-large split-document post-processing (the 614 MB `index.xml` witness); two-pass streaming split design. |
| [`performance/ISSUE_361_MEMORY_TIME_PROFILE_2026-07-24.md`](performance/ISSUE_361_MEMORY_TIME_PROFILE_2026-07-24.md) | Very-large **single** doc (#361): RAM+time diagnosis; M1 (`List.font`→`Rc`) + M2 (box `KeyVals`) + M4 (box `Whatsit`'s reversion slots) landed — 9.05 → 5.99 GB (−34 %) at unchanged wall time; M3 (stream boxes→DOM) measured and **reverted** as a dead-end. |
| [`performance/CORTEX_WORKER_HARNESS.md`](performance/CORTEX_WORKER_HARNESS.md) | `cortex_worker --harness` fleet orchestration: one-conversion-per-process, memory guards, deployment. |
| [`performance/TELEMETRY.md`](performance/TELEMETRY.md) | Per-job structured telemetry schema for `cortex_worker` runs. |
| [`performance/SOURCE_PROVENANCE.md`](performance/SOURCE_PROVENANCE.md) | Design for the source↔preview showcase over a shared locator substrate (issues #47/#92). |
| [`performance/SCHEMA_DOCUMENTATION.md`](performance/SCHEMA_DOCUMENTATION.md) | RelaxNG Compact schema → rustdoc-styled HTML doc site. |

## 📚 Reference collections (subdirectories, kept as-is)

| Directory | What it holds |
|-----------|---------------|
| [`archive/`](archive/README.md) | Completed/superseded snapshots and session logs (see its own `README.md`). |
| `reproducers/` | Single-paper reproducers for tracked bugs. |
| `out-of-scope/` | Cases intentionally out of scope (Perl also fails, no-DTD, …). |
| `known_crashes/` | Known crash records with triage. |
| `examples/` | Example bindings (e.g. `sample.sty.rhai`). |
| `scripts/` | One-off analysis helpers referenced by archived diagnostics (e.g. `bucket_callgrind_hot.py`). |

---

*Keep this index current when adding, renaming, merging, or archiving a doc.
`CLAUDE.md` at the repo root carries the same map inline (the authoritative
per-file index with the placement rules); this page is the navigational front
door. Diagnostic-snapshot docs (`*_TRIAGE`, `*_AUDIT`, `*_ANALYSIS`, …) carry a
date in the filename; living worklists do not.*

---

## Per-file detail

Moved here from the root `CLAUDE.md` on 2026-07-28 (it was a second, richer copy
of the table above, loaded into every session). The tables above stay the quick
map; this section is the authoritative per-file index, and both are maintained
together — when you add, rename, merge, or archive a doc, update the table AND
this section. The placement rules that govern what belongs where remain in
`CLAUDE.md` ("Rules for these docs"), because they are policy rather than
navigation.

The layout: `docs/SYNC_STATUS.md` (the start-here worklist) and
`docs/AR5IV_DIAGNOSTICS.md` stay at the root; **`docs/parity/`** (Target 1
faithful translation, `+ diagnostics/`), **`docs/math/`** (Marpa math parser),
**`docs/performance/`** (Target 2 beyond-Perl / arXiv), **`docs/release/`** (ship
contracts); reference collections `docs/{archive,reproducers,out-of-scope,known_crashes,examples,scripts}/`
are unchanged (`scripts/` holds one-off analysis helpers referenced by archived
diagnostics, e.g. `bucket_callgrind_hot.py`). Grouped by the two mission targets
(docs serving both come first):

**Worklists & contracts (start here when resuming work):**
- **[`docs/SYNC_STATUS.md`](docs/SYNC_STATUS.md)** — The BRIEF ACTIONABLE worklist for both targets. Opens with **"How to read this file"** and a **ranked worklist (R1…R9)** — take the top unblocked row; everything after it is supporting detail (current status, per-row detail, standing policies, parked-family pointers, stable reference). Completed session logs are lifted to `docs/archive/SYNC_SESSIONS_*.md`; a section that outgrows ~100 lines is extracted to its own doc. Labels here have gone stale before — **verify a status against the named guard test or `gh issue view` before acting on it; SHA-ancestry does not work, the repo squash-merges.** **Start here.**
- **Parked families extracted from `SYNC_STATUS.md` (2026-07-25)**, each the detail behind one ranked row: **[`docs/parity/BIBLIOGRAPHY_WORKLIST.md`](docs/parity/BIBLIOGRAPHY_WORKLIST.md)** (R5 — missing-references targets + the MakeBibliography full-parity re-port), **[`docs/performance/BEYOND_PERL_LEVERS.md`](docs/performance/BEYOND_PERL_LEVERS.md)** (R7 — BP-1…BP-6 from the 60k-doc telemetry, POST-RELEASE), **[`docs/math/CONTENT_MATHML_GAPS.md`](docs/math/CONTENT_MATHML_GAPS.md)** (R8 — deferred by user directive 2026-06-20; do not pick off in isolation), **[`docs/parity/DEFERRED_FAMILIES.md`](docs/parity/DEFERRED_FAMILIES.md)** (R9 — `.bst`, xy-pic, mode-frame and friends; several carry explicit "do NOT start").
- **[`docs/release/RELEASE_CRITERIA.md`](docs/release/RELEASE_CRITERIA.md)** — The "what must be true before a public 1.0" contract: release gates, binary-size budget, portability staging, license/public-domain audit, distribution safety profile, tail-latency/RSS signals, surpass-Perl policy.
- **[`docs/release/LICENSE_INVENTORY.md`](docs/release/LICENSE_INVENTORY.md)** — Living license inventory for the redistributable binary (the RELEASE_CRITERIA §4 deliverable): Rust deps (cargo-deny-gated), embedded assets, the TeX-Live-derived dumps position, linked syslibs, subprocess-only graphics tools. Scopes the CC0 claim.
- **[`docs/release/ISSUE_AUDIT.md`](docs/release/ISSUE_AUDIT.md)** — Local mirror of open GitHub issues with status + interpretation; the file carries its own refresh stamp (do not duplicate the count here — it drifted twice). **Refresh before milestone planning.** (Issue numbers are GitHub-tracker numbers — they do **not** correspond to any internal `#N` in `WISDOM.md`.)
- **[`docs/release/WINDOWS_COMPATIBILITY_PLAN.md`](docs/release/WINDOWS_COMPATIBILITY_PLAN.md)** — Living worklist for the Windows port (`windows-compatibility` branch): MSVC + vcpkg-static toolchain, TeX Live + MiKTeX runtime, phased plan from compile blockers (libmarpa cc-port, libxml2/libxslt vcpkg) through `cargo test --release` green on `windows-latest` CI to a zipped `.exe` release artifact. Operationalizes RELEASE_CRITERIA portability rung 5.

**Target 1 — faithful Perl translation (parity):**
- **[`docs/parity/OXIDIZED_DESIGN.md`](docs/parity/OXIDIZED_DESIGN.md)** — Public-facing design **index + overview** (guiding principles, architecture). Detail lives in a themed family it links to: **[`OXIDIZED_DESIGN_DIVERGENCES.md`](docs/parity/OXIDIZED_DESIGN_DIVERGENCES.md)** (the numbered **intentional Perl divergences** that `.rs` comments cite as `OXIDIZED_DESIGN #N`), **[`OXIDIZED_DESIGN_MATH.md`](docs/math/OXIDIZED_DESIGN_MATH.md)** (Marpa math-parser + grammar rules), **[`OXIDIZED_DESIGN_TYPES.md`](docs/parity/OXIDIZED_DESIGN_TYPES.md)** (type-system improvements + tactical pitfalls), **[`OXIDIZED_DESIGN_FUTURE_WORK.md`](docs/parity/OXIDIZED_DESIGN_FUTURE_WORK.md)**. Read the divergences file to check if a translation difference was a marked intentional divergence. (Divergence `#N` numbers are load-bearing and kept verbatim; note the pre-existing collision between divergence `#7–#18` and the math cluster `#7–#18` — the index explains which file owns each.)
- **[`docs/parity/ORGANIZATION.md`](docs/parity/ORGANIZATION.md)** — Maps Perl engine files (`LaTeXML/Engine/*.pool.ltxml`) to Rust files (`latexml_engine/src/*.rs`). Loading hierarchy and LaTeX chapter structure.
- **[`docs/parity/WISDOM.md`](docs/parity/WISDOM.md)** — Tactical insights about system internals from specialized debugging. Check here to avoid re-introducing known bugs.
- **[`docs/parity/KNOWN_PERL_ERRORS.md`](docs/parity/KNOWN_PERL_ERRORS.md)** — Upstream Perl LaTeXML issues (56 numbered entries, plus 7 unnumbered). Check here first when investigating a test failure; when a shared bug is simple, fix in Rust and record it here (candidate to upstream).
- **[`docs/parity/DUMP_DESIGN.md`](docs/parity/DUMP_DESIGN.md)** — Design record for the kernel dump precompilation (strict-Perl LoadFormat mutual exclusivity, unconditional apply) — the live architecture behind the per-TL-year release dumps. NOTE the format-layering nuance: the latex format sits on the REAL-plain.tex layer (Perl's is hand-curated), so plain-only macros can leak into latex sessions (the `\+` class, retracted at the `latex.rs` seam; audit in SYNC_STATUS 2026-07-02).
- **[`docs/parity/BINDING_DSL_ARCHITECTURE.md`](docs/parity/BINDING_DSL_ARCHITECTURE.md)** — Decision record for the binding-definition DSL: one shared `ConstructorBuilder` lowering spine, compile-time `macro_rules!` + runtime Rhai front-ends. Subsumes closed issues #93/#171.
- **[`docs/parity/script_bindings_plan.md`](docs/parity/script_bindings_plan.md)** — The runtime (Rhai) script-bindings front-end reference. Gated by the **`runtime-bindings`** feature (ON by default, and in the distribution build; the old `script-bindings` alias was removed pre-publish).

**Target 2 — beyond-Perl improvement runs over arXiv:**
- **[`docs/performance/ARXIV_PERFORMANCE.md`](docs/performance/ARXIV_PERFORMANCE.md)** — Living empirical performance campaign over arXiv: slowest-100 testbed, corpus-wide profiles, phase rollups, optimization log; records settled dead-ends.
- **[`docs/performance/PERFORMANCE.md`](docs/performance/PERFORMANCE.md)** — Timeless optimization principles, open/closed lever state, and the dated **Audit log** of periodic perf passes.
- **[`docs/performance/STABILITY_WITNESSES.md`](docs/performance/STABILITY_WITNESSES.md)** — Living worklist of reliability witness papers (timeout/OOM/peak-RSS/hang) with current-binary + Perl baselines. Distinct from `SYNC_STATUS.md` (correctness errors).
- **[`docs/performance/CORTEX_WORKER_HARNESS.md`](docs/performance/CORTEX_WORKER_HARNESS.md)** — `cortex_worker --harness` fleet orchestration: one-conversion-per-process, five-layer memory guards, crash-loop backoff, production deployment recommendation. Companion to pericortex `docs/HARNESS.md` and CorTeX `MANUAL.md` §7.
- **[`docs/performance/TELEMETRY.md`](docs/performance/TELEMETRY.md)** — Per-job structured telemetry schema for `cortex_worker` runs.
- **[`docs/performance/SOURCE_PROVENANCE.md`](docs/performance/SOURCE_PROVENANCE.md)** — Design for the prioritized beyond-Perl showcase: live source ↔ preview over a shared locator substrate (ar5iv-editor + VSCode clients), accurate linting (#47) and Rust-grade author errors (#92). Locators opt-in (`--source-map`). (The landed-but-deprioritized `--server` LSP docs: [`docs/archive/LSP_SERVER.md`](docs/archive/LSP_SERVER.md), [`docs/archive/LSP_MULTIFILE_PLAN.md`](docs/archive/LSP_MULTIFILE_PLAN.md); smoke `tools/lsp_smoke.py`.)
- **[`docs/AR5IV_DIAGNOSTICS.md`](docs/AR5IV_DIAGNOSTICS.md)** — The ar5iv issue-tracker sweep: every open "Improve article X" report screened against the current binary and classified vs same-host Perl, with the ranked worklist. Carries a 2026-07-20 re-measurement block on top of the 2026-07-18 snapshot. **Refresh before quoting any row** — a wrong main-file pick manufactures fake error counts (the file records the correct detector).
- **[`docs/release/RELEASING.md`](docs/release/RELEASING.md)** — Tag-driven release procedure; the self-contained-binary requirement.
- **[`docs/release/CRATES_IO_PUBLISH.md`](docs/release/CRATES_IO_PUBLISH.md)** — The `cargo publish` + docs.rs + library-consumer story: bottom-up publish order for the 8 crates, the open blockers (workspace-`resources/` packaging **B3**, the `pericortex` git dep **B2**), docs.rs metadata, and the `latexml::api` library entrypoint. Distinct from `RELEASING.md` (the GitHub-Release binary flow).
- **[`docs/release/SAFETY.md`](docs/release/SAFETY.md)** — Threat model and `unsafe` inventory (distribution posture in `RELEASE_CRITERIA.md` §6).
- **[`docs/performance/SCHEMA_DOCUMENTATION.md`](docs/performance/SCHEMA_DOCUMENTATION.md)** — RelaxNG Compact schema → rustdoc-styled HTML doc site (supported the closed #199 HTML-dialect schema).

**Math parser (serves both targets):**
- **[`docs/math/MATH_PARSER_AND_ASF.md`](docs/math/MATH_PARSER_AND_ASF.md)** — Canonical: the three-stage ambiguity pipeline vs the Marpa ASF traversal paradigm. Read before touching `latexml_math_parser/src/parser.rs::parse_string` or `semantics.rs::Actions`. Companion to [`marpa/ASF_STATUS.md`](https://github.com/dginev/marpa/blob/asf-completion/ASF_STATUS.md).
- **[`docs/math/MATH_PARSER_ASF_TIEBREAKING.md`](docs/math/MATH_PARSER_ASF_TIEBREAKING.md)** — ASF tie-breaking rules detail.
- **[`docs/math/MATH_GRAMMAR_FIRST_PRINCIPLES.md`](docs/math/MATH_GRAMMAR_FIRST_PRINCIPLES.md)** — Design rationale for the Marpa grammar.
- **[`docs/math/MATH_OVERPARSE_DEEP_DIVE_2026-06-30.md`](docs/math/MATH_OVERPARSE_DEEP_DIVE_2026-06-30.md)** — Measured and-node counts per ambiguity pattern; ranked open levers (`f(x)` apply-vs-multiply, bare-`|x|` pre-lexer, integral Step 2). The top `math_parse` lever for the arXiv runs; supersedes the archived 2026-05-21 ambiguity audit.

**Open dated diagnostics** (point-in-time studies with pending halves — see naming rule):
- **[`docs/parity/diagnostics/EXPECTED_ID_XMREF_DESIGN_2026-06-08.md`](docs/parity/diagnostics/EXPECTED_ID_XMREF_DESIGN_2026-06-08.md)** — `expected:id` dangling-XMRef cluster: container-id half landed; content-branch/MathFork reconciliation still pending.
- **[`docs/parity/diagnostics/EXPL3_CATCODE_GAP_2026-06-08.md`](docs/parity/diagnostics/EXPL3_CATCODE_GAP_2026-06-08.md)** — expl3 catcode-gap study: **largely closed** (re-measured 2026-07-20 — the original witnesses now convert at 0 errors; one, `2110.12034`, went the other way at 8). A third member was found and fixed 2026-07-27 (`\c_sys_*` constants emitted through `RawTeX!` under the ambient catcode table redefined the `\c` cedilla accent — guard `expl3_load_does_not_clobber_cedilla_accent`). Keep for its **settled dead-ends**: four attempted fixes that all regressed and were reverted.
- **[`docs/performance/STREAMING_POST_DESIGN_2026-07-06.md`](docs/performance/STREAMING_POST_DESIGN_2026-07-06.md)** — very-large split-document post-processing: the correctness+foundation floor is landed (limit-safe queries so split fires on the 614 MB `index.xml`, stream-from-file, rust-libxml `TextReader`/checked-XPath); the **two-pass streaming split** to cut peak RSS 15.6 GB → <1 GB is the pending, parity-gated half. New resume point for that work (was `HANDOFF.md`).
- **[`docs/performance/ISSUE_361_MEMORY_TIME_PROFILE_2026-07-24.md`](docs/performance/ISSUE_361_MEMORY_TIME_PROFILE_2026-07-24.md)** — very-large **single** document (#361, 232K-line/7.9 MB book, `--splitat=subsection`): full RAM+time diagnosis (peak 9 GB = transient digested boxes + DOM coexisting at Building; flat CPU profile = allocator + libxml2 XPath). **M1+M2+M4 landed** (`List.font`→`Rc<Font>`; box the `DigestedData::KeyVals` variant; box `Whatsit`'s two never-filled reversion-cache slots — together 9.05 → **5.99 GB**, −34 %, at unchanged wall time; `DigestedData` 424→128 B, guarded by `digested_data_size_budget`). Density is now near its floor (`TBox`/`List` bound the enum at 104 B); the box-type census that settles "box the variant?" questions is in the doc. **M3 (stream boxes→DOM) is a SETTLED DEAD-END** — implemented, byte-identical, suite-green, but only −3–4 % (inside run variance) because the box mass hangs off the constructor API's by-reference nested absorbs; reverted, do not re-attempt in that shape. The ~4.5 GB end-of-digestion plateau is the floor.

Completed/superseded snapshots live in `docs/archive/` (see
[`docs/archive/README.md`](docs/archive/README.md) — most recently, the
2026-07-02 consolidation archived the 2026-06 session logs, the BibTeX port
plan, the 2026-05-21 ambiguity audit + sandbox-triage workflow, the 3-sandbox
fatal analysis, and the startup-cost study). Single-paper reproducers /
out-of-scope cases live in `docs/reproducers/`, `docs/out-of-scope/`,
`docs/known_crashes/`.

