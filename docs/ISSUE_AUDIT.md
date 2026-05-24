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
| **47** | [Feature] Accurate latex linting | enhancement | **Product north star.** Folds together with #92 + the VSCode synced-preview goal into one *source-provenance* track. Two tiers + a process-model prerequisite — see [`RELEASE_CRITERIA.md`](RELEASE_CRITERIA.md) §9. *Not* purely post-1.0: the element-level slice is near-term. |
| **92** | Superior debugging and error-reporting for document authors | enhancement | Same locator foundation as #47; natural co-tenant of an LSP server. Rust-compiler-grade error UX is the differentiator vs "TeX being TeX". |
| **199** | RelaxNG schema for HTML dialect | enhancement | Real gap. The `ltx` schema exists; the *emitted-HTML-dialect* schema does not. Doubles as the validation contract for the preview track (#47) and a release-CI check. See [`SCHEMA_DOCUMENTATION.md`](SCHEMA_DOCUMENTATION.md). |
| **217** | Portable use on MacOS | — | Release-readiness, [`RELEASE_CRITERIA.md`](RELEASE_CRITERIA.md) §3 stage 4. Real blocker: `libkpathsea-dev` packaging → upstream upgrade to `rust-kpathsea`. Needs README install steps + a CI sanity job. |
| **191** | Add support for original command-line options | enhancement | **Largely done.** All four binaries use `clap` 4 derive (`bin/latexml_oxide.rs`). Remaining work is *option-coverage parity* vs Perl `Common/Config.pm`, not "adopt a parser." Downscope or close. |
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

## Reading

* **Product / near-term:** #47 (Tier A), #92, #199.
* **Release blockers / gates:** #101 (size), #143 (toolchain), #217 (mac),
  plus the license + safety items in [`RELEASE_CRITERIA.md`](RELEASE_CRITERIA.md)
  that have no issue number yet.
* **Backlog / exploratory:** #192, #183, #171, #127, #93, #82, #80.
* **Closeable / downscope:** #191 (clap landed).
