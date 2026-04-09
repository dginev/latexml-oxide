# Engine Sync Status: Perl vs Rust

> **This is a Perl-to-Rust translation project.** Every ported function, macro, and definition must faithfully reproduce the original Perl semantics, control flow, and edge-case behavior. The Perl source (`LaTeXML/` directory) is the ground truth. Only diverge when explicitly documented in `docs/OXIDIZED_DESIGN.md`.

Updated 2026-04-08. Only lists open gaps & TODOs; completed items live in git history.

**Test inventory:** 407 tests pass (0 failures); all 10 tikz tests pass. MakeBibliography pipeline fully operational.

**arxiv sandbox:** 100+ papers in `arxiv-examples/`. **90/97 OK (93%)** on full catalog. 7 remaining: 3 Perl-also-fails, 2 timeout, 1 version conflict, 1 state corruption.

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

> **Reminder:** Every entry ported from Perl must follow tightly the original semantics and nuances. Read the Perl source, translate precisely, preserve edge cases. The Perl code is the ground truth.
