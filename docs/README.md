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
| [`SYNC_STATUS.md`](SYNC_STATUS.md) | **The brief actionable worklist for both targets.** Opens with *How to read this file* + a **ranked worklist (R1…R9)** — take the top unblocked row. Then: current status, per-row detail, standing policies, parked-family pointers, stable reference. Completed logs lift to `archive/`. Labels here have gone stale before — **verify a status against the named guard test or `gh issue view` before acting on it; SHA-ancestry does not work, the repo squash-merges.** |
| [`parity/BIBLIOGRAPHY_WORKLIST.md`](parity/BIBLIOGRAPHY_WORKLIST.md) | **R5** — surveyed missing-references targets + the MakeBibliography full-parity re-port. |
| [`performance/BEYOND_PERL_LEVERS.md`](performance/BEYOND_PERL_LEVERS.md) | **R7** — BP-1…BP-6 levers from the 60k-doc telemetry; POST-RELEASE. |
| [`math/CONTENT_MATHML_GAPS.md`](math/CONTENT_MATHML_GAPS.md) | **R8** — content-MathML / math-parser gaps; deferred by user directive, do not pick off in isolation. |
| [`parity/DEFERRED_FAMILIES.md`](parity/DEFERRED_FAMILIES.md) | **R9** — parked deep families (`.bst`, xy-pic, mode-frame, …); several carry explicit "do NOT start". |
| [`release/RELEASE_CRITERIA.md`](release/RELEASE_CRITERIA.md) | The "what must be true before a public 1.0" contract: gates, binary-size budget, portability staging, license/public-domain audit, distribution safety profile, tail-latency/RSS signals, surpass-Perl policy. |
| [`release/RELEASING.md`](release/RELEASING.md) | Tag-driven release procedure; the self-contained-binary requirement. |
| [`release/CRATES_IO_PUBLISH.md`](release/CRATES_IO_PUBLISH.md) | `cargo publish` + docs.rs + library-use story: bottom-up publish order for the 8 crates, open blockers (workspace-`resources/` packaging **B3**, the `pericortex` git dep **B2**), docs.rs metadata, `latexml::api` entrypoint. Distinct from `RELEASING.md` (the GitHub-Release binary flow). |
| [`release/LICENSE_INVENTORY.md`](release/LICENSE_INVENTORY.md) | Living license inventory for the redistributable binary (the RELEASE_CRITERIA §4 deliverable): Rust deps (cargo-deny-gated), embedded assets, the TeX-Live-derived dumps position, linked syslibs, subprocess-only graphics tools. Scopes the CC0 claim. |
| [`release/ISSUE_AUDIT.md`](release/ISSUE_AUDIT.md) | Local mirror of open GitHub issues with status + interpretation; carries its own refresh stamp (do not duplicate the count elsewhere — it drifted twice). **Refresh before milestone planning.** Issue numbers are GitHub-tracker numbers — they do **not** correspond to any internal `#N` in `WISDOM.md`. |
| [`release/SAFETY.md`](release/SAFETY.md) | Threat model and `unsafe` inventory (distribution posture in `RELEASE_CRITERIA.md` §6). |
| [`release/WINDOWS_COMPATIBILITY_PLAN.md`](release/WINDOWS_COMPATIBILITY_PLAN.md) | Living worklist for the Windows port (`windows-compatibility` branch): MSVC + vcpkg-static toolchain, TeX Live + MiKTeX runtime, phased plan from compile blockers (libmarpa cc-port, libxml2/libxslt vcpkg) through `cargo test --release` green on `windows-latest` CI to a zipped `.exe` release artifact. Operationalizes RELEASE_CRITERIA portability rung 5. |
| [`AR5IV_DIAGNOSTICS.md`](AR5IV_DIAGNOSTICS.md) | The ar5iv issue-tracker sweep: every open "Improve article X" report screened against the current binary and classified vs same-host Perl, plus the ranked worklist. Re-measured 2026-07-20 on top of the 2026-07-18 snapshot. **Refresh before quoting any row** — a wrong main-file pick manufactures fake error counts (the file records the correct detector). |

## 🎯 Target 1 — faithful Perl translation (`parity/`)

Strict parity at the dump/format boundary plus corpus-driven parity mining.

### Design & orientation
| Doc | What it is |
|-----|------------|
| [`parity/OXIDIZED_DESIGN.md`](parity/OXIDIZED_DESIGN.md) | Public-facing design **index + overview** (principles, architecture). Links the themed family below. |
| [`parity/OXIDIZED_DESIGN_DIVERGENCES.md`](parity/OXIDIZED_DESIGN_DIVERGENCES.md) | The numbered **intentional Perl divergences** that `.rs` comments cite as `OXIDIZED_DESIGN #N`. Read here to check whether a translation difference was a marked intentional divergence. `#N` numbers are load-bearing and kept verbatim; note the pre-existing collision between divergence `#7–#18` and the math cluster `#7–#18` — `OXIDIZED_DESIGN.md`'s index explains which file owns each. |
| [`parity/OXIDIZED_DESIGN_TYPES.md`](parity/OXIDIZED_DESIGN_TYPES.md) | Type-system improvements + tactical pitfalls. |
| [`parity/OXIDIZED_DESIGN_FUTURE_WORK.md`](parity/OXIDIZED_DESIGN_FUTURE_WORK.md) | Future-work backlog. |
| [`parity/ORGANIZATION.md`](parity/ORGANIZATION.md) | Maps Perl engine files (`Engine/*.pool.ltxml`) → Rust (`latexml_engine/src/*.rs`); loading hierarchy. |

### Engine internals & known issues
| Doc | What it is |
|-----|------------|
| [`parity/WISDOM.md`](parity/WISDOM.md) | Tactical insights about system internals — check here to avoid re-introducing known bugs. |
| [`parity/KNOWN_PERL_ERRORS.md`](parity/KNOWN_PERL_ERRORS.md) | Upstream Perl LaTeXML issues; check first when investigating a test failure. When a shared bug is simple, fix in Rust and record it here (candidate to upstream). |
| [`parity/DUMP_DESIGN.md`](parity/DUMP_DESIGN.md) | Kernel dump precompilation (strict LoadFormat mutual exclusivity, unconditional apply) — the live architecture behind the per-TL-year release dumps. NOTE the format-layering nuance: the latex format sits on the REAL-plain.tex layer (Perl's is hand-curated), so plain-only macros can leak into latex sessions (the `\+` class, retracted at the `latex.rs` seam; audit in SYNC_STATUS 2026-07-02). |
| [`parity/BINDING_DSL_ARCHITECTURE.md`](parity/BINDING_DSL_ARCHITECTURE.md) | Binding-definition DSL: shared `ConstructorBuilder` spine, compile-time `macro_rules!` + runtime Rhai front-ends. Subsumes closed issues #93/#171. |
| [`parity/script_bindings_plan.md`](parity/script_bindings_plan.md) | The runtime (Rhai) script-bindings front-end reference (the `runtime-bindings` feature; on by default and in the distribution build — the old `script-bindings` alias was removed pre-publish). |

### Open dated diagnostics (`parity/diagnostics/`)
Point-in-time studies with pending halves.
| Doc | What it is |
|-----|------------|
| [`parity/diagnostics/EXPECTED_ID_XMREF_DESIGN_2026-06-08.md`](parity/diagnostics/EXPECTED_ID_XMREF_DESIGN_2026-06-08.md) | `expected:id` dangling-XMRef cluster: container-id half landed; MathFork reconciliation pending. |
| [`parity/diagnostics/EXPL3_CATCODE_GAP_2026-06-08.md`](parity/diagnostics/EXPL3_CATCODE_GAP_2026-06-08.md) | expl3 catcode-gap study — still OPEN; records four reverted attempts. |

## ➗ Math parser (`math/`) — serves both targets

The Marpa-style highly-ambiguous grammar that replaced Perl's Parse::RecDescent.

| Doc | What it is |
|-----|------------|
| [`math/MATH_PARSER_AND_ASF.md`](math/MATH_PARSER_AND_ASF.md) | **Canonical:** three-stage ambiguity pipeline vs the Marpa ASF traversal. Read before touching `parser.rs::parse_string` / `semantics.rs::Actions`. Companion to [`marpa/ASF_STATUS.md`](https://github.com/dginev/marpa/blob/asf-completion/ASF_STATUS.md). |
| [`math/MATH_PARSER_ASF_TIEBREAKING.md`](math/MATH_PARSER_ASF_TIEBREAKING.md) | ASF tie-breaking rules, in detail. |
| [`math/MATH_GRAMMAR_FIRST_PRINCIPLES.md`](math/MATH_GRAMMAR_FIRST_PRINCIPLES.md) | Design rationale for the Marpa grammar. |
| [`math/MATH_OVERPARSE_DEEP_DIVE_2026-06-30.md`](math/MATH_OVERPARSE_DEEP_DIVE_2026-06-30.md) | Measured and-node counts per ambiguity pattern; ranked open levers (`f(x)` apply-vs-multiply, bare-`\|x\|` pre-lexer, integral Step 2). The top `math_parse` lever for the arXiv runs; supersedes the archived 2026-05-21 ambiguity audit. |
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
| [`performance/STABILITY_WITNESSES.md`](performance/STABILITY_WITNESSES.md) | Living worklist of reliability witnesses (timeout/OOM/peak-RSS/hang) with current + Perl baselines. Distinct from `SYNC_STATUS.md` (correctness errors). |
| [`performance/STREAMING_POST_DESIGN_2026-07-06.md`](performance/STREAMING_POST_DESIGN_2026-07-06.md) | Very-large split-document post-processing: the correctness+foundation floor is landed (limit-safe queries so split fires on the 614 MB `index.xml`, stream-from-file, rust-libxml `TextReader`/checked-XPath); the **two-pass streaming split** to cut peak RSS 15.6 GB → <1 GB is the pending, parity-gated half. Resume point for that work (was `HANDOFF.md`). |
| [`performance/ISSUE_361_MEMORY_TIME_PROFILE_2026-07-24.md`](performance/ISSUE_361_MEMORY_TIME_PROFILE_2026-07-24.md) | Very-large **single** doc (#361, 232K-line/7.9 MB book, `--splitat=subsection`): RAM+time diagnosis (peak 9 GB = transient digested boxes + DOM coexisting at Building; flat CPU profile = allocator + libxml2 XPath). **M1+M2+M4 landed** (`List.font`→`Rc<Font>`; box `DigestedData::KeyVals`; box `Whatsit`'s two never-filled reversion-cache slots — 9.05 → **5.99 GB**, −34 %, unchanged wall time; `DigestedData` 424→128 B, guarded by `digested_data_size_budget`). Density is near its floor (`TBox`/`List` bound the enum at 104 B); the box-type census that settles "box the variant?" questions is in the doc. **M3 (stream boxes→DOM) is a SETTLED DEAD-END** — byte-identical, suite-green, but only −3–4 % (inside run variance); reverted, do not re-attempt in that shape. The ~4.5 GB end-of-digestion plateau is the floor. |
| [`performance/CORTEX_WORKER_HARNESS.md`](performance/CORTEX_WORKER_HARNESS.md) | `cortex_worker --harness` fleet orchestration: one-conversion-per-process, five-layer memory guards, crash-loop backoff, production deployment recommendation. Companion to pericortex `docs/HARNESS.md` and CorTeX `MANUAL.md` §7. |
| [`performance/TELEMETRY.md`](performance/TELEMETRY.md) | Per-job structured telemetry schema for `cortex_worker` runs. |
| [`performance/SOURCE_PROVENANCE.md`](performance/SOURCE_PROVENANCE.md) | Design for the source↔preview showcase over a shared locator substrate (ar5iv-editor + VSCode clients): accurate linting (#47), Rust-grade author errors (#92). Locators opt-in (`--source-map`). The landed-but-deprioritized `--server` LSP docs: [`archive/LSP_SERVER.md`](archive/LSP_SERVER.md), [`archive/LSP_MULTIFILE_PLAN.md`](archive/LSP_MULTIFILE_PLAN.md); smoke `tools/lsp_smoke.py`. |
| [`performance/SCHEMA_DOCUMENTATION.md`](performance/SCHEMA_DOCUMENTATION.md) | RelaxNG Compact schema → rustdoc-styled HTML doc site (supported the closed #199 HTML-dialect schema). |

## 📚 Reference collections (subdirectories, kept as-is)

| Directory | What it holds |
|-----------|---------------|
| [`archive/`](archive/README.md) | Completed/superseded snapshots and session logs (see its own `README.md`; most recently, the 2026-07-02 consolidation archived the 2026-06 session logs, the BibTeX port plan, the 2026-05-21 ambiguity audit + sandbox-triage workflow, the 3-sandbox fatal analysis, and the startup-cost study). |
| `reproducers/` | Single-paper reproducers for tracked bugs. |
| `out-of-scope/` | Cases intentionally out of scope (Perl also fails, no-DTD, …). |
| `known_crashes/` | Known crash records with triage. |
| `examples/` | Example bindings (e.g. `sample.sty.rhai`). |
| `scripts/` | One-off analysis helpers referenced by archived diagnostics (e.g. `bucket_callgrind_hot.py`). |

---

*This page is the **authoritative per-file index** — keep it current when
adding, renaming, merging, or archiving a doc. `CLAUDE.md` at the repo root
carries the layout summary and the doc-authoring rules (what goes where,
snapshot naming, conclusion-not-play-by-play). Diagnostic-snapshot docs
(`*_TRIAGE`, `*_AUDIT`, `*_ANALYSIS`, …) carry a date in the filename; living
worklists do not.*
