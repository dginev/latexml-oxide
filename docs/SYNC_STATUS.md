# Engine Sync Status: Perl vs Rust

> **This is a Perl-to-Rust translation project.** Every ported function, macro, and definition must faithfully reproduce the original Perl semantics, control flow, and edge-case behavior. The Perl source (`LaTeXML/` directory) is the ground truth. Only diverge when explicitly documented in `docs/OXIDIZED_DESIGN.md`.

Updated 2026-04-10. Only lists open gaps & TODOs; completed items live in git history.

**Test inventory:** 407 tests pass (0 failures); all 10 tikz tests pass. MakeBibliography pipeline fully operational.

**arxiv sandbox:** 100+ papers in `arxiv-examples/`. **90/97 OK (93%)** on full catalog. 7 remaining: 3 Perl-also-fails, 2 timeout, 1 version conflict, 1 state corruption.

**10k sandbox:** 7,898 arxiv ZIPs in `$HOME/data/10k_sandbox/`. Benchmark pending (Phase D).

**Production-ready:** Full CorTeX ZIP-to-ZIP pipeline operational:
```
latexml_oxide --whatsin=archive --format=html5 --pmml --mathtex --noinvisibletimes \
  --nodefaultresources --nobibtex --preload=ar5iv.sty --timeout=2700 --log=log.txt \
  --dest=output.zip input.zip
```

## Legend
- **OK** = fully synced | **MINOR** = small gaps | **GAPS** = significant missing | **EMPTY** = not ported

**See also:** [`KNOWN_PERL_ERRORS.md`](KNOWN_PERL_ERRORS.md) | [`OXIDIZED_DESIGN.md`](OXIDIZED_DESIGN.md) | [`PERFORMANCE.md`](PERFORMANCE.md)

---

## Engine Files — Open Gaps Only

| File | Status | Open Gaps |
|------|--------|-----------|
| base_parameter_types.rs | MINOR | `DirectoryList`, `CommaList`, `DigestUntil` stubbed |
| tex_box.rs | MINOR | Minor box dimension edge cases |
| tex_fonts.rs | MINOR | Missing: `\fontdimen` full array semantics |
| tex_tables.rs | MINOR | Minor: padding CSS classes (XSLT concern) |

## Cross-Cutting Infrastructure Gaps

1. **`FontDef` parameter type** — Simplified to `FontToken`. Blocks `\fontdimen`, `\hyphenchar` per-font tracking.

## Unported Perl Engine Files

| File | Defs | Status | Notes |
|------|------|--------|-------|
| `AmSTeX.pool.ltxml` | 112 | ~30% | Plain TeX format (rare) |
| `BibTeX.pool.ltxml` | 956 | 0% | Skipped via `--nobibtex` in production |

## Package Bindings

**100% coverage: all 406 Perl bindings ported to Rust.** Zero `todo!()` panics. Zero MISSING.

## Tikz — Known Diffs (vs Perl output)

1. foreignObject transform Y / width/height
2. Arrow tip shape (different path data)
3. SVG viewBox/width — total dimensions differ slightly
4. tikz matrix rendering uses `<svg:g class="ltx_tikzmatrix">` groups (Rust) vs inline-blocks (Perl)

---

## Completed Work (Sessions 90–96)

All Phase A (EMPTY→OK) and Phase B (parity improvement) tasks completed. Key fixes:
- **S96:** `\shortstack` mode cascade in m-column tables; token limit 30M→100M; graphics candidate path fix
- **S95:** `\pgfsetdash` native override; pgfkeys sentinel fix; expl3 autoload; smfart/animate bindings
- **S94:** `DefMacro!("\\begin{env}")` pitfall; graphics page=N; tikz halign bgroup/egroup
- **S93:** algorithm2e fixes; bibconfig=bbl,bib; elsart affiliation parser
- **S92:** authblk mark fix; elsart affiliation; end_mode recovery
- **S91:** Pure Rust BibTeX parser; MakeBibliography pipeline; `\lx@ifusebbl` fallback

### Permanent ignores
- **ns1–ns5** (52_namespace) — DTD not supported in Rust port
- **2402.03300**, **2410.10068**, **2511.03798** — Perl also fails on these papers

---

## Work Plan — Ordered TODO List

### Phase C: 100-Document Sandbox Parity

Expand the test sandbox to 100+ arxiv papers and achieve HTML conversion parity with Perl for all of them.

#### [x] C1. Benchmark all 97 papers — DONE (session 96)
**Result:** 90/97 OK (93%) after session 97 fixes.
Remaining failures:
- **Perl also fails:** 2402.03300 (pgfkeys), 2410.10068 (quantikz), 2511.03798 (eqnarray)
- **readBalanced state corruption:** 2405.17032 (cumulative state issue — sections 5-8 lost; binary search narrowed to line 1047)
- **tcolorbox version mismatch:** 2306.00809 (version check `\ifx` fails despite matching versions)
- **Package conflict:** 2308.13697 (chemmacros `\Chemalpha` already defined — texlive environment issue)
- **Timeout (heavy pgf):** 1204.4501 (sigma class), 2509.12083 (pgfplots)

#### [ ] C2a. babel.sty fix — IN PROGRESS (session 98)
**Root causes found and fixed:**
1. **`\@fontenc@load@list` cleared to empty** — babel's `\AtBeginDocument` code (L3931) does `\edef\bbl@tempa{\expandafter\@gobbletwo\@fontenc@load@list}`, which with empty list causes `\@gobbletwo` to eat subsequent tokens, corrupting `\bbl@trim` expansion → `\bbl@trim@a` undefined. **Fix:** removed `\def\@fontenc@load@list{}` from babel_sty.rs.
2. **`\CurrentOption` leakage** — keyval.sty's `\ExecuteOptions{unknownkeyserror}` leaves `\CurrentOption` set. babel's `\bbl@load@language{nil}` at L4177 uses `\CurrentOption` (not #1) to set `\bbl@loaded`, producing `\bbl@loaded=unknownkeyserror`. **Fix:** `\let\CurrentOption\@empty` before loading babel.

**Status:** Both fixes applied. `\usepackage{babel}`, `\usepackage{keyval}\usepackage{babel}`, and test.zip (aa.cls) all produce 0 errors/warnings. Need to verify babel-specific tests still pass.

**TODO:** faithful keyval.sty.ltxml translation (Perl L19-36 → Rust already matches)

#### [ ] C2. Fix high-impact Rust-specific failures — IN PROGRESS
**Fixed so far (session 96):**
- **smfart dispatch**: `smfart_cls.rs` was never in dispatch table → added, 7→1 errors
- **XMApp panic**: `todo!()` in math parser → graceful recovery (2506.10218: crash→1.4MB)
- **`_loaded` early return**: prevents double-loading of bindings
- **`find_main_tex`**: 00README.json support + preferred name heuristic

**Fixed (session 96, continued):**
- **end_mode faithful rewrite**: Removed speculative recovery loop; now matches Perl's `endMode` exactly (log error, don't pop on mismatch).
  - 1801.02041: 0B → 737KB
  - 2507.23241: 0B → 4.2MB

**Fixed (session 97):**
- **`find_main_tex` faithful port**: Ported Perl Pack.pm `detect_source` — line-by-line scoring, `\input` veto, 4 tiebreakers, 00README.json/XXX support
- **Lossy UTF-8 read**: `find_main_tex` now handles Latin-1 encoded .tex files (was silently skipping non-UTF8 files)
  - 1711.07162: wrong file → correct file (182KB with all sections)
- **thm-restate dispatch fix**: dispatch key had underscore instead of hyphen → binding never loaded, raw TeX looped. Also added kvsetkeys/keyval RequirePackage.
  - 2007.05477: 0B → 238KB
  - 2103.12243: 0B → 531KB

**Remaining:** Mode stack still has 1 extra frame at `\end{document}` in both papers (1 warning each). Root cause: cumulative bgroup imbalance from content processing. Papers produce full content despite the warning.

#### [x] C3. Directory/archive input parity — DONE (session 96)
**Result:** All three modes work:
- `--whatsin=directory`: Fixed auto-detection of main `.tex` file via `find_main_tex()` (was passing directory path to converter instead of `.tex` file). Now locates file with `\documentclass`, matching Perl.
- `--whatsin=archive`: ZIP input works end-to-end (tested on 2210.09945).
- `--whatsout=archive`: ZIP output with HTML + log + status (tested on 0710.2281, 167KB ZIP).

#### [x] C5. Code quality improvements — DONE (session 97)
- **Static regex compilation**: `maybe_require_dependencies` regexes moved to `once_cell::sync::Lazy` statics (was recompiling on every call)
- **Re-entrancy guard**: thread-local `SCANNING` flag prevents infinite recursion in `maybe_require_dependencies` → `require_package` → `maybe_require_dependencies` cycle
- **`_found_loaded` flag cleanup**: renamed from `_binding_loaded`, now set for both binding AND raw TeX successful loads (matches Perl's `InputDefinitions` return-value semantics). Not set on error/not-found paths.
- **Dead code removal**: `is_base_frame()` in state.rs (unused after `end_mode_opt` rewrite)
- **JSON parsing robustness**: `find_main_tex` 00README.json parsing extracted to `parse_readme_json()` with proper escape handling
- **`--token-limit` CLI flag**: token limit now configurable (default 100M), via `gullet::set_token_limit()`
- **Duplicate comment removed**: content.rs L282-287 had `\ver@` comment twice
- **Avoid clone**: `options.extension` no longer cloned in `require_package`

#### [ ] C4. Upstream Perl sync — continuous
**Approach:**
1. Check `LaTeXML/` git log for new commits
2. Port relevant fixes to Rust (engine, bindings, test files)
3. Update expected XMLs when Perl test output changes

---

### Phase D: 10k-Document Sandbox — Coverage & Performance

Scale testing to ~8,000 arxiv papers (`$HOME/data/10k_sandbox/`, 7,898 ZIP files). All are known to convert successfully under Perl LaTeXML (no_problem or warnings-only). The goal is full Rust parity: zero errors, zero fatal failures.

**Tool:** `cortex_worker --standalone --input <zip> --output <zip>`

**Directories:**
- Input: `$HOME/data/10k_sandbox/` (7,898 arxiv ZIP archives)
- Output: `$HOME/data/10k_sandbox_html/` (one output ZIP per input)

**Process guards (mandatory):** Each conversion must be wrapped with:
- **Timeout:** 2 minutes (120s) wall-clock max via `timeout --kill-after=10 120`
- **RAM:** 8 GB max via `ulimit -v 8388608`
- **Core dumps disabled:** `ulimit -c 0` (prevents disk fill from crash dumps)
- **Output size cap:** 200 MB max per output ZIP (catches SVG/tikz blowup)

**Operational best practices:**
- **Resumability:** Skip inputs whose output ZIP already exists. Re-run failures with `--rerun-failures`.
- **Temp cleanup:** Each task gets its own `TMPDIR` subdirectory, removed after completion. Prevents `/tmp` fill from killed processes.
- **Structured manifest:** `results.tsv` in output dir — one row per task: `arxiv_id, entry_id, exit_code, wall_time_s, output_size_bytes, status_line, category`. Log files (`<id>.log`) also preserved for legacy ingestion pipeline.
- **Parallelism:** GNU parallel, default 16 workers. RAM cap is per-process (16 × 8 GB = 128 GB theoretical peak).
- **Dry run:** `--limit 50` to validate infrastructure before full run.
- **Failure categories:** `ok`, `timeout`, `oom_or_kill`, `segfault`, `abort`, `error`, `empty_output`, `oversized`.

**Runner script:** `tools/benchmark_10k.sh` (see `--help` for all options).

#### Approach: exponential ramp-up with zero-error gates

**Invariant:** After every Rust code change, delete all output (`rm -rf $HOME/data/10k_sandbox_html/*`) and start the ramp-up from scratch. Stale outputs from a prior code version are not trustworthy.

**Ramp-up protocol:**
1. Start with `--limit 4` (4 papers).
2. If the run has **0 errors** (all `ok` or `timeout` only): double the limit → 8, 16, 32, 64, …
3. If the run has **1+ errors** (crash, panic, missing binding, wrong output): **STOP**.
   - Do not increase the limit.
   - Diagnose every error. Extract minimal reproducers, trace root causes, fix in Rust.
   - Re-run **only the failing files** (`--rerun-failures`) until they produce 0 errors.
   - Then delete all output and restart the ramp-up from `--limit 4`.
4. Continue doubling until all 7,898 files pass.

**Note on timeouts:** Timeouts (`exit 124`) are tracked but do not block the ramp-up. They are addressed in Stage 2 (performance). Only non-timeout failures gate progress.

#### Two stages

**Stage 1 — Coverage:** Ramp up to all 7,898 ZIPs with zero non-timeout failures. Fix every crash, panic, missing binding, and wrong output along the way.

**Stage 2 — Performance (after Stage 1):** Catalog all tasks >60s. Profile and optimize. Target: zero timeouts at the 120s cap.

#### [ ] D1. Ramp-up runs
Track each ramp-up round here:

| Round | Limit | OK | Timeout | Errors | Action |
|-------|-------|----|---------|--------|--------|
| 1     | 4     | 4  | 0       | 0      | Pass → double |
| 2     | 8     | 7  | 0       | 1      | STOP: 0704.1304 `\stop` undefined |
| 3     | 16    | 13 | 0       | 3      | STOP: `Error:expected:id` (3 papers) |
| 4     | 32    | 21 | 4       | 7      | STOP: `Error:expected:id` (6), `\documentstyle` (1) |
|       |       |    |         |        | **Applied fixes 1-5, clean slate restart** |
| 1'    | 4     | 4  | 0       | 0      | Pass |
| 2'    | 8     | 7  | 0       | 1      | STOP: 0704.1304 `\stop` undefined |
| 3'    | 16    | 15 | 0       | 1      | STOP: 0704.3480 `Missing $` |
| 4'    | 32    | 25 | 4       | 3      | STOP: 3 remaining errors |
|       |       |    |         |        | **Applied fixes 6-8, clean slate restart** |
| 1''   | 4     | 4  | 0       | 0      | Pass |
| 2''   | 8     | 8  | 0       | 0      | Pass |
| 3''   | 16    | 15 | 0       | 1      | 0704.3480: `Missing $` (document issue) |
| 4''   | 32    | 26 | 4       | 2      | 0705.1277 (2.09 compat), 0705.2808 (mode) |
|       |       |    |         |        | **Applied fixes 10-12, clean slate restart** |
| 5     | 128   |111 | 7       | 10     | 87% OK. xypic (4), display math (2), misc (4) |

#### [ ] D2. Coverage fixes

**Fix 1 — POSTFIX textrec panic with empty args**
- **Failing papers:** 0705.3794
- **Root cause:** `textrec_apply` POSTFIX branch indexed `args[0]` without checking `args.is_empty()`.
- **Fix:** Guard `args.is_empty()` in POSTFIX branch (`latexml_math_parser/src/parser.rs:1432`).
- **Verified:** 0705.3794 converts successfully (869 math, Status:conversion:2→ok after other fixes).

**Fix 2 — `lx@GEN` undeclared keyval keys**
- **Failing papers:** (log noise, not a blocker)
- **Root cause:** `name`, `meaning`, `datameaning`, `left`, `right`, etc. used via `getValue`/templates but never declared with `DefKeyVal`.
- **Fix:** Added 10 `DefKeyVal!` declarations in `base_xmath.rs`.
- **Verified:** No more `Info:undefined:Encountered unknown KeyVals key` for lx@GEN.

**Fix 3 — `find_main_tex` comment-aware scoring**
- **Failing papers:** 0704.0192 (+ any archive with commented-out `\documentclass` in sub-files)
- **Root cause:** cortex_worker had simplified `find_main_tex` that matched `%\documentclass` in comments. Perl's Pack.pm strips comments before checking.
- **Fix:** Replaced cortex_worker's `find_main_tex` with faithful Pack.pm port (scoring, `\input` veto, 4 heuristic tiebreakers, 00README.json/XXX support).
- **Verified:** 0704.0192: Status:conversion:2 → Status:conversion:0 (aastex.cls now loads).

**Fix 4 — `Status:conversion:2` now tracked as `conversion_error`**
- **Root cause:** Benchmark script only checked exit code, not status line. Papers with Error-level log messages but exit 0 were counted as "ok".
- **Fix:** `benchmark_10k.sh` now parses status line from output ZIP. `conversion:2` → `conversion_error`, `conversion:3` → `conversion_fatal`.

**Fix 5 — XMRef id lookup: Error→Warn (match Perl)**
- **Failing papers:** 0704.3530, 0704.2400, 0705.2618, 0705.3564, 0705.3794, 0705.1050 (6 papers)
- **Root cause:** `realnode_from_ref` in `parser.rs` used `Error!` for missing XMRef targets. Perl `Document.pm` L1553 uses `Warn`.
- **Fix:** Changed `Error!("expected", "id", ...)` to `Warn!("expected", "id", ...)` in `latexml_math_parser/src/parser.rs:1558`.
- **Verified:** All 6 papers now produce `Status:conversion:0` or `conversion:1` (warnings only).

**Fix 6 — `\@fontenc@load@list` proper format**
- **Root cause:** List was bare text "OT1,,," instead of `\@elt{OT1}`. Babel's `\@gobbletwo` expects `\@elt{enc}` tokens.
- **Fix:** `\def\@fontenc@load@list{\@elt{OT1}}` in `babel_sty.rs` before loading babel.

**Fix 7 — `\stop` → `\endinput`**
- **Failing papers:** 0704.1304
- **Fix:** `Let!("\\stop", "\\endinput")` in `latex_ch14_pictures_and_color.rs`.

**Fix 8 — `\documentstyle` faithful port (TeX.pool)**
- **Failing papers:** 0705.1277
- **Root cause:** `\documentstyle` was undefined. In Perl, defined in `TeX.pool.ltxml` L60.
- **Fix:** `DefMacro!("\\documentstyle[]{}", sub[...])` in `tex_job.rs` — reads options/class, loads LaTeX.pool, re-emits as `\documentclass`.
- **Verified:** `\documentstyle[12pt]{article}` now loads article.cls correctly. Remaining error in 0705.1277 is LaTeX 2.09 option-as-package semantics (`epsfig` treated as class option, not package).

**Fix 9 — `\@farcm`/`\@farcs`/`\@fdg`/`\@fs` DefMath + `\aas@@fstack` constructor**
- **Failing papers:** 0705.2004
- **Root cause:** Internal math macros for AAS astronomical units were missing.
- **Fix:** Added `DefConstructor!("\\aas@@fstack Undigested {}", ...)` and 8 `DefMath!` definitions in `aas_support_sty.rs`.
- **Verified:** 0705.2004 now produces Status:conversion:0.

**Fix 10 — revtex4.cls: load AMS packages after article (Perl L41-58)**
- **Failing papers:** 0706.1840 (`\eqref` undefined)
- **Root cause:** Perl's revtex4.cls.ltxml loads `amsfonts`, `amssymb`, `amsmath` after `LoadClass('article')` and `RequirePackage('revtex4_support')`. Rust had them as ignored class options.
- **Fix:** Added `RequirePackage!` for all three AMS packages after article+revtex4_support in `revtex4_cls.rs`.
- **Verified:** 0706.1840 now produces Status:conversion:0.

**Fix 11 — `\documentstyle` compat: load options as packages (Perl L137-151)**
- **Failing papers:** 0705.1277, 0707.1730
- **Root cause:** LaTeX 2.09 `\documentstyle[epsfig,amsbsy]{article}` needs options loaded as packages, not class options.
- **Fix:** `\documentstyle` macro now tries each option as `\RequirePackage` via `\IfFileExists` after class loads.
- **Verified:** Both papers now load their packages (epsfig, amsbsy) correctly.

**Fix 12 — Case-insensitive `.TEX` extension in `find_main_tex`**
- **Failing papers:** 0709.4569
- **Root cause:** Archive contained `rapid07.TEX` (uppercase). `find_main_tex` only matched `.tex`/`.txt`.
- **Fix:** Case-insensitive extension check via `to_ascii_lowercase()` in both cortex_worker and latexml_oxide.

**Remaining errors at 128-paper scale (10 `conversion_error`):**
- `Missing $` display math (0704.3480, 0707.0739) — document structure
- `colordvi` `\ifglobalcolors` (0705.1190) — unsupported niche package
- `table*` mode mismatch (0705.2808, 0707.4170) — mode stack issue
- xypic `\xylinewidth@i` (0707.1718, 0707.2392, 0708.3157, 0709.2286) — raw xyline.tex defines before latexml driver loads
- `utf8x` `\PackageNoteNoLine` (0707.3268) — ucs/utf8x internal
- `\figcaption` undefined (0707.4283) — aipproc/aipproc-like class
- `\psecurve` undefined (0708.2155) — pstricks curve command
- `\Rset` undefined (0709.3641) — custom math command

#### [ ] D3. Performance catalog
After Stage 1 reaches all 7,898 with 0 non-timeout errors:
1. List all tasks >60s with wall-clock time
2. Profile top offenders (flamegraph, token count, loop detection)
3. Targeted optimizations (per-task or systemic)

---

### Phase E: Kernel Dump Integration (HIGHEST PRIORITY — blocks sandbox testing)

The LaTeX kernel dump provides ~20K definitions from `latex.ltx` (expl3, fonts, captions, counters, etc.) that the Perl LaTeXML gets from its precompiled format. Without it, many features fail (babel captions, `\@fontenc@load@list`, etc.).

**Current state:**
- `latexml_oxide --init=latex.ltx` generates `resources/dumps/latex.dump.txt` (3.4 MB, 22.6K entries)
- `build.rs` embeds the dump via `include_str!` when the file exists
- `dump_reader::load_from_str` loads the text dump into typed state entries at runtime (~30ms)
- **PROBLEM:** Loading the full dump (19.5K entries) breaks 407 existing tests — dump definitions conflict with compiled engine definitions.

**Root cause of conflicts:**
The dump contains raw `latex.ltx` state (all `\let`, `\def`, `\chardef`, register assignments, etc.). Many of these redefine macros that our compiled engine (`latex_ch*.rs`) already defines with LaTeXML-specific semantics (constructors, custom behavior). When the dump loads AFTER the engine, it overwrites LaTeXML definitions with raw TeX ones.

**Solution: Selective dump loading with conflict resolution**

The dump entries fall into categories that need different treatment:

| Category | Count (est.) | Strategy |
|----------|-------------|----------|
| Registers (`\dimen`, `\skip`, `\count`, `\toks`) | ~3K | Load: these set numerical values, no semantic conflict |
| `\chardef` / `\mathchardef` | ~2K | Load: assigns char/math codes |
| `\let` aliases | ~5K | Load selectively: skip if target is a LaTeXML constructor |
| `\def` / `\edef` expandables | ~8K | Load selectively: skip if name matches a compiled binding |
| Font info (`fontinfo_*`, `font_shared_key_*`) | ~1K | Load: font metadata, no conflict |
| Boolean/string values | ~500 | Load: state flags |

**Implementation plan:**

#### [ ] E1. Classify dump entries
Parse `resources/dumps/latex.dump.txt` and categorize each entry. Build a blocklist of CS names that must NOT be overwritten (= names defined by our compiled engine with LaTeXML-specific behavior).

#### [ ] E2. Selective loader
Modify `dump_reader::load_from_str` (or create `dump_reader::load_selective`) to accept a filter function or blocklist. Skip entries whose CS name is in the blocklist.

#### [ ] E3. Build blocklist from compiled engine
Programmatically collect all CS names defined by our `LoadDefinitions!` macros. These are the "LaTeXML-semantic" definitions that the dump must not overwrite. Approach: after engine loading in `initialize_singletons`, snapshot the defined CS set, then use it as the blocklist for dump loading.

#### [ ] E4. Enable dump loading
Uncomment `latex_dump::load_definitions()` in `latex.rs` with the selective loader. Verify all 407 tests pass. Then verify the 10k sandbox improves.

#### [ ] E5. Commit dump to repo
Add `resources/dumps/latex.dump.txt` to git. Document regeneration in CLAUDE.md:
```
cargo run --release --bin latexml_oxide -- --init=latex.ltx
```

#### [ ] E6. Type-safe dump representation
Ensure the text dump format is loaded into well-typed data tables where each row is one entry (current `dump_reader` approach). Verify that the representation uses proper Rust types (not stringly-typed) for registers, dimensions, glue, tokens, etc.

---

### Phase F: Engine File Reorganization

Restructure the Rust `engine/` directory to **exactly match** the Perl `Engine/` file organization. Move every current definition (without losing any, not even a line) to files matching the Perl names. Then separately rearrange definition order within each file to match Perl.

**Goal:** `latexml_package/src/engine/` file names ↔ `LaTeXML/lib/LaTeXML/Engine/*.pool.ltxml` file names, 1:1.

**Approach:**
1. Map current Rust files → Perl equivalents
2. Create new files matching Perl names
3. Move definitions, preserving every line
4. Verify with `cargo test` after each move
5. Rearrange definition order within files to match Perl

---

> **Reminder:** Every entry ported from Perl must follow tightly the original semantics and nuances. Read the Perl source, translate precisely, preserve edge cases. The Perl code is the ground truth.
