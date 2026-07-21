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
> Last refreshed: **2026-07-20** (17 open, from `gh issue list`). This is a
> reconciled refresh across two same-day edits: one closed the #304/#305/#307
> user-report cluster and #309, the other corrected the stale #192/#82 rows (both
> closed **2026-07-17**) and added #297/#303. Ground truth then moved again — #309
> closed (PR #310 merged), and a fresh **#314–#321 Rhai binding-API** cluster was
> filed the same day. Both are reflected below. Failure mode worth knowing: this
> file drifts fast — re-run the one-liner rather than incrementally editing rows.

Tracker: <https://github.com/dginev/latexml-oxide/issues>

## Open issues (15)

| # | Title | Labels | Local status / interpretation |
|---|---|---|---|
| **47** | [Feature] Accurate latex linting | enhancement | **Prioritized beyond-Perl showcase.** Live source ↔ preview over a shared locator substrate, two clients: the **ar5iv-editor** (CodeMirror web UI) and a **VSCode extension** (webview). Accurate linting falls out of the same substrate. Design: [`SOURCE_PROVENANCE.md`](../performance/SOURCE_PROVENANCE.md). *Not* purely post-1.0 — Tier A is near-term and parity-neutral. |
| **92** | Superior debugging and error-reporting for document authors | enhancement | Same source-provenance substrate as #47 ([`SOURCE_PROVENANCE.md`](../performance/SOURCE_PROVENANCE.md)): construct-start + macro-origin locators give Rust-compiler-grade author errors, fixing TeX's "error points at the end of the environment". |
| **143** | Switch to rust stable, when `#[thread_local]` is stabilized | enhancement, performance | Toolchain-longevity risk for a public-domain tool. Pin a known-good nightly; track stabilization. [`RELEASE_CRITERIA.md`](RELEASE_CRITERIA.md) §3. |
| **94** | Document model: RelaxNG vs Rust data-type trade-offs | enhancement, question, documentation | Doc debt; relates to the (closed) #199 HTML-dialect schema and [`SCHEMA_DOCUMENTATION.md`](../performance/SCHEMA_DOCUMENTATION.md). |
| **297** | latexml_oxide 0.7.4 binding transition: `nowrap.sty.ltxml` | documentation, packages | Filed 2026-07-18. A user-supplied `.ltxml` binding on `--path` is ignored: `Warning:missing_file:nowrap … No dispatcher entry and no raw file found on disk` (`latexml_core/src/binding/content.rs`). Perl finds it silently. This is the **user-supplied `.ltxml` discovery** path — `.ltxml` is Perl, so the Rust answer is either the Rhai `runtime-bindings` front-end or a clear diagnostic; today it is neither. |
| **303** | Precompiled kernel (dump) release and update strategy | enhancement | Filed 2026-07-18, follow-up to #299/PR #300. The dump is keyed to the TL **year**, but the LaTeX format is not frozen within a year (`\fmtversion`, `tlmgr` `fmttriggers` rebuilds) — a false positive was traded for a rarer false negative. Candidate signals: key on `\fmtversion` + L3 date, or compare against the ambient `latex.fmt` mtime; warn-not-fail. |
| **311** | Raw-loaded package `\newif` conditionals die with the standalone subfile group | bug | Filed 2026-07-20. **Explicitly NOT Rust-only** — same-host Perl reports the identical error, and `standalone.sty.ltxml` uses the same `bgroup` architecture. General shape: any raw-loaded package pairing a `\newif` with a document-level hook (`\ifpgf@external@grabshipout` + `\AtEndDocument`). Two fixes tried and refuted (hoisting the `RequirePackage`; `\globaldefs=1`). Related to the group-scoping class in `SYNC_STATUS.md`. |
| **312** | 0.7.5-rc1: many problems rendering math (default CSS) | (none) | Filed 2026-07-20 — **untriaged.** Self-contained `amsmath` reproducer in the issue body (`\left( … \right)` spacing, `\tag`, `align`/`align*`, `\prime`). Needs the usual same-host Perl + pdflatex classification before any code change. |
| **316** | not possible to call `DefPrimitive` from `DefPrimitive`? | enhancement | Filed 2026-07-20. Rhai-API expressiveness gap (nested primitive definition). Cluster #314–#321. |
| **317** | allow extra parameter of `RequireResource` in Rhai binding API | enhancement | Filed 2026-07-20. Rhai-API surface gap. Cluster #314–#321. |
| **318** | allow external commands in Rhai bindings | enhancement | Filed 2026-07-20. Rhai-API surface gap. Cluster #314–#321. |
| **319** | missing `Info`, `Fatal`, `NoteSTDERR`, `NoteLog`, progress spinners, etc. in Rhai binding API | enhancement | Filed 2026-07-20. Rhai-API diagnostics surface gap. Cluster #314–#321. |
| **320** | expose the latexml-oxide version to Rhai bindings | enhancement | Filed 2026-07-20. Rhai-API surface gap. Cluster #314–#321. |
| **321** | add `LookupDefinition` to Rhai bindings | enhancement | Filed 2026-07-20. Rhai-API surface gap. Cluster #314–#321. |
| **80** | space XMhints as elided arguments | enhancement | **Open — still reproduces (verified 2026-06-16).** `$[D_{0},\ ]$` → the escaped space is dropped, so the grammar sees a dangling `,]` and rejects. Fix = emit an XMHint for the in-math space and teach the marpa grammar to treat it as an elided argument slot. Real grammar work, not a quick win. Backlog. |

## Recently closed (since the 2026-05-24 refresh — outcomes)

| # | Closed | Outcome |
|---|---|---|
| **315** | 2026-07-20 (PR) | **Bug (Rust-only; internal-representation leak) — fixed via structural list access.** `LookupString("class_options")` leaked the internal `VecDequeStored[…]` Debug repr (`class_options` is a `Stored::VecDequeStored` of option strings). First fix (#325, comma-separated reversion) was **reverted**: the comma-join was an over-general heuristic in the *generic* `VecDequeStored` reversion, and Perl has no canonical list→string anyway (an arrayref stringifies to `ARRAY(0x…)`; `join(',')` is applied only at option sites). Correct fix: **`LookupString` is scalar-only** — returns `""` for a list value (state.rs, gated by new `Stored::is_list`), never the Debug repr; and a new Perl-style **`LookupValue`** binding returns the list AS a Rhai array (`["a4paper","12pt"]`), mirroring Perl's `LookupValue` → arrayref, so the caller iterates/joins structurally. `LookupTokens` unchanged (still reverts to concatenated tokens; #314). Foundational for #321. Guards: `lookup_string_on_list_value_is_empty_not_leaked`, `lookup_value_on_list_returns_rhai_array`. |
| **314** | 2026-07-20 (PR) | **Bug (Rust-only; no Perl analogue — RefCell borrow model).** `LookupTokens("class_options")` from a Rhai binding panicked `RefCell already borrowed`. `class_options` is a `Stored::VecDequeStored`; `state::lookup_tokens` (state.rs:1657) held its immutable `state!()` borrow across the queue→Tokens conversion, which reverts each `String` item through `mouth::tokenize_internal` — a *mutable* STATE borrow → `borrow_mut` under a live `borrow`. The adjacent `Stored::String` branch already dropped the borrow before tokenizing; the VecDeque branch did not. Fixed by cloning the queue and dropping the borrow first (the general root — `lookup_tokens` is the only fn producing `Option<Tokens>`, hence the sole reentry site; arena is re-entrant so unaffected). Guard: `script_bindings::tests::lookup_tokens_on_vecdeque_value_does_not_panic`. |
| **309** | 2026-07-20 (PR) | **Shared upstream bug, fixed ahead of Perl.** `\subimport`ing a child whose preamble is `\documentclass[12pt]{article}` warned `missing_file:12pt`: `standalone.sty.ltxml` L24-33 `RequirePackage`s the comma-split **optional** argument of the intercepted `\documentclass`, but that argument holds class *options*, not packages. Same-host Perl warns identically, so the fix is a documented divergence (OXIDIZED_DESIGN #63, KNOWN_PERL_ERRORS #54) rather than a straight patch. The loop is now gated as `standalone.sty` L604-614 gates it — class must be `standalone` — and further limited to the options `standalone.cls` turns into a same-named package load (`tikz`, `pstricks`, `preview`, `varwidth`, `multido`), which preserves upstream LaTeXML#1432's `\documentclass[tikz]{standalone}`. Also fixes `[border=2pt]{standalone}`. Follow-on to #293 (the mandatory-argument half). Guard: `06_cluster_regressions::standalone_subimport_documentclass_no_spurious_require`. |
| **192** | 2026-07-17 | Compile-time string interning. (Was still listed as open in the 2026-07-19 refresh.) |
| **82** | 2026-07-17 | Manually copy perldoc over as rustdoc. (Was still listed as open in the 2026-07-19 refresh.) |
| **304** | 2026-07-19 | **Not reproducible — environment, not latexml_oxide.** "TEXINPUTS ignored for `\input{my_core}`" on a Linux Mint VirtualBox guest with the tree on a `/mnt/g` shared-folder mount. Unreproducible here across five matched factors (multi-part `TEXINPUTS`, trailing/empty colons, `//` recursion, a real `st_nlink=1` ntfs-3g mount, the bundled TL2025 lib). Resolved on the reporter's side by rebooting the VM; their `KPATHSEA_DEBUG=32` log then showed `TEXINPUTS` reaching the process, libkpathsea initialized, and the file resolving — positively excluding our resolver paths. **Two real defects were nevertheless found while investigating** (PR #308): (a) `select_kpaths()` discarded a failed `Kpaths::new()` with `.ok()?`, silently disabling ALL file resolution — fixed upstream in kpathsea 0.3.4 (dginev/rust-kpathsea#25, degrading program-name anchor) plus a subprocess fallback here; (b) a per-lookup subprocess fallback added mid-investigation cost one `kpsewhich` spawn per distinct missing file (30 spawns / 20 missing packages; 2.54 s against a 0.20 s conversion) — removed. **Durable outcome:** every conversion log now records the resolved kpathsea backend, so this class of report is diagnosable from an ordinary log. Guard: `003_kpathsea_backend_resolution.rs`. |
| **305** | 2026-07-18 | "Where is latexml.sty?" — **out of scope by design.** Raw `.sty` files are not shipped; the compiled `latexml_sty.rs` binding always applies, needs no path, and cannot be overridden by a raw `latexml.sty` on the search path even under `--includestyles` (precedence is decided by `input_definitions`'s `_loaded` gate, not `find_file_aux`'s `notex` gate — see WISDOM #63). |
| **307** | 2026-07-19 | **Downstream of #304, not an `\iflatexml` bug.** `\iflatexml\else\usepackage[fit]{truncate}…\fi` took the `\else` branch. `\iflatexml` is defined only by loading `latexml.sty` — identical in Perl LaTeXML (`latexml.sty.ltxml:27`); neither predefines it, and both emit the same `undefined` error and take `\else` for a bare `\iflatexml` (verified same-host). The reporter's log proves `\usepackage{latexml}` lives inside `my_core.tex` (`latexml_sty.rs` loads the moment that file is processed), so while #304 made it unresolvable the conditional was never defined. Their exact preamble converts cleanly on released 0.7.4 here. No code change. |
| **301** | 2026-07-18 (branch) | Build: `cargo test` on the published `latexml` crate panicked in the `exemption_keys_have_unique_stems` unit test — "no .tex fixtures found under tests/ — wrong CWD?" (reported by a Nixpkgs `buildRustPackage` consumer). Root cause: that `#[cfg(test)]` test lives under `src/` (so it *ships* in the package) yet scanned the **CWD-relative** `Path::new("tests")` at run time, while `tests/` is deliberately EXCLUDED from the tarball (Cargo.toml `exclude`, crates.io 10 MiB cap). Fixed by locating the corpus via `env!("CARGO_MANIFEST_DIR")` (CWD-independent) and treating an absent corpus as a legitimate skip, not a failure (the audit helper returns `None`). Guards: `util::test::exemption_audit::{audit_skips_when_corpus_absent, audit_detects_duplicate_stems}`. |
| **299** | 2026-07-17 (branch) | Bug (Rust-only distribution machinery; no Perl analogue): the standalone 0.7.4 release binary printed `Warning:latex_dump:mismatch TeXLive MISMATCH — dump stamped 'kpathsea version 6.4.1', ambient kpsewhich '6.4.2' … Run tools/make_formats.sh` on every run under TL2026. **Two defects.** (a) False positive: `build.rs`'s generated `compare_stamp_to_ambient` compared the exact kpathsea version *string*, so a within-year patch bump (release-build container's 6.4.1 vs shipped-TL2026's 6.4.2) tripped it even though the embedded **TL2026** dump is the correct dump for a **TL2026** install (same macro set). The kpathsea string is also not a reliable cross-year discriminator (CLAUDE.md: TL2023/TL2025 share it). (b) The remedy named `tools/make_formats.sh`, which a standalone-binary user has no source tree for. **Fix:** compare TeX-Live **years** (already how the dump is selected) via a testable `dump_paths::dump_year_mismatch_warning(dump_year, ambient_year, from_embedded)` — warn only on a genuine year mismatch (fell back to another year's dump), stay silent when years match or the ambient year is undetectable; the embedded-path message drops the `tools/` reference. Guard `dump_paths::stamp_check_tests` (6 cases incl. the reporter's TL2026/TL2026 silent case). |
| **293** | 2026-07-17 (branch) | Bug: `\subimport`ing a `standalone` child with a `\documentclass{article}` preamble warned `missing_file:article`. Root cause: `standalone_sty.rs`'s `\@standalone@documentclass[]{}` intercept required the **mandatory** class-name arg, whereas Perl `standalone.sty.ltxml` L24-33 requires the **optional `[]`** list (`$packages = $_[1]`) and ignores the class. Fixed by binding the optional arg; guard `06_cluster_regressions::standalone_subimport_documentclass_no_spurious_require`. |
| **291** | 2026-07-17 (branch) | Bug: `\setcounter{tocdepth}{0}` ignored — the full ToC rendered instead of chapters only. Root cause: `Post::CrossRef::gen_toc` ignored the `<ltx:TOC>` `select`/`lists` attributes (hardcoded `NORMAL_TOC_TYPES` + `inlist=="toc"`), so `tocdepth` (which rides on `select`) had no effect. Fixed by porting Perl `CrossRef.pm::gentoc` L246-261 (type filter from `select`, `inlist_match` on `lists`). **Same root defect also silently broke the LaTeXML#2316 arXiv-fork "abstract exempt from `\tableofcontents`" half** (abstract leaked into the user TOC); the fix completes it. Guards: `06_cluster_regressions::{tocdepth_select_restricts_the_toc, nav_toc_includes_abstract_issue_2316}`. **Follow-up hardening (same branch):** (a) a *negative* `tocdepth` (`\setcounter{tocdepth}{-1}` = parts only) panicked — the `select` computation cast `tocdepth+1` through `as usize`, overflowing on negatives (debug panic; release over-full ToC); fixed to compute in signed space per Perl `latex_constructs.pool.ltxml` L727-733 (`0 .. $td` is empty for negative `$td`). (b) honoring `lists` also repaired `\listoffigures`/`\listoftables`, which the old hardcoded `"toc"` bucket had listed a document *section* in. Guards: `06_cluster_regressions::{tocdepth_negative_is_parts_only_no_panic, listoffigures_lists_figures_not_toc_sections}`. |
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
