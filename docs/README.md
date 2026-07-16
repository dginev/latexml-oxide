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
| [`SYNC_STATUS.md`](SYNC_STATUS.md) | **The brief actionable worklist for both targets** — current status, in-flight session, open tasks, deferred families. Completed logs lift to `archive/`. |
| [`release/RELEASE_CRITERIA.md`](release/RELEASE_CRITERIA.md) | The "what must be true before a public 1.0" contract: gates, binary-size budget, portability, license audit, tail-latency/RSS signals. |
| [`release/RELEASING.md`](release/RELEASING.md) | Tag-driven release procedure; the self-contained-binary requirement. |
| [`release/CRATES_IO_PUBLISH.md`](release/CRATES_IO_PUBLISH.md) | `cargo publish` + docs.rs + library-use story: bottom-up publish order, open blockers (workspace-`resources/` packaging, `pericortex` git dep), docs.rs metadata, `latexml::api` entrypoint. |
| [`release/LICENSE_INVENTORY.md`](release/LICENSE_INVENTORY.md) | Living license inventory for the redistributable binary (scopes the CC0 claim). |
| [`release/ISSUE_AUDIT.md`](release/ISSUE_AUDIT.md) | Local mirror of open GitHub issues with status + interpretation. |
| [`release/SAFETY.md`](release/SAFETY.md) | Threat model and `unsafe` inventory. |

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
| [`parity/script_bindings_plan.md`](parity/script_bindings_plan.md) | The runtime (Rhai) `script-bindings` front-end reference (off by default). |

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

---

*Keep this index current when adding, renaming, merging, or archiving a doc.
`CLAUDE.md` at the repo root carries the same map inline (the authoritative
per-file index with the placement rules); this page is the navigational front
door. Diagnostic-snapshot docs (`*_TRIAGE`, `*_AUDIT`, `*_ANALYSIS`, …) carry a
date in the filename; living worklists do not.*
