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
| [`SYNC_STATUS.md`](SYNC_STATUS.md) | **The brief actionable worklist for both targets.** Opens with *How to read this file* + a **ranked worklist (R, 0-0c, R8, KC, MS, … R7, by importance)** — take the top unblocked row; each row is a `###` section of one-fact bullets so edits merge line by line, and only the main session edits it (writers hand it a SYNC note). Then: a pointer to the silent-loss inventory (its own doc since 2026-11-09, `perfect_kernel/SILENT_LOSS_INVENTORY.md`), Rust Error Fixes (titles), current status, per-row detail, standing policies, parked-family pointers, stable reference. Landed work moves to the month's `archive/SYNC_SESSIONS_YYYY-MM.md`; the 2026-11-09 compaction's moved rows (run 336, the bundle rows, the long rows' landed history, the Rust Error Fixes' bodies) are in [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1). |
| [`perfect_kernel/MANUAL_REGRESSIONS.md`](perfect_kernel/MANUAL_REGRESSIONS.md) | **Living worklist (2026-11-06), the branch's primary goal:** every regression on the TeX Live manuals since sweep #149, with clusters, first errors and how to re-measure. All must be addressed before the merge. |
| [`perfect_kernel/OPEN_ROWS.md`](perfect_kernel/OPEN_ROWS.md) | Open-row detail behind the SYNC_STATUS worklist (moved 2026-10-23): corpus triage quick wins, perfect-kernel RED repros without a row elsewhere, fidelity audit round 2, the drained handoff register — and the open residuals lifted from the archived batch narratives. Read it when a ranked row points here, or before filing a residual. |
| [`perfect_kernel/SILENT_LOSS_INVENTORY.md`](perfect_kernel/SILENT_LOSS_INVENTORY.md) | **Living worklist, blocks the 98 % claim (user 2026-10-18):** every open loss of authored content or markup with 0 errors — the numbered items #5-#27, the frontmatter residual rows (the single engine's REDs, 108b, Z14, the int153a flip row, the pre-flip leftovers), the float and caption residuals; one `###` section per row, one fact per bullet. Moved from SYNC_STATUS 2026-11-09 (SYNC1b); landed rows in `archive/SYNC_SESSIONS_2026-11.md`. |
| [`parity/BIBLIOGRAPHY_WORKLIST.md`](parity/BIBLIOGRAPHY_WORKLIST.md) | Surveyed missing-references targets + the MakeBibliography full-parity re-port — done and superseded (kept in place: code comments cite it); open bibliography work is R3. |
| [`parity/BIB_ABSENCE_AUDIT_2026-07-29.md`](parity/BIB_ABSENCE_AUDIT_2026-07-29.md) | **R3** — every doc lacking `ltx_bibitem` across 2605/2606 sandboxes + full arXiv (52 299 corpus cases); cause families F1–F12, sprints S1–S10; complete lists in [`parity/bib_absence_2026-07-29/`](parity/bib_absence_2026-07-29/). |
| [`performance/BEYOND_PERL_LEVERS.md`](performance/BEYOND_PERL_LEVERS.md) | BP-1…BP-6 levers from the 60k-doc telemetry; POST-RELEASE. |
| [`math/MATH_PARSE_RESIDUALS.md`](math/MATH_PARSE_RESIDUALS.md) | Parked math-parse residual rows (the 57cj train and earlier), moved from SYNC_STATUS 2026-10-04; math fidelity is a later project. |
| [`math/CONTENT_MATHML_GAPS.md`](math/CONTENT_MATHML_GAPS.md) | Content-MathML / math-parser gaps; deferred by user directive, do not pick off in isolation. |
| [`parity/DEFERRED_FAMILIES.md`](parity/DEFERRED_FAMILIES.md) | Parked deep families (`.bst`, xy-pic, mode-frame, …); several carry explicit "do NOT start". |
| [`release/RELEASE_CRITERIA.md`](release/RELEASE_CRITERIA.md) | The "what must be true before a public 1.0" contract: gates, binary-size budget, portability, license audit, tail-latency/RSS signals. |
| [`release/RELEASING.md`](release/RELEASING.md) | Tag-driven release procedure; the self-contained-binary requirement. |
| [`release/CRATES_IO_PUBLISH.md`](release/CRATES_IO_PUBLISH.md) | `cargo publish` + docs.rs + library-use story: bottom-up publish order, open blockers (workspace-`resources/` packaging, `pericortex` git dep), docs.rs metadata, `latexml::api` entrypoint. |
| [`release/LICENSE_INVENTORY.md`](release/LICENSE_INVENTORY.md) | Living license inventory for the redistributable binary (scopes the CC0 claim). |
| [`release/SAFETY.md`](release/SAFETY.md) | Threat model and `unsafe` inventory. |
| [`release/WINDOWS_COMPATIBILITY_PLAN.md`](release/WINDOWS_COMPATIBILITY_PLAN.md) | Living worklist for the Windows port (`windows-compatibility` branch): MSVC + vcpkg-static toolchain, phased plan to `cargo test --release` green on `windows-latest` CI and a zipped `.exe` artifact. |
| [`release/WASM_COMPATIBILITY_AUDIT_2026-09-03.md`](release/WASM_COMPATIBILITY_AUDIT_2026-09-03.md) | **Stage 4 WASM Audit & Technical Analysis**: WebAssembly target analysis, C dependency audit (`libxml2`, `libxslt`, `marpa-asf`), subprocess mitigation, call stack bounds, and pure-Rust vs C trade-offs. |
| [`release/WASM_COMPATIBILITY_PLAN.md`](release/WASM_COMPATIBILITY_PLAN.md) | **Stage 4 WASM Phased Implementation Plan**: Actionable bring-up worklist for `wasm32-wasip1` via `wasi-sdk`, client-side in-browser execution via `@bjorn3/browser_wasi_shim`, and virtual asset packaging. |
| [`perfect_kernel/README.md`](perfect_kernel/README.md) | **Perfect-kernel mission** (goal bar met at sweeps #146-148; closing) (branch `perfect_kernel`, brief in [`PERFECT_KERNEL.md`](PERFECT_KERNEL.md)): raw-interpretation (`--preload=[rawstyles,rawclasses]latexml.sty`, no new bindings, no OmniBus) conversion of the ~2,400-manual TeX Live doc corpus; protocol + quality bars, with the living [ledger](perfect_kernel/LEDGER.md) (phases 56-58 in [`perfect_kernel/archive/`](perfect_kernel/archive/)), the goal bar and ranked path in [`PERFECT_KERNEL.md`](PERFECT_KERNEL.md#goal-bar-and-ranked-path-2026-10-02), [improvement-plans ledger](perfect_kernel/PLANS.md), [difficult-cases catalog](perfect_kernel/DIFFICULT_CASES.md), the [Lua rebinding strategy](perfect_kernel/LUA_REBINDING.md), the **[arXiv html_feedback subgoal](perfect_kernel/HTML_FEEDBACK.md)** (reader reports → problem classes verified on head → ranked rows HF1-HF13, user 2026-10-08), the [architecture-themes design brief](perfect_kernel/ARCHITECTURE_THEMES.md), the **[generalized kernel-capability program](perfect_kernel/KERNEL_CAPABILITIES.md)** (approved 2026-09-05: K1 provenance/overlay bindings, K2 nest vs save stack, K3 lthooks store, K4 templates/sockets, K5 raw-line reader, K6 font model, K7 file model, K8 runaway cap; K9–K18 added since: group codes, byte mouth, frontmatter stores, pTeX letters, binding conformance, typed parameters, the horizontal list, bibliography programs, the manual net, typed box sizes; K20–K24 added 2026-10-11 from the [capability-gaps study](perfect_kernel/KERNEL_CAPABILITY_GAPS_2026-10-11.md): gullet expansion fidelity, environments as TeX does them, binding provenance and ownership, the author-block event model, the audited A/B comparator). |
| [`THERMALS.md`](THERMALS.md) | **Read before any parallel workload on the dev laptop** (the laptop budget; work since 2026-09-29 runs on the cortex host, whose cores split 0-63 sweeps / 64-127 gates) (sweeps, `cargo nextest`, oracle/validate runs): the CPU/memory budget (20 threads, 31 GB + 8 GB swap), the 2026-09-02 throttling incident (sweep + suite at once), `JOBS`/`-j` ceilings alone vs. combined, and the commands to check package temperature, throttle events and swap. |

## 🎯 Target 1 — faithful Perl translation (`parity/`)

Strict parity at the dump/format boundary plus corpus-driven parity mining.

### Design & orientation
| Doc | What it is |
|-----|------------|
| [`parity/FIDELITY_CHARTER.md`](parity/FIDELITY_CHARTER.md) | **Start here for any conversion decision** — the user's rulings distilled into standing rules (priorities, content, numbering, frontmatter, bindings and copies, diagnostics, verification incl. reach-complete A/Bs); ask the user only where these conflict or are silent. |
| [`parity/OXIDIZED_DESIGN.md`](parity/OXIDIZED_DESIGN.md) | Public-facing design **index + overview** (principles, architecture). Links the themed family below. |
| [`parity/OXIDIZED_DESIGN_DIVERGENCES.md`](parity/OXIDIZED_DESIGN_DIVERGENCES.md) | The numbered **intentional Perl divergences** that `.rs` comments cite as `OXIDIZED_DESIGN #N`. (`#N` numbers are load-bearing and kept verbatim; note the pre-existing collision between divergence `#7–#18` and the math cluster `#7–#18`.) |
| [`parity/OXIDIZED_DESIGN_TYPES.md`](parity/OXIDIZED_DESIGN_TYPES.md) | Type-system improvements + tactical pitfalls. |
| [`parity/OXIDIZED_DESIGN_FUTURE_WORK.md`](parity/OXIDIZED_DESIGN_FUTURE_WORK.md) | Future-work backlog. |
| [`parity/ORGANIZATION.md`](parity/ORGANIZATION.md) | Maps Perl engine files (`Engine/*.pool.ltxml`) → Rust (`latexml_engine/src/*.rs`); loading hierarchy. |
| [`parity/FRONTMATTER_SINGLE_ENGINE_PLAN.md`](parity/FRONTMATTER_SINGLE_ENGINE_PLAN.md) | **The one living frontmatter doc (2026-11-01; the digested refactor merged in 2026-11-08; the user's frontmatter priority)** — the principle (every segmentation and ownership decision reads the observer's events or the digested XML, never TeX token or source spelling; tokens carry, digestion decides; what digestion cannot settle goes to a structural fail-safe), "The path" (Route D: the emitter (the default since the flip, 2026-11-09; the switch and the old driver deleted, DEL2), the safety net, strict porting from the rule tables, knowledge in and driver out, the flip Z17, the deletions Z18-Z19 (Z19's S2b writer and the `LATEXML_FM_SHADOW` census deleted in DEL4), where it stands and a mechanical definition of done), the event model (what exists, gaps 1-10), what a rollback leaves (the rehearsal's isolation census and read budget), the test suite, the diagnosis, the target model with its placement order (labels and marks → spelling → declared convention → block), the structure model (cut, kind, group, link, emit) and the rulings R1-R17. Read before any frontmatter batch, reader, binding splitter or label key. |
| [`parity/FRONTMATTER_REDS_AT_EMIT2.tsv`](parity/FRONTMATTER_REDS_AT_EMIT2.tsv) | The guards skipped at `LATEXML_FM_EMIT=2` (the switch since deleted, DEL2) as named REDs before the flip (test, what `=2` gets wrong, and DEL1's outcome: the guard deleted, or kept with its `=0` tail cut); the record of known `=2` deficits — SYNC leftovers item (37). |
| [`parity/FRONTMATTER_CONTENT_RESIDUAL_AT_EMIT2.tsv`](parity/FRONTMATTER_CONTENT_RESIDUAL_AT_EMIT2.tsv) | The content `=2` still loses against the switch off at the flip, per paper (the dev sets and the A/B population, `kit/emit/docwords.py`): flipped with as named REDs (user 2026-11-09), fixed after the flip — SYNC leftovers item (39), OD #981. |
| [`parity/ALGORITHM_RENDERING.md`](parity/ALGORITHM_RENDERING.md) | **In-progress worklist** — algorithm2e/algorithmicx golden-match: completed fixes (line numbering, ruled/boxed frames, `\fname@`), open follow-ups (inline `\Comment*[r]`, ruled caption-at-top, `\ref`-to-line counter, side-by-side minipages), and the cross-binding markup-unification plan. |
| [`parity/ENV_MARKUP_DESIGN.md`](parity/ENV_MARKUP_DESIGN.md) | **Phase 2 Design Brief** — `ltx_env_<name>` environment wrapper markup class for responsive CSS styling; implementation strategy and golden test plan. |

### Engine internals & known issues
| Doc | What it is |
|-----|------------|
| [`parity/WISDOM.md`](parity/WISDOM.md) | Tactical insights about system internals — check here to avoid re-introducing known bugs. |
| [`parity/KNOWN_PERL_ERRORS.md`](parity/KNOWN_PERL_ERRORS.md) | Upstream Perl LaTeXML issues; check first when investigating a test failure. |
| [`parity/DUMP_DESIGN.md`](parity/DUMP_DESIGN.md) | Kernel dump precompilation (strict LoadFormat mutual exclusivity, unconditional apply). |
| [`parity/BINDING_DSL_ARCHITECTURE.md`](parity/BINDING_DSL_ARCHITECTURE.md) | Binding-definition DSL: shared `ConstructorBuilder` spine, compile-time + runtime front-ends. |
| [`parity/script_bindings_plan.md`](parity/script_bindings_plan.md) | The runtime (Rhai) script-bindings front-end reference (the `runtime-bindings` feature; on by default). |

## ➗ Math parser (`math/`) — serves both targets

The Marpa-style highly-ambiguous grammar that replaced Perl's Parse::RecDescent.

| Doc | What it is |
|-----|------------|
| [`math/MATH_PARSER_AND_ASF.md`](math/MATH_PARSER_AND_ASF.md) | **Canonical:** three-stage ambiguity pipeline vs the Marpa ASF traversal. Read before touching `parser.rs::parse_string` / `semantics.rs::Actions`. |
| [`math/MATH_PARSER_ASF_TIEBREAKING.md`](math/MATH_PARSER_ASF_TIEBREAKING.md) | ASF tie-breaking rules, in detail. |
| [`math/MATH_GRAMMAR_FIRST_PRINCIPLES.md`](math/MATH_GRAMMAR_FIRST_PRINCIPLES.md) | Design rationale for the Marpa grammar. |
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
| [`performance/PERFORMANCE_AUDIT_2026-09-03.md`](performance/PERFORMANCE_AUDIT_2026-09-03.md) | Read-only performance and memory audit: ranked algorithm/allocation findings, source anchors, implementation boundaries, validation gates, documentation reconciliation, and handoff order. |
| [`performance/WINDOWS_PERFORMANCE.md`](performance/WINDOWS_PERFORMANCE.md) | Windows performance profile (2026-08-21): the kpathsea backend comparison (subprocess vs in-process linked-libkpathsea 16× speedup). |
| [`performance/STABILITY_WITNESSES.md`](performance/STABILITY_WITNESSES.md) | Living worklist of reliability witnesses (timeout/OOM/peak-RSS/hang) with current + Perl baselines. |
| [`performance/MULTIDOC_JOIN.md`](performance/MULTIDOC_JOIN.md) | Joining a main paper + Supplementary-Material documents into one output. In-memory join LANDED (#639/#640); the streaming-scale post-join (reusing the two-pass Scan/ObjectDB split engine over N core-XML files) is designed + queued. |
| [`performance/CORTEX_WORKER_HARNESS.md`](performance/CORTEX_WORKER_HARNESS.md) | `cortex_worker --harness` fleet orchestration: one-conversion-per-process, memory guards, deployment, and the host TeX Live apt list arXiv needs (not `texlive-full`). |
| [`performance/TELEMETRY.md`](performance/TELEMETRY.md) | Per-job structured telemetry schema for `cortex_worker` runs. |
| [`performance/SOURCE_PROVENANCE.md`](performance/SOURCE_PROVENANCE.md) | Design for the source↔preview showcase over a shared locator substrate (issues #47/#92). |
| [`performance/SCHEMA_DOCUMENTATION.md`](performance/SCHEMA_DOCUMENTATION.md) | RelaxNG Compact schema → rustdoc-styled HTML doc site. |

## 📚 Reference collections (subdirectories, kept as-is)

| Directory | What it holds |
|-----------|---------------|
| [`archive/`](archive/README.md) | Completed/superseded snapshots and session logs (see its own `README.md`). |
| `reproducers/` | Single-paper reproducers for tracked bugs. |
| `out-of-scope/` | Cases intentionally out of scope (Perl also fails, no-DTD, …). |
| `known_crashes/` | Crash reproducers and the intake index; resolved write-ups in `archive/known_crashes/`. |
| `examples/` | Example bindings (e.g. `sample.sty.rhai`). |
| `scripts/` | One-off analysis helpers referenced by archived diagnostics (e.g. `bucket_callgrind_hot.py`). |

---

*This page is the single doc index — keep it current when adding, renaming,
merging, or archiving a doc. The placement **rules** that govern what belongs
where live in `CLAUDE.md` ("Rules for these docs"), because they are policy
rather than navigation. Diagnostic-snapshot docs (`*_TRIAGE`, `*_AUDIT`,
`*_ANALYSIS`, …) carry a date in the filename; living worklists do not.*
