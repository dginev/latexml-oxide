# GitHub Issue Audit

> **Why this file exists.** GitHub issue access requires a working `gh`
> login plus (in the sandbox) network escalation, so offline agents
> routinely miss issue-tracker context. Worse, issue numbers collide with
> unrelated local numbering — e.g. the `#47` in [`WISDOM.md`](WISDOM.md) is
> **not** GitHub issue #47. This file mirrors the open issues with a local
> interpretation so planning does not depend on live tracker access.
>
> **Refresh** before milestone planning:
> `gh issue list --state open --limit 100 --json number,title,labels,createdAt`.
> Last refreshed: **2026-07-02** (8 open).

Tracker: <https://github.com/dginev/latexml-oxide/issues>

## Open issues (8)

| # | Title | Labels | Local status / interpretation |
|---|---|---|---|
| **47** | [Feature] Accurate latex linting | enhancement | **Prioritized beyond-Perl showcase.** Live source ↔ preview over a shared locator substrate, two clients: the **ar5iv-editor** (CodeMirror web UI) and a **VSCode extension** (webview). Accurate linting falls out of the same substrate. Design: [`SOURCE_PROVENANCE.md`](SOURCE_PROVENANCE.md). *Not* purely post-1.0 — Tier A is near-term and parity-neutral. |
| **92** | Superior debugging and error-reporting for document authors | enhancement | Same source-provenance substrate as #47 ([`SOURCE_PROVENANCE.md`](SOURCE_PROVENANCE.md)): construct-start + macro-origin locators give Rust-compiler-grade author errors, fixing TeX's "error points at the end of the environment". |
| **191** | Add support for original command-line options | enhancement | **PARTIAL — not closeable.** `clap` 4 derive is adopted (the issue's suggestion) and core options work, but coverage is **~47 flags vs ~95 in the Perl omni set** (`Common/Config.pm`). Audited 2026-05-24 — gaps below. |
| **143** | Switch to rust stable, when `#[thread_local]` is stabilized | enhancement, performance | Toolchain-longevity risk for a public-domain tool. Pin a known-good nightly; track stabilization. [`RELEASE_CRITERIA.md`](RELEASE_CRITERIA.md) §3. |
| **94** | Document model: RelaxNG vs Rust data-type trade-offs | enhancement, question, documentation | Doc debt; relates to the (closed) #199 HTML-dialect schema and [`SCHEMA_DOCUMENTATION.md`](SCHEMA_DOCUMENTATION.md). |
| **192** | Compile-time string interning? | enhancement, performance | Perf nice-to-have. The arena/interner is already the hottest read site (see [`SAFETY.md`](SAFETY.md) §B); the 2026-07-02 audit settled the related pin!/pin_static policy ([`PERFORMANCE.md`](PERFORMANCE.md) Principle 1). Measure before investing. Backlog. |
| **82** | Manually copy over perldoc as rustdoc | enhancement, help wanted, documentation | Doc debt. Long tail. |
| **80** | space XMhints as elided arguments | enhancement | **Open — still reproduces (verified 2026-06-16).** `$[D_{0},\ ]$` → the escaped space is dropped, so the grammar sees a dangling `,]` and rejects. Fix = emit an XMHint for the in-math space and teach the marpa grammar to treat it as an elided argument slot. Real grammar work, not a quick win. Backlog. |

## Recently closed (since the 2026-05-24 refresh — outcomes)

| # | Closed | Outcome |
|---|---|---|
| **101** | 2026-05-26 | Binary size accepted as structural (~47 MB maxperf; ~60k binding functions, no fat generic to shrink). [`RELEASE_CRITERIA.md`](RELEASE_CRITERIA.md) §2, [`PERFORMANCE.md`](PERFORMANCE.md) build-pipeline. |
| **217** | 2026-06-08 | macOS portable use RESOLVED — suite green on macos-15 arm64; kpathsea 0.3 subprocess fallback; text-node UAF fixed (WISDOM #58). [`archive/PORTABILITY_MACOS_PROBE_2026-06-07.md`](archive/PORTABILITY_MACOS_PROBE_2026-06-07.md). |
| **93** | 2026-06-09 | Declarative binding dialect → superseded by #247's runtime interpreter + shared `ConstructorBuilder` spine. [`BINDING_DSL_ARCHITECTURE.md`](BINDING_DSL_ARCHITECTURE.md). |
| **199** | 2026-06-12 | HTML-dialect RelaxNG schema — see [`SCHEMA_DOCUMENTATION.md`](SCHEMA_DOCUMENTATION.md). |
| **247** | 2026-06-12 | Runtime (Rhai) binding interpreter landed (`script-bindings` feature, off by default). [`script_bindings_plan.md`](script_bindings_plan.md). |
| **183** | 2026-06-16 | String-literal max-width style item. |
| **171** | 2026-06-16 | XML-replacement parser — resolved as a component of #247 (`ReplacementOp` AST + winnow, decision recorded in [`BINDING_DSL_ARCHITECTURE.md`](BINDING_DSL_ARCHITECTURE.md)). |
| **127** | 2026-06-18 | 64-bit numbers — fixed by exact `xn_over_d`-style fixed-point unit conversion (`numeric_ops::fixpoint_unit`), bit-exact vs pdftex. The storage was never the bug; f64 unit *conversion* was. |

## #191 — CLI option-coverage detail (audited 2026-05-24)

Our binary has ~47 `#[arg]` flags vs ~95 in Perl `Common/Config.pm` (the
`latexmlc` omni union). Existing Perl-name aliases: `--destination`,
`--noparse`, `--presentationmathml`, `--contentmathml`, `--xmath`.

- **Cheap parity gaps (map to existing/near features):** `--profile`
  (biggest — `fragment`/`math`/`article`/…; planned as **TOML** profiles
  deserialized into the clap option struct, not Perl `.opt` — design in
  [`OXIDIZED_DESIGN.md`](OXIDIZED_DESIGN.md) "Future Work"), `--strict`,
  `--includestyles`,
  `--validate`/`--novalidate`, `--mode`, `--debug`, `--navtoc` (alias),
  `--mathml`.
- **Feature gaps (option absent because the feature is):** `--mathimages` /
  `--mathsvg` / `--svg` (SVG deferred), `--jats` / `--html4` /
  `--tex` / `--box` output, `--crossref` / `--bibliography` / `--index` /
  `--permutedindex` / `--splitbibliography`, daemon mode (`--port`,
  `--expire`, `--cache_key`, `--address`, `--autoflush`, `--exist`,
  `--count`, `--name`).
- **Intentional non-goals:** `--output` (we keep `--destination` / `--dest`).

## Reading

* **Beyond-Perl showcase:** #47 + #92 — the ar5iv-editor + provenance
  substrate ([`SOURCE_PROVENANCE.md`](SOURCE_PROVENANCE.md)).
* **Release gates:** #143 (toolchain pin), plus the license + safety items in
  [`RELEASE_CRITERIA.md`](RELEASE_CRITERIA.md) that have no issue number.
* **Partial parity (open):** #191 — clap landed + core options, but ~47/95
  omni flags; `--profile` and a long tail still missing (detail above).
* **Backlog / exploratory:** #192, #94, #82, #80.
