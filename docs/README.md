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
| [`parity/BIB_ABSENCE_AUDIT_2026-07-29.md`](parity/BIB_ABSENCE_AUDIT_2026-07-29.md) | **R3** — every doc lacking `ltx_bibitem` across 2605/2606 sandboxes + full arXiv (52 299 corpus cases); cause families F1–F12, sprints S1–S10; complete lists in [`parity/bib_absence_2026-07-29/`](parity/bib_absence_2026-07-29/). |
| [`performance/BEYOND_PERL_LEVERS.md`](performance/BEYOND_PERL_LEVERS.md) | **R7** — BP-1…BP-6 levers from the 60k-doc telemetry; POST-RELEASE. |
| [`math/CONTENT_MATHML_GAPS.md`](math/CONTENT_MATHML_GAPS.md) | **R8** — content-MathML / math-parser gaps; deferred by user directive, do not pick off in isolation. |
| [`parity/DEFERRED_FAMILIES.md`](parity/DEFERRED_FAMILIES.md) | **R9** — parked deep families (`.bst`, xy-pic, mode-frame, …); several carry explicit "do NOT start". |
| [`release/RELEASE_CRITERIA.md`](release/RELEASE_CRITERIA.md) | The "what must be true before a public 1.0" contract: gates, binary-size budget, portability, license audit, tail-latency/RSS signals. |
| [`release/RELEASING.md`](release/RELEASING.md) | Tag-driven release procedure; the self-contained-binary requirement. |
| [`release/CRATES_IO_PUBLISH.md`](release/CRATES_IO_PUBLISH.md) | `cargo publish` + docs.rs + library-use story: bottom-up publish order, open blockers (workspace-`resources/` packaging, `pericortex` git dep), docs.rs metadata, `latexml::api` entrypoint. |
| [`release/LICENSE_INVENTORY.md`](release/LICENSE_INVENTORY.md) | Living license inventory for the redistributable binary (scopes the CC0 claim). |
| [`release/SAFETY.md`](release/SAFETY.md) | Threat model and `unsafe` inventory. |
| [`release/WINDOWS_COMPATIBILITY_PLAN.md`](release/WINDOWS_COMPATIBILITY_PLAN.md) | Living worklist for the Windows port (`windows-compatibility` branch): MSVC + vcpkg-static toolchain, phased plan to `cargo test --release` green on `windows-latest` CI and a zipped `.exe` artifact. |
| [`perfect_kernel/README.md`](perfect_kernel/README.md) | **Perfect-kernel mission** (branch `perfect_kernel`): raw-interpretation (`--preload=[rawstyles,rawclasses]latexml.sty`, no new bindings, no OmniBus) conversion of the ~2,400-manual TeX Live doc corpus; protocol + quality bars, with the living [ledger](perfect_kernel/LEDGER.md), [cluster worklist](perfect_kernel/CLUSTERS.md), [difficult-cases catalog](perfect_kernel/DIFFICULT_CASES.md) and the [Lua rebinding strategy](perfect_kernel/LUA_REBINDING.md). |
| [`AR5IV_DIAGNOSTICS.md`](AR5IV_DIAGNOSTICS.md) | The ar5iv issue-tracker sweep: every open "Improve article X" report screened against the current binary and classified vs same-host Perl, plus the ranked worklist. **Refresh before quoting any row** — a wrong main-file pick manufactures fake error counts. Re-measured 2026-07-20 on top of the 2026-07-18 snapshot. |

## 🎯 Target 1 — faithful Perl translation (`parity/`)

Strict parity at the dump/format boundary plus corpus-driven parity mining.

### Design & orientation
| Doc | What it is |
|-----|------------|
| [`parity/OXIDIZED_DESIGN.md`](parity/OXIDIZED_DESIGN.md) | Public-facing design **index + overview** (principles, architecture). Links the themed family below. |
| [`parity/OXIDIZED_DESIGN_DIVERGENCES.md`](parity/OXIDIZED_DESIGN_DIVERGENCES.md) | The numbered **intentional Perl divergences** that `.rs` comments cite as `OXIDIZED_DESIGN #N`. (`#N` numbers are load-bearing and kept verbatim; note the pre-existing collision between divergence `#7–#18` and the math cluster `#7–#18`.) |
| [`parity/OXIDIZED_DESIGN_TYPES.md`](parity/OXIDIZED_DESIGN_TYPES.md) | Type-system improvements + tactical pitfalls. |
| [`parity/OXIDIZED_DESIGN_FUTURE_WORK.md`](parity/OXIDIZED_DESIGN_FUTURE_WORK.md) | Future-work backlog. |
| [`parity/ORGANIZATION.md`](parity/ORGANIZATION.md) | Maps Perl engine files (`Engine/*.pool.ltxml`) → Rust (`latexml_engine/src/*.rs`); loading hierarchy. |
| [`parity/AUTHOR_MARKUP_PIPELINE.md`](parity/AUTHOR_MARKUP_PIPELINE.md) | **In-progress worklist** — unify the two-branch `\lx@add@authors` author/affiliation parser into one line-first pipeline; witness corpus + baseline + confirmed metadata-markup defects. |
| [`parity/ALGORITHM_RENDERING.md`](parity/ALGORITHM_RENDERING.md) | **In-progress worklist** — algorithm2e/algorithmicx golden-match: completed fixes (line numbering, ruled/boxed frames, `\fname@`), open follow-ups (inline `\Comment*[r]`, ruled caption-at-top, `\ref`-to-line counter, side-by-side minipages), and the cross-binding markup-unification plan. |

### Engine internals & known issues
| Doc | What it is |
|-----|------------|
| [`parity/WISDOM.md`](parity/WISDOM.md) | Tactical insights about system internals — check here to avoid re-introducing known bugs. |
| [`parity/CODE_REVIEW_2026-08-03.md`](parity/CODE_REVIEW_2026-08-03.md) | Frozen multi-lens review of the 2026-08 status/diagnostics/persistence campaign — findings + recommendations |
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
| [`performance/STREAMING_CORE_DESIGN_2026-07-29.md`](performance/STREAMING_CORE_DESIGN_2026-07-29.md) | Fragmented **core**-stage conversion so document size is bounded by disk, not RAM. Measured: ~1.84 GB RSS per MB of source, ~57 % of it the libxml2 DOM; a 131 MB witness needs ~241 GB. `TextReader::expand_to_document` as the partial-DOM substrate; the blocker is document-global labels in `DefRewrite`. |
| [`performance/STREAMING_POST_DESIGN_2026-07-06.md`](performance/STREAMING_POST_DESIGN_2026-07-06.md) | Very-large split-document post-processing (the 614 MB `index.xml` witness); two-pass streaming split design. |
| [`performance/MULTIDOC_JOIN.md`](performance/MULTIDOC_JOIN.md) | Joining a main paper + Supplementary-Material documents into one output. In-memory join LANDED (#639/#640); the streaming-scale post-join (reusing the two-pass Scan/ObjectDB split engine over N core-XML files) is designed + queued. |
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

*This page is the single doc index — keep it current when adding, renaming,
merging, or archiving a doc. The placement **rules** that govern what belongs
where live in `CLAUDE.md` ("Rules for these docs"), because they are policy
rather than navigation. Diagnostic-snapshot docs (`*_TRIAGE`, `*_AUDIT`,
`*_ANALYSIS`, …) carry a date in the filename; living worklists do not.*

