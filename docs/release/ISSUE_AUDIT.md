# GitHub Issue Audit

> **Why this file exists.** GitHub issue access requires a working `gh`
> login plus (in the sandbox) network escalation, so offline agents
> routinely miss issue-tracker context. Worse, issue numbers collide with
> unrelated local numbering — e.g. the `#47` in [`WISDOM.md`](../parity/WISDOM.md) is
> **not** GitHub issue #47. This file mirrors the open issues with a local
> interpretation so planning does not depend on live tracker access.
>
> **Refresh** before milestone planning:
> `gh issue list --state open --limit 100 --json number,title,labels,createdAt`.
> Last refreshed: **2026-07-09** (7 open; #191 closed via #276).

Tracker: <https://github.com/dginev/latexml-oxide/issues>

## Open issues (7)

| # | Title | Labels | Local status / interpretation |
|---|---|---|---|
| **47** | [Feature] Accurate latex linting | enhancement | **Prioritized beyond-Perl showcase.** Live source ↔ preview over a shared locator substrate, two clients: the **ar5iv-editor** (CodeMirror web UI) and a **VSCode extension** (webview). Accurate linting falls out of the same substrate. Design: [`SOURCE_PROVENANCE.md`](../performance/SOURCE_PROVENANCE.md). *Not* purely post-1.0 — Tier A is near-term and parity-neutral. |
| **92** | Superior debugging and error-reporting for document authors | enhancement | Same source-provenance substrate as #47 ([`SOURCE_PROVENANCE.md`](../performance/SOURCE_PROVENANCE.md)): construct-start + macro-origin locators give Rust-compiler-grade author errors, fixing TeX's "error points at the end of the environment". |
| **143** | Switch to rust stable, when `#[thread_local]` is stabilized | enhancement, performance | Toolchain-longevity risk for a public-domain tool. Pin a known-good nightly; track stabilization. [`RELEASE_CRITERIA.md`](RELEASE_CRITERIA.md) §3. |
| **94** | Document model: RelaxNG vs Rust data-type trade-offs | enhancement, question, documentation | Doc debt; relates to the (closed) #199 HTML-dialect schema and [`SCHEMA_DOCUMENTATION.md`](../performance/SCHEMA_DOCUMENTATION.md). |
| **192** | Compile-time string interning? | enhancement, performance | Perf nice-to-have. The arena/interner is already the hottest read site (see [`SAFETY.md`](SAFETY.md) §B); the 2026-07-02 audit settled the related pin!/pin_static policy ([`PERFORMANCE.md`](../performance/PERFORMANCE.md) Principle 1). Measure before investing. Backlog. |
| **82** | Manually copy over perldoc as rustdoc | enhancement, help wanted, documentation | Doc debt. Long tail. |
| **80** | space XMhints as elided arguments | enhancement | **Open — still reproduces (verified 2026-06-16).** `$[D_{0},\ ]$` → the escaped space is dropped, so the grammar sees a dangling `,]` and rejects. Fix = emit an XMHint for the in-math space and teach the marpa grammar to treat it as an elided argument slot. Real grammar work, not a quick win. Backlog. |

## Recently closed (since the 2026-05-24 refresh — outcomes)

| # | Closed | Outcome |
|---|---|---|
| **292** | 2026-07-17 (branch) | Bug: a user `--stylesheet` that `<xsl:import>`s the engine via `urn:x-LaTeXML:XSLT:LaTeXML-html5.xsl` failed to parse ("unable to load urn:…"). Root cause: `xslt.rs`'s embedded-XSLT libxml2 input callback matched only the `embed:///` scheme, and was installed only on the embed fallback path — so a disk-loaded user stylesheet using the LaTeXML-canonical `urn:` scheme (which Perl resolves via an XML catalog) had no resolver. Fixed by resolving `urn:x-LaTeXML:XSLT:` (and libxml2's relative-composed `urn:X`) against the embedded table by basename, and installing the callback at the parse chokepoint. Guard `63_xslt_custom_stylesheet::custom_stylesheet_imports_engine_by_urn`. |
| **191** | 2026-07-09 | CLI options — `clap` 4 derive adopted (the issue's suggestion) + every `latexmlc` option whose engine feature works end-to-end wired; remaining options documented with rationale (`--validate` postponed pending a rust-libxml RelaxNG publish; image/crossref/index/bib/daemon/alt-output features absent → flags kept as strict parse errors, option C). Closed via #276. Full rationale record retained below. |
| **101** | 2026-05-26 | Binary size accepted as structural (~47 MB maxperf; ~60k binding functions, no fat generic to shrink). [`RELEASE_CRITERIA.md`](RELEASE_CRITERIA.md) §2, [`PERFORMANCE.md`](../performance/PERFORMANCE.md) build-pipeline. |
| **217** | 2026-06-08 | macOS portable use RESOLVED — suite green on macos-15 arm64; kpathsea 0.3 subprocess fallback; text-node UAF fixed (WISDOM #58). [`archive/PORTABILITY_MACOS_PROBE_2026-06-07.md`](../archive/PORTABILITY_MACOS_PROBE_2026-06-07.md). |
| **93** | 2026-06-09 | Declarative binding dialect → superseded by #247's runtime interpreter + shared `ConstructorBuilder` spine. [`BINDING_DSL_ARCHITECTURE.md`](../parity/BINDING_DSL_ARCHITECTURE.md). |
| **199** | 2026-06-12 | HTML-dialect RelaxNG schema — see [`SCHEMA_DOCUMENTATION.md`](../performance/SCHEMA_DOCUMENTATION.md). |
| **247** | 2026-06-12 | Runtime (Rhai) binding interpreter landed (`script-bindings` feature, off by default). [`script_bindings_plan.md`](../parity/script_bindings_plan.md). |
| **183** | 2026-06-16 | String-literal max-width style item. |
| **171** | 2026-06-16 | XML-replacement parser — resolved as a component of #247 (`ReplacementOp` AST + winnow, decision recorded in [`BINDING_DSL_ARCHITECTURE.md`](../parity/BINDING_DSL_ARCHITECTURE.md)). |
| **127** | 2026-06-18 | 64-bit numbers — fixed by exact `xn_over_d`-style fixed-point unit conversion (`numeric_ops::fixpoint_unit`), bit-exact vs pdftex. The storage was never the bug; f64 unit *conversion* was. |

## #191 — CLI option-coverage detail (audited 2026-05-24; refreshed 2026-07-09; **CLOSED 2026-07-09** via #276 — retained as the remaining-options rationale record)

Authoritative Perl spec = `getopt_specification` in `Common/Config.pm`
(~82 canonical omni options; the `latexmlc` union). Existing Perl-name
aliases: `--destination`, `--noparse`, `--presentationmathml`,
`--contentmathml`, `--xmath`, and (2026-07-09) `--nopresentationmathml`,
`--nocontentmathml`, `--nokeepXMath`, `--navtoc`.

**Landed 2026-07-09** — every flag whose engine feature already exists is now
wired (option C: wire real features, keep the parser strict — see below).
- *Batch 1:* `--strict` (State `STRICT`), `--includestyles` (State
  `INCLUDE_STYLES`/`INCLUDE_CLASSES`), `--comments` (positive of `--nocomments`),
  `--xml` (= `--format=xml`), `--embed` (= `--whatsout=fragment`), `--nopost`,
  `--nosplit`, the math-rep negations `--nopmml`/`--nocmml`/`--nomathtex`/
  `--noxmath`, the `--navtoc` alias. `--debug` already existed.
- *Batch 2:* `--timestamp` (`=0` omits; XSLT `TIMESTAMP` footer param),
  `--icon` (XSLT `ICON` param + favicon copy), `--nographicimages`/
  `--graphicimages` (gate the Graphics post-phase), and the positive-complement
  parity flags `--numbersections`, `--mathparse`, `--invisibletimes`,
  `--defaultresources`.

- **Postponed to next release:** `--validate`/`--novalidate` — real RelaxNG
  validation is gated on the rust-libxml fork providing a safe RelaxNG interface
  (published as `libxml 0.3.16`); `Post::Document::validate()` is a stub today.
  Plan in [`SYNC_STATUS.md`](../SYNC_STATUS.md).
- **Remaining cheap gaps (feature exists / near):** `--profile`
  (biggest — `fragment`/`math`/`article`/…; planned as **TOML** profiles
  deserialized into the clap option struct, not Perl `.opt` — design in
  [`OXIDIZED_DESIGN.md`](../parity/OXIDIZED_DESIGN.md) "Future Work") + its `--mode` alias.
- **Feature gaps (flag absent because the feature is) — kept as hard parse
  errors, NOT stubbed:** `--mathimages` / `--mathsvg` / `--pictureimages`
  (need the unwired LaTeXImages latex+dvipng pipeline), `--svg` (**deferred**:
  the HTML5 XSLT already renders `<ltx:picture>` as inline `<svg>` by default,
  so the standalone `svg.rs` post-processor is redundant + produces divergent,
  unverified output — verified 2026-07-09), `--jats` / `--html4` / `--tex` /
  `--box` output, `--crossref` / `--bibliography` / `--index` /
  `--permutedindex` / `--splitbibliography`, `--openmath` / `--unicodemath` /
  `--plane1` / `--hackplane1` / `--parallelmath` / `--linelength` /
  `--mathimagemagnification`, `--scan`/`--noscan` (Scan IS wired as post Phase 2
  but the off-switch is parked with the crossref cluster) / `--prescan` /
  `--dbfile` / `--urlstyle` / `--base` / `--omitdoctype` (no DTD), daemon mode
  (`--port`, `--expire`, `--cache_key`, `--address`, `--autoflush`).
- **Intentional non-goals:** `--output` (we keep `--destination` / `--dest`).

**Design decision (2026-07-09, option C).** Deferred-feature flags are
*deliberately left as clap "unexpected argument" errors* rather than
accept-and-warn stubs — a strict parser never silently accepts a flag whose
feature is absent (no misleading success). Revisit per-flag as each feature
lands. The parser-library question the issue raised (clap vs alternatives) is
settled: clap 4 derive, adopted.

## Reading

* **Beyond-Perl showcase:** #47 + #92 — the ar5iv-editor + provenance
  substrate ([`SOURCE_PROVENANCE.md`](../performance/SOURCE_PROVENANCE.md)).
* **Release gates:** #143 (toolchain pin), plus the license + safety items in
  [`RELEASE_CRITERIA.md`](RELEASE_CRITERIA.md) that have no issue number.
* **Closed (2026-07-09, via #276):** #191 — clap 4 derive adopted + every
  real-feature `latexmlc` option wired; remaining options documented with
  rationale (`--validate` postponed pending a rust-libxml RelaxNG publish;
  `--profile`/`--mode` the largest near-term gap; image/crossref/index/bib/
  daemon/alt-output flags kept as hard parse errors — detail above).
* **Backlog / exploratory:** #192, #94, #82, #80.
