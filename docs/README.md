# latexml-oxide documentation

The index of the project's docs, grouped by theme. Users start with the first table; contributors resuming work start
with [`SYNC_STATUS.md`](SYNC_STATUS.md).

> **What this project is.** latexml-oxide is a faithful Perl→Rust translation of
> [LaTeXML](https://github.com/brucemiller/latexml). Two co-equal targets drive the work: **(1) faithful parity** with
> the original Perl (the Perl source is ground truth), and **(2) beyond-Perl improvements** over the ~3M-document
> arXiv corpus (the levers Rust affords that single-threaded, libxslt-bound Perl cannot). The themes below mirror that
> split.

---

## 👤 Using and building latexml-oxide

The guides behind the root [`README.md`](../README.md), which is the short entry point.

| Doc | What it is |
|-----|------------|
| [`INSTALL.md`](INSTALL.md) | Installing: prebuilt binaries per platform, the one-time format build, Docker, the system dependencies, `cargo install`, use as a library. |
| [`USAGE.md`](USAGE.md) | The two binaries, output formats (HTML5, XHTML, XML, EPUB 3), stylesheets, raw package loading and its limits, split runs, your own `.rhai` bindings. |
| [`ENVIRONMENT.md`](ENVIRONMENT.md) | Every environment variable read or set: values, defaults, effects, audience. Also the rustdoc page `latexml::environment` (a copy in `latexml_oxide/src/`, guarded by `cluster_cli::environment_doc`). |
| [`INTERMEDIATE_FILES.md`](INTERMEDIATE_FILES.md) | Every file a conversion writes besides its output: caches, scratch files, locks — where each lives and who removes it. Also the rustdoc page `latexml::intermediate_files` (guarded by `cluster_cli::intermediate_files_doc`). |
| [`BUILDING.md`](BUILDING.md) | Building from source per platform, the format dumps, build profiles, development tips (hooks, test requirements and skips), Cargo features. |
| [`SUPPORTED_PACKAGES.md`](SUPPORTED_PACKAGES.md) | Generated: every package and class binding (ported from Perl, beyond Perl, contributed, raw TeX with overrides) and the rule for the rest. |
| [Rust API (rustdoc)](https://dginev.github.io/latexml-oxide/latexml/) | The crates' API documentation, including the Rhai binding interface and the Perl \| Rust \| Rhai cheat sheet. |

## 🧭 Worklists and contracts

The live worklists and the ship contract.

| Doc | What it is |
|-----|------------|
| [`GOAL_v_0_8_1.md`](GOAL_v_0_8_1.md) | The current goal: the planned tasks of SYNC_STATUS on `int/towards_v0_8_1`, then the open issues and PRs; the two release timelines, the order of work, what "solved" means and the bar for merging. |
| [`SYNC_STATUS.md`](SYNC_STATUS.md) | **The actionable worklist for both targets**, the single list of planned work (one line per task, the Bruce Miller Q&A rows QA01-QA93 included), then Rust error fixes, current status, standing policies and stable reference. Only the main session edits it. Closed history moves to the month's [`archive/SYNC_SESSIONS_YYYY-MM.md`](archive/SYNC_SESSIONS_2026-11.md). |
| [`perfect_kernel/MANUAL_REGRESSIONS.md`](perfect_kernel/MANUAL_REGRESSIONS.md) | Every regression on the TeX Live manuals since sweep #149, with clusters, first errors and how to re-measure; goal met, residual rows in SYNC_STATUS. |
| [`perfect_kernel/OPEN_ROWS.md`](perfect_kernel/OPEN_ROWS.md) | The detail behind SYNC_STATUS rows: corpus triage quick wins, RED repros without a row elsewhere, the fidelity audit, open residuals from archived batches. Read it when a row points here, or before filing a residual. |
| [`perfect_kernel/SILENT_LOSS_INVENTORY.md`](perfect_kernel/SILENT_LOSS_INVENTORY.md) | **Living worklist:** every open loss of authored content or markup with 0 errors (numbered items, frontmatter residuals, float and caption residuals), one section per row. |
| [`parity/BIBLIOGRAPHY_WORKLIST.md`](parity/BIBLIOGRAPHY_WORKLIST.md) | Surveyed missing-references targets and the MakeBibliography re-port — done and superseded; kept because code comments cite it. |
| [`parity/BIB_ABSENCE_AUDIT_2026-07-29.md`](parity/BIB_ABSENCE_AUDIT_2026-07-29.md) | Every document lacking `ltx_bibitem` across the sandboxes and full arXiv (52,299 cases): cause families F1–F12, sprints S1–S10; full lists in [`parity/bib_absence_2026-07-29/`](parity/bib_absence_2026-07-29/). |
| [`performance/BEYOND_PERL_LEVERS.md`](performance/BEYOND_PERL_LEVERS.md) | Levers BP-1…BP-6 from the 60k-document telemetry; post-release. |
| [`math/MATH_PARSE_RESIDUALS.md`](math/MATH_PARSE_RESIDUALS.md) | Parked math-parse residual rows; math fidelity is a later project. |
| [`math/CONTENT_MATHML_GAPS.md`](math/CONTENT_MATHML_GAPS.md) | Content MathML and math-parser gaps; deferred, not to be picked off in isolation. |
| [`parity/DEFERRED_FAMILIES.md`](parity/DEFERRED_FAMILIES.md) | Parked deep families (`.bst`, xy-pic, mode-frame, …); several say "do NOT start". |
| [`release/RELEASE_CRITERIA.md`](release/RELEASE_CRITERIA.md) | What must be true before a public 1.0: gates, binary-size budget, portability, license audit, tail-latency and RSS signals. |
| [`release/RELEASING.md`](release/RELEASING.md) | The tag-driven release procedure and the self-contained-binary requirement. |
| [`release/CHANGES_v0.8.0.md`](release/CHANGES_v0.8.0.md) | Every change in 0.8.0 with its guards and design references, and the 199 new bindings; [`CHANGELOG.md`](../CHANGELOG.md) holds the release summary. |
| [`release/CRATES_IO_PUBLISH.md`](release/CRATES_IO_PUBLISH.md) | Publishing to crates.io and docs.rs: the bottom-up publish order, the per-release checklist, the durable traps (10 MiB cap, never an `-rc`), the `latexml::api` entry point. |
| [`release/CODE_SIGNING_POLICY.md`](release/CODE_SIGNING_POLICY.md) | Windows code signing: status (unsigned), the Microsoft Store plan, roles, privacy, system changes (issue 775). |
| [`release/LICENSE_INVENTORY.md`](release/LICENSE_INVENTORY.md) | The license inventory of the redistributable binary (it scopes the CC0 claim); the notices themselves are [`THIRD-PARTY-NOTICES`](../THIRD-PARTY-NOTICES). |
| [`release/SAFETY.md`](release/SAFETY.md) | The threat model and the `unsafe` inventory. |
| [`release/WINDOWS_COMPATIBILITY_PLAN.md`](release/WINDOWS_COMPATIBILITY_PLAN.md) | The Windows port (shipped): the MSVC + vcpkg-static toolchain, the bring-up summary and the remaining worklist. |
| [`release/WINDOWS_SUITE_TRIAGE_2026-11-16.md`](release/WINDOWS_SUITE_TRIAGE_2026-11-16.md) | The Windows full suite on main `f7e359fe7e1`: 32 failures in 5 classes, each with its evidence, code site and fix plan. |
| [`release/WASM_COMPATIBILITY_AUDIT_2026-09-03.md`](release/WASM_COMPATIBILITY_AUDIT_2026-09-03.md) | The WebAssembly target analysis: C dependencies (`libxml2`, `libxslt`, `marpa-asf`), subprocesses, call-stack bounds, pure-Rust trade-offs. |
| [`release/WASM_COMPATIBILITY_PLAN.md`](release/WASM_COMPATIBILITY_PLAN.md) | The phased WASM plan: `wasm32-wasip1` via `wasi-sdk`, in-browser execution via `@bjorn3/browser_wasi_shim`, virtual asset packaging. |
| [`perfect_kernel/README.md`](perfect_kernel/README.md) | **The perfect-kernel mission (complete):** raw interpretation of the ~2,400 TeX Live manuals (`--preload=[rawstyles,rawclasses]latexml.sty`); the brief, goal bar and close status are in [`archive/PERFECT_KERNEL_2026-11-13.md`](archive/PERFECT_KERNEL_2026-11-13.md) ([goal bar and ranked path](archive/PERFECT_KERNEL_2026-11-13.md#goal-bar-and-ranked-path-2026-10-02)). Its documents: the [ledger](perfect_kernel/LEDGER.md) (older phases in [`perfect_kernel/archive/`](perfect_kernel/archive/)), the [improvement plans](perfect_kernel/PLANS.md), the [difficult cases](perfect_kernel/DIFFICULT_CASES.md), the [Lua rebinding strategy](perfect_kernel/LUA_REBINDING.md), the [arXiv html_feedback subgoal](perfect_kernel/HTML_FEEDBACK.md) (reader reports → problem classes → rows HF1-HF13), the [architecture themes](perfect_kernel/ARCHITECTURE_THEMES.md), and the [kernel-capability program](perfect_kernel/KERNEL_CAPABILITIES.md) (K1–K24, from provenance bindings to the audited A/B comparator; the [capability-gaps study](perfect_kernel/KERNEL_CAPABILITY_GAPS_2026-10-11.md) added K20–K24). |
| [`THERMALS.md`](THERMALS.md) | **Read before any parallel workload on the dev laptop** (sweeps, `cargo nextest`, oracle runs): the CPU and memory budget, the throttling incident of running a sweep and the suite at once, `JOBS`/`-j` ceilings, and the commands that check temperature, throttling and swap. |

## 🎯 Target 1 — faithful Perl translation (`parity/`)

Strict parity at the dump/format boundary, plus corpus-driven parity mining.

### Design and orientation
| Doc | What it is |
|-----|------------|
| [`parity/FIDELITY_CHARTER.md`](parity/FIDELITY_CHARTER.md) | **Start here for any conversion decision:** the user's rulings as standing rules (priorities, content, numbering, frontmatter, bindings and copies, diagnostics, verification). |
| [`parity/OXIDIZED_DESIGN.md`](parity/OXIDIZED_DESIGN.md) | The design index and overview (principles, architecture); links the family below. |
| [`parity/OXIDIZED_DESIGN_DIVERGENCES.md`](parity/OXIDIZED_DESIGN_DIVERGENCES.md) | The numbered **intentional divergences from Perl** that code comments cite as `OXIDIZED_DESIGN #N`. The numbers are load-bearing (divergences `#7–#18` collide with the math cluster's `#7–#18`); the admission gate is in its header, and `tools/perfect_kernel/od_anchors.sh` checks every cited number resolves. The frontmatter pipeline is eleven primary entries, their former members frozen in [`archive/OD_FRONTMATTER_RECORDS_2026-11-15.md`](archive/OD_FRONTMATTER_RECORDS_2026-11-15.md). Ends with the catalog of bindings beyond Perl's tree. |
| [`parity/OXIDIZED_DESIGN_TYPES.md`](parity/OXIDIZED_DESIGN_TYPES.md) | Type-system improvements and tactical pitfalls. |
| [`parity/OXIDIZED_DESIGN_FUTURE_WORK.md`](parity/OXIDIZED_DESIGN_FUTURE_WORK.md) | The future-work backlog. |
| [`parity/ORGANIZATION.md`](parity/ORGANIZATION.md) | **The Rust layout for Perl readers:** every crate, and each Perl module or file → its Rust crate and file (`Core`, `Common`, `Util`, `Package.pm`, `Package/*.ltxml`, the dump, `Engine/*.pool.ltxml` and its loading hierarchy, the math parser, `Post`, the CLI). |
| [`parity/SCHEMA_EXTENSIONS.md`](parity/SCHEMA_EXTENSIONS.md) | Every difference between our RelaxNG and Perl's (12 widenings, no new element), each with its OD entry, emitters and the rule an upstream patch would state. |
| [`parity/FRONTMATTER_SINGLE_ENGINE_PLAN.md`](parity/FRONTMATTER_SINGLE_ENGINE_PLAN.md) | **The one living frontmatter doc:** the principle (decide on digested events and XML, never on source spelling), the path to the single emitter, the event model, the test suite, the target and structure models, rulings R1-R17. Read before any frontmatter change. |
| [`parity/perl_bindings.txt`](parity/perl_bindings.txt) | Generated snapshot of Perl LaTeXML's binding names, which `SUPPORTED_PACKAGES.md`'s "ported" reads; refresh it with each upstream sync (the command is in its header). |
| [`parity/FRONTMATTER_REDS_AT_EMIT2.tsv`](parity/FRONTMATTER_REDS_AT_EMIT2.tsv) | The guards that were named REDs when the frontmatter emitter became the default (`LATEXML_FM_EMIT=2`, a switch since deleted): what each got wrong, and its outcome. |
| [`parity/FRONTMATTER_CONTENT_RESIDUAL_AT_EMIT2.tsv`](parity/FRONTMATTER_CONTENT_RESIDUAL_AT_EMIT2.tsv) | The content the frontmatter emitter still lost at that switch, per paper; fixed after it (OD #981). |
| [`parity/ALGORITHM_RENDERING.md`](parity/ALGORITHM_RENDERING.md) | **In-progress worklist:** algorithm2e/algorithmicx golden matches, completed fixes, open follow-ups and the markup-unification plan. |
| [`parity/ENV_MARKUP_DESIGN.md`](parity/ENV_MARKUP_DESIGN.md) | Design brief: the `ltx_env_<name>` environment class for responsive CSS, and its test plan. |

### Engine internals and known issues
| Doc | What it is |
|-----|------------|
| [`parity/WISDOM.md`](parity/WISDOM.md) | Tactical insights about system internals — check here to avoid re-introducing known bugs. |
| [`parity/KNOWN_PERL_ERRORS.md`](parity/KNOWN_PERL_ERRORS.md) | Upstream Perl LaTeXML issues; check first when investigating a test failure. |
| [`parity/UPSTREAM_SYNC_2833_to_0db6f47e.md`](parity/UPSTREAM_SYNC_2833_to_0db6f47e.md) | The upstream LaTeXML commits after #2833 through `0db6f47e`: each one's state here (ported or superseded) and its guard. |
| [`parity/DUMP_DESIGN.md`](parity/DUMP_DESIGN.md) | The kernel format dumps (strict LoadFormat mutual exclusivity, unconditional apply). |
| [`parity/BINDING_DSL_ARCHITECTURE.md`](parity/BINDING_DSL_ARCHITECTURE.md) | The binding-definition DSL: the shared `ConstructorBuilder` spine, its compile-time and runtime front-ends. |
| [`parity/RHAI_BINDINGS_GUIDE.md`](parity/RHAI_BINDINGS_GUIDE.md) | **The newcomer guide to Rhai bindings:** loading, values, the Perl closure form, truthiness, diagnostics, migrating from the deprecated names. |
| [Rhai API reference](https://dginev.github.io/latexml-oxide/latexml_contrib/script_bindings/interface/index.html) and [cheat sheet](https://dginev.github.io/latexml-oxide/latexml_contrib/script_bindings/cheatsheet/index.html) | The generated rustdoc pages: every registered Rhai function (from `latexml_contrib/src/script_bindings/API.md`, gated by `api_reference_is_up_to_date`) and the Perl \| Rust \| Rhai cheat sheet (`CHEATSHEET.md`). |
| [`parity/script_bindings_plan.md`](parity/script_bindings_plan.md) | The design of runtime (Rhai) bindings (the default `runtime-bindings` feature): why Rhai, the seam, discovery, sandbox, failure containment, shared lowering. |

## ➗ Math parser (`math/`) — serves both targets

The highly ambiguous Marpa grammar that replaced Perl's Parse::RecDescent.

| Doc | What it is |
|-----|------------|
| [`math/MATH_PARSER_AND_ASF.md`](math/MATH_PARSER_AND_ASF.md) | **Canonical:** the three-stage ambiguity pipeline and the Marpa ASF traversal. Read before touching `parser.rs::parse_string` or `semantics.rs::Actions`. |
| [`math/MATH_PARSER_ASF_TIEBREAKING.md`](math/MATH_PARSER_ASF_TIEBREAKING.md) | The ASF tie-breaking rules in detail. |
| [`math/MATH_GRAMMAR_FIRST_PRINCIPLES.md`](math/MATH_GRAMMAR_FIRST_PRINCIPLES.md) | The design rationale of the Marpa grammar. |
| [`math/OXIDIZED_DESIGN_MATH.md`](math/OXIDIZED_DESIGN_MATH.md) | The math parser's design and its divergences (`#19`–`#62`, each "formerly OD #N"). |

## 🚀 Target 2 — beyond Perl (`performance/`)

Performance and reliability over the arXiv corpus, the fleet and telemetry behind it, and features Perl has no
equivalent of (source provenance, the schema documentation site).

| Doc | What it is |
|-----|------------|
| [`performance/ARXIV_PERFORMANCE.md`](performance/ARXIV_PERFORMANCE.md) | The living performance campaign over arXiv: the slowest-100 testbed, phase rollups, the optimization log. |
| [`performance/PERFORMANCE.md`](performance/PERFORMANCE.md) | Optimization principles, the state of each lever, the audit log. |
| [`performance/PERFORMANCE_AUDIT_2026-09-03.md`](performance/PERFORMANCE_AUDIT_2026-09-03.md) | A performance and memory audit: ranked findings with source anchors, validation gates, handoff order. |
| [`performance/POST_V080_PERFORMANCE_PLAN.md`](performance/POST_V080_PERFORMANCE_PLAN.md) | The post-0.8.0 performance roadmap from profiling large split books and arXiv papers: a lazy SQLite `ObjectDB`, formula memoization, `--navigationtoc=context` caching, parallel math, `CrossRef` consolidation. |
| [`performance/WINDOWS_PERFORMANCE.md`](performance/WINDOWS_PERFORMANCE.md) | The Windows performance profile: in-process libkpathsea is 16× faster than the `kpsewhich` subprocess. |
| [`performance/STABILITY_WITNESSES.md`](performance/STABILITY_WITNESSES.md) | Living worklist of reliability witnesses (timeout, OOM, peak RSS, hang) with current and Perl baselines. |
| [`performance/MULTIDOC_JOIN.md`](performance/MULTIDOC_JOIN.md) | Joining a main paper and its supplementary documents into one output: the in-memory join (landed) and the streaming post-join (designed). |
| [`performance/CORTEX_WORKER_HARNESS.md`](performance/CORTEX_WORKER_HARNESS.md) | `cortex_worker --harness`: one conversion per process, memory guards, deployment, and the TeX Live apt list arXiv needs. |
| [`performance/TELEMETRY.md`](performance/TELEMETRY.md) | The per-job telemetry schema of `cortex_worker` runs. |
| [`performance/SOURCE_PROVENANCE.md`](performance/SOURCE_PROVENANCE.md) | The source↔preview design over a shared locator substrate (issues #47/#92). |
| [`performance/SCHEMA_DOCUMENTATION.md`](performance/SCHEMA_DOCUMENTATION.md) | The RelaxNG Compact schema rendered as a rustdoc-styled HTML site. |

## 📚 Reference collections

| Directory | What it holds |
|-----------|---------------|
| [`archive/`](archive/README.md) | Completed and superseded snapshots and session logs (see its own `README.md`). |
| [`reproducers/`](reproducers/) | Single-paper reproducers for tracked bugs. |
| [`out-of-scope/`](out-of-scope/README.md) | Cases out of scope on purpose (Perl also fails, no DTD, …). |
| [`known_crashes/`](known_crashes/README.md) | Crash reproducers and their intake index; resolved write-ups in [`archive/known_crashes/`](archive/known_crashes/). |
| [`examples/`](examples/) | Worked Rhai bindings with their documents, converted in CI (`30_script_bindings::docs_examples_convert_clean`). |
| [`scripts/`](scripts/) | Analysis helpers referenced by archived diagnostics (e.g. `bucket_callgrind_hot.py`). |

---

*Keep this index current when adding, renaming, merging or archiving a doc. The rules for what belongs where are
in `CLAUDE.md` ("Rules for these docs"). Diagnostic snapshots (`*_TRIAGE`, `*_AUDIT`, `*_ANALYSIS`, …) carry a date
in the filename; living worklists do not.*
