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
| **217** | Portable use on MacOS | — | **RESOLVED 2026-06-08** ([`archive/PORTABILITY_MACOS_PROBE_2026-06-07.md`](archive/PORTABILITY_MACOS_PROBE_2026-06-07.md)): the full `cargo test --tests --workspace` suite is **green on macos-15 arm64** (brew-texlive leg: 1390 passed / 0 failed / 0 crashes, 43 binaries). MacTeX ships NO libkpathsea → covered by **kpathsea 0.3.0 (crates.io)** subprocess-`kpsewhich` fallback. The macOS-only worker-thread Node corruption was a **use-after-free of a libxml2-merged text node** (detected via a read of the freed node — benign on glibc, exposed by macOS libmalloc); fixed in `open_text_internal` with a pointer-identity merge check (WISDOM #58), audited for siblings. README install matrix done; probe is a gating CI job. |
| **191** | Add support for original command-line options | enhancement | **PARTIAL — not closeable.** `clap` 4 derive is adopted (the issue's suggestion) and core options work, but coverage is **~47 flags vs ~95 in the Perl omni set** (`Common/Config.pm`). Audited 2026-05-24 — gaps below. |
| **101** | Binary speed+size for releases | enhancement, performance | `maxperf` is **45 MB / 14 MB tarball**; `.text` is ~36.7 MB — dominated by the **compile-time binding pool**, *not* dumps (dumps are already gzip-embedded to ~870 KB, DEP-12). Re-run `cargo bloat` before acting. [`RELEASE_CRITERIA.md`](RELEASE_CRITERIA.md) §2. |
| **143** | Switch to rust stable, when `#[thread_local]` is stabilized | enhancement, performance | Toolchain-longevity risk for a public-domain tool. Pin a known-good nightly; track stabilization. [`RELEASE_CRITERIA.md`](RELEASE_CRITERIA.md) §3. |
| **94** | Document model: RelaxNG vs Rust data-type trade-offs | enhancement, question, documentation | Doc debt; relates to #199 and [`SCHEMA_DOCUMENTATION.md`](SCHEMA_DOCUMENTATION.md). |
| ~~**93**~~ | ~~Declarative Def/binding dialect~~ | enhancement | **CLOSED 2026-06-09 → #247.** Superseded by the runtime-interpreter work tracked in #247. The Rhai `script-bindings` layer + the shared `ConstructorBuilder` spine are the compile-time-AND-runtime declarative dialect #93 asked for; a YAML *data* surface remains possible later as an additional front-end on the same spine. See [`BINDING_DSL_ARCHITECTURE.md`](BINDING_DSL_ARCHITECTURE.md). |
| **247** | Interpreted runtime bindings | enhancement | **Active.** The runtime (Rhai) binding interpreter — the live work of this track. Builder spine landed (`ConstructorBuilder`); macros, both constructor forms, primitives, `DeclareOption`, the option-bag map, `document`/`whatsit` proxies all validated. Subsumes #93; #171 is its template-parser component. See [`BINDING_DSL_ARCHITECTURE.md`](BINDING_DSL_ARCHITECTURE.md), [`script_bindings_plan.md`](script_bindings_plan.md). |
| **192** | Compile-time string interning? | enhancement, performance | Perf nice-to-have. The arena/interner is already the hottest read site (see [`SAFETY.md`](SAFETY.md) §B); measure before investing. Backlog. |
| **183** | String literals and max-width | enhancement | Minor. Backlog. |
| **171** | [exploratory] dedicated parser for XML replacements | enhancement | **Resolved (component of #247):** unify the two template impls (compile-time `Constructor::Compiler` proc-macro + runtime `apply_template`) behind one `ReplacementOp` AST + a **winnow** parser in `latexml_core` (winnow already lock-pinned via `toml_edit`; clap-family/trusted; correct-RD + structured-error guarantees). `latexml_codegen` (depends on core) consumes the AST for `quote!`; core's interpreter consumes it at runtime — no new crate. RSTML/typed-html/RSX rejected (compile-time-only). See [`BINDING_DSL_ARCHITECTURE.md`](BINDING_DSL_ARCHITECTURE.md) "Resolved decisions". |
| **127** | Are we committed to 64-bit numbers? | enhancement | **NOT closeable — surpass-Perl opportunity (re-read 2026-06-16).** Storage is settled (`Number(i64)`, `Dimension(i64)`, `MuDimension(i64)`, `Glue{i64…}`, `Float(f64)`; no `i32`/`f32`). But @brucemiller's rebuttal reframes it: TeX uses **fixed-point**; Perl's `UNITS` inch (`72.27 * 65536` as a float) is admittedly *sloppy*, "always used with the appropriate adjustments." The surpass-Perl play (the author's option 2, B-Book §458) is to stop storing units as floats and do TeX's exact `xn_over_d` integer conversion so we are bit-faithful to TeX, not merely f64-parity with Perl. **Decision test: `\dim=1in \the\dim`** must equal TeX/pdfTeX's `72.26999pt` — we already do (every `1<unit>` matches). The residual gap is fractional multipliers: a 200k-decimal sweep vs TeX's exact `xn_over_d` shows off-by-1-sp float errors on `cm` (~0.4%), `bp`, `mm`, `cc` (`in`,`dd` clean). **FIXED 2026-06-16** (PR closing this issue): `numeric_ops::fixpoint_unit` does `sp = floor(fix·num/den)` in i128 with `state::convert_unit_ratio`'s exact per-unit fractions (`in=7227/100, bp=7227/7200, cm=7227/254, mm=7227/2540, dd=1238/1157, cc=14856/1157`), threaded through every dimension-construction site (gullet/glue/dimension/graphics). Verified bit-exact vs pdftex; full suite green. The mistake was never the i64 *storage* — it was doing the unit *conversion* in f64; widening f32→f64 in 2023 only shrank the error to ±1 sp, it didn't remove the float-arithmetic class. |
| **82** | Manually copy over perldoc as rustdoc | enhancement, help wanted, documentation | Doc debt. Long tail. |
| **80** | space XMhints as elided arguments | enhancement | **Open — still reproduces (verified 2026-06-16).** `$[D_{0},\ ]$` → lexemes `[OPEN:[, UNKNOWN:D, _0, WIDE_PUNCT:,, CLOSE:]]` with **no token** where `\ ` was: the escaped space is dropped, so the grammar sees a dangling `,]` and rejects ("Grammar did not recognize expression"). Fix = emit an XMHint for the in-math space and teach the marpa grammar to treat it as an elided argument slot. Real grammar work, not a quick win. Backlog. |

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
* **Backlog / exploratory:** #192, #183, #171, #93, #82, #80. (#127 fixed 2026-06-16.)
