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
> Last refreshed: **2026-05-24**.

Tracker: <https://github.com/dginev/latexml-oxide/issues>

## Open issues

| # | Title | Labels | Local status / interpretation |
|---|---|---|---|
| **47** | [Feature] Accurate latex linting | enhancement | **Prioritized showcase.** Live source ↔ preview over a shared locator substrate, two clients: the **ar5iv-editor** (CodeMirror web UI) and a **VSCode extension** (webview). Accurate linting falls out of the same substrate. Design: [`SOURCE_PROVENANCE.md`](SOURCE_PROVENANCE.md). *Not* purely post-1.0 — Tier A is near-term and parity-neutral. |
| **92** | Superior debugging and error-reporting for document authors | enhancement | Same source-provenance substrate as #47 ([`SOURCE_PROVENANCE.md`](SOURCE_PROVENANCE.md)): construct-start + macro-origin locators give Rust-compiler-grade author errors, fixing TeX's "error points at the end of the environment". |
| **199** | RelaxNG schema for HTML dialect | enhancement | Real gap. The `ltx` schema exists; the *emitted-HTML-dialect* schema does not. Doubles as the validation contract for the preview track (#47) and a release-CI check. See [`SCHEMA_DOCUMENTATION.md`](SCHEMA_DOCUMENTATION.md). |
| **217** | Portable use on MacOS | — | **Mostly resolved 2026-06-07** ([`PORTABILITY_MACOS_PROBE_2026-06-07.md`](PORTABILITY_MACOS_PROBE_2026-06-07.md)): builds + converts on macos-15 arm64 (brew-texlive probe leg green). MacTeX ships NO libkpathsea (nothing to link) → covered by kpathsea 0.3's subprocess-`kpsewhich` fallback (rust-kpathsea `subprocess-fallback` branch, Perl-parity ls-R cache). Remaining: crates.io release + dep swap, README install matrix, promote probe to gating CI. |
| **191** | Add support for original command-line options | enhancement | **PARTIAL — not closeable.** `clap` 4 derive is adopted (the issue's suggestion) and core options work, but coverage is **~47 flags vs ~95 in the Perl omni set** (`Common/Config.pm`). Audited 2026-05-24 — gaps below. |
| **101** | Binary speed+size for releases | enhancement, performance | `maxperf` is **45 MB / 14 MB tarball**; `.text` is ~36.7 MB — dominated by the **compile-time binding pool**, *not* dumps (dumps are already gzip-embedded to ~870 KB, DEP-12). Re-run `cargo bloat` before acting. [`RELEASE_CRITERIA.md`](RELEASE_CRITERIA.md) §2. |
| **143** | Switch to rust stable, when `#[thread_local]` is stabilized | enhancement, performance | Toolchain-longevity risk for a public-domain tool. Pin a known-good nightly; track stabilization. [`RELEASE_CRITERIA.md`](RELEASE_CRITERIA.md) §3. |
| **94** | Document model: RelaxNG vs Rust data-type trade-offs | enhancement, question, documentation | Doc debt; relates to #199 and [`SCHEMA_DOCUMENTATION.md`](SCHEMA_DOCUMENTATION.md). |
| **93** | Declarative Def/binding dialect | enhancement | Relates to the `DefMacro!`/`DefConstructor!` binding layer; also a potential lever on the #101 binary-size (binding-pool codegen). Large, speculative. Backlog. |
| **192** | Compile-time string interning? | enhancement, performance | Perf nice-to-have. The arena/interner is already the hottest read site (see [`SAFETY.md`](SAFETY.md) §B); measure before investing. Backlog. |
| **183** | String literals and max-width | enhancement | Minor. Backlog. |
| **171** | [exploratory] dedicated parser for XML replacements | enhancement | Exploratory. Backlog. |
| **127** | Are we committed to 64-bit numbers? | enhancement | Type-design question. Backlog. |
| **82** | Manually copy over perldoc as rustdoc | enhancement, help wanted, documentation | Doc debt. Long tail. |
| **80** | space XMhints as elided arguments | enhancement | Math-hint detail. Backlog. |

(Numbers not listed have no open issue or are out of current scope. Re-run
the refresh command to regenerate.)

### #191 — CLI option-coverage detail (audited 2026-05-24)

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

* **Prioritized showcase:** #47 + #92 — the ar5iv-editor + provenance
  substrate ([`SOURCE_PROVENANCE.md`](SOURCE_PROVENANCE.md)); #199 supports it.
* **Release blockers / gates:** #101 (size), #143 (toolchain), #217 (mac),
  plus the license + safety items in [`RELEASE_CRITERIA.md`](RELEASE_CRITERIA.md)
  that have no issue number yet.
* **Partial parity (open):** #191 — clap landed + core options, but ~47/95
  omni flags; `--profile` and a long tail still missing (detail above).
* **Backlog / exploratory:** #192, #183, #171, #127, #93, #82, #80.
