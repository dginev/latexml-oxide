# Engine Sync Status: Perl vs Rust

> **This is a Perl-to-Rust translation project.** Every ported function, macro, and definition must faithfully reproduce the original Perl semantics, control flow, and edge-case behavior. The Perl source (`LaTeXML/` directory) is the ground truth. Only diverge when explicitly documented in `docs/OXIDIZED_DESIGN.md`.

Updated 2026-04-05. Only lists open gaps & TODOs; completed items live in git history.

**Test inventory:** 407 tests pass. Perl reference parity: 214/298 zero-diff (72%), ~28K diff lines across 84 non-zero tests. MakeBibliography wired (Scan→MakeBib→CrossRef). Top diff sources: siunitx (3.5K), SVG/tikz (4.3K), beamer (1.2K), physics (1.2K), math parser (2K).

**arxiv sandbox:** See [`arxiv-examples/CATALOG.md`](../arxiv-examples/CATALOG.md) for the full 47-paper test catalog with per-paper status, errors, and visual comparison results.

**Production-ready:** Full CorTeX ZIP-to-ZIP pipeline operational. All legacy production options supported:
```
latexml_oxide --whatsin=archive --format=html5 --pmml --mathtex --noinvisibletimes \
  --nodefaultresources --nobibtex --preload=ar5iv.sty --timeout=2700 --log=log.txt \
  --dest=output.zip input.zip
```

**High-level roadmap:** See [`mini_3_plan.md`](mini_3_plan.md) for the 4-phase strategic plan
(Engine Parity → Package Bindings → Post-Processing → Production).

## Legend
- **OK** = fully synced | **MINOR** = small gaps | **GAPS** = significant missing | **EMPTY** = not ported

**See also:** [`KNOWN_PERL_ERRORS.md`](KNOWN_PERL_ERRORS.md) — upstream Perl issues (not Rust bugs)

---

## Engine Files — Open Gaps Only

Only files with GAPS or significant MINOR issues listed. OK files omitted (see git history).

| File | Status | Open Gaps |
|------|--------|-----------|
| base_parameter_types.rs | MINOR | `DirectoryList`, `CommaList`, `DigestUntil` stubbed; `Variable` reversion safe fallback |
| tex_box.rs | MINOR | Minor box dimension edge cases |
| tex_fonts.rs | MINOR | Missing: `\fontdimen` full array semantics, `getFontDimen()` helper |
| tex_tables.rs | MINOR | Minor: padding CSS classes |

---

## Cross-Cutting Infrastructure Gaps

1. **`FontDef` parameter type** — Simplified to `FontToken`. Blocks `\fontdimen`, `\hyphenchar` per-font tracking.

---

## Unported Perl Engine Files

| File | Defs | Status | Notes |
|------|------|--------|-------|
| `AmSTeX.pool.ltxml` | 112 | ~30% | Plain TeX format (rare) |
| `BibTeX.pool.ltxml` | 956 | 0% | Skipped via `--nobibtex` in production |

---

## Core Modules (MINOR+ only)

| Module | Status | Open Gaps |
|--------|--------|-----------|
| gullet.rs | MINOR | `readArg` isolation (type ergonomics) |
| document.rs | MINOR | XML comment creation (needs libxml2 FFI) |

---

## Package.pm — DefFoo Sync Status (dialect.rs)

| DefFoo | Status | Gaps |
|--------|--------|------|
| `DefMacroI` | MINOR | `outer`/`long` not mapped |
| `DefPrimitiveI` | MINOR | Missing `outer`/`long` |
| `DefConstructorI` | MINOR | Missing `outer`/`long`/`attributeForm`; robust alias fallback |

---

## Package Bindings

**100% coverage: all 406 Perl bindings ported to Rust.** Zero `todo!()` panics. Zero MISSING.

### Remaining gaps in ported bindings

| Binding | Gap | Notes |
|---|---|---|
| beamer.cls | 88% | Overlay specs, themes — largest gap (unused by arxiv test papers) |
| authblk/inst_support | callbacks | `relocateInstitute`/`authblkRelocateAffil` DOM surgery (no test regression) |

---

## Tikz Test References

XML files in `LaTeXML/t/tikz/` are OUTDATED. Always regenerate fresh Perl output.

### Priority FIX items (shared across tikz tests)

1. **foreignObject transform Y=16.6** — Perl uses fixed 12pt maxy; Rust uses actual height
2. **foreignObject width/height** — `fo_get_size` differs from Perl
3. **Nested minipage/SVG sizing** — `appendNodeBox` vs Perl's `pushContent`
4. **Arrow tip shape** — Different arrowhead path data
5. **SVG viewBox/width** — Total dimensions differ slightly
6. **Listings escapechar + color** — `escapechar=@` with `\color{red}` inline
7. **Missing `\vspace{2mm}` output** — `\vspace` in vertical mode

---

## Work Plan — Ordered TODO List

Follow the [`arxiv-examples/CATALOG.md`](../arxiv-examples/CATALOG.md) for per-paper status.

**Current status (2026-04-05):** 37/47 OK (79%), 24/37 >=90% (65%), 29/37 >=80% (78%).
**Visual comparison (2026-04-05):** 11/37 IDENTICAL on first page, 8 p1-OK, 7 cosmetic, 6 functional, 3 critical.

**Session 90 fixes (2026-04-05):**
1. `input_definitions()` search order — versioned package fallback before raw TeX. Banners eliminated in 2306.00809 + 2506.03074. 2405.19425 now uses neurips.sty binding.
2. `\pgfmathsetlength` override — prevents `\pgfmath@` delimiter cascade (A2 papers: 1001→45 errors). Zigzag decoration path now matches Perl.
3. `\pgfmath@smuggleone` override — proper scope smuggling for expandable definitions.
4. Visual comparison: 5/6 issues were FALSE POSITIVES (stale CSS, upstream Perl bug). Only real bug: `authblkRelocateAffil` DOM surgery not ported (2511.14458).
5. Stale Perl HTML regeneration with correct flags (`--nodefaultresources` + ar5iv CSS).

---

### Phase A: Get 10 EMPTY/FAIL papers to produce HTML (37→47 OK)

Papers grouped by shared root cause, ordered by impact (most papers fixed per task):

#### [ ] A1. PGF arrows.meta library (3 papers → OK)
**Papers:** 2209.14198, 2402.10301, 2410.10068
**Root cause:** `arrows.meta` library defines arrow tips (Stealth, Computer Modern Rightarrow, Hooks, Implies) via pgfkeys declarations. The `\pgfarrowsdeclarealias`, `\pgfarrowsdeclare`, and `\pgfkeys{/pgf/arrow keys/...}` machinery is not ported. First error: `Unknown arrow tip kind 'Computer Modern Rightarrow'`.
**Approach:**
1. Read Perl `tikzlibraryarrows.meta.code.tex.ltxml` (the LaTeXML binding, NOT raw TeX)
2. Port arrow tip declaration infrastructure: `\pgfdeclarearrow`, `\pgfarrowsdeclarealias`
3. Port the ~20 standard arrow tips: Stealth, Computer Modern Rightarrow, Hooks, Implies, Bar, Bracket, Parenthesis, Ellipsis, Kite, Latex, Triangle, Circle, Square, Diamond, Turned Square, Rays, Arc Barb, Tee Barb
4. Each tip needs: `setup code` (dimensions), `drawing code` (SVG path), `defaults` (pgfkeys)
5. Key file: `latexml_package/src/package/pgf/` — create `arrows_meta.rs`
6. Test: 2209.14198 (simplest), then 2410.10068 (tikz-cd), then 2402.10301 (complex)
**Estimate:** High complexity. This is the single highest-impact task (3 papers + unblocks tikz-cd for 2602.18719).

#### [ ] A2. PGF text node boxing (2 papers → OK)
**Papers:** 2005.13625, 2103.01205
**Root cause:** `\pgfmath@` undefined during pgf text-in-picture processing. When pgf tries to compute text bounding boxes via `\pgf@text@options` → `\pgfmath@`, the expansion fails because the pgfmath text-width calculation path is not ported. This causes a cascade of group-mismatch errors (1000+) or timeout.
**Approach:**
1. Trace the Perl execution path: `\pgfinterruptpicture` → `\pgftext` → `\pgf@text@options` → `\pgfmathparse` → what `\pgfmath@` is
2. The issue is likely that `\pgfmath@` is an internal alias set during pgfmath initialization that our port misses
3. Check if `pgfmathutil.code.tex` sets `\pgfmath@` — it may be a `\let\pgfmath@=\pgfmathparse` or similar
4. Fix: ensure pgfmath initialization sets the required internal aliases
5. The timeout (2103.01205) should resolve once errors stop cascading
**Estimate:** Medium complexity. Likely a missing `\let` alias, not deep infrastructure.

#### [ ] A3. PGF keys filter recursion (1 paper → OK)
**Papers:** 2402.03300
**Root cause:** `\pgfkeys@mainstop` expands into itself recursively (2650 errors → token limit 30M). The pgfkeys filter machinery from `pgfkeyslibraryfiltered.code.tex` uses `\pgfkeys@mainstop` as a sentinel token, but our expansion engine treats it as a regular expandable macro instead of stopping.
**Approach:**
1. Read `pgfkeyslibraryfiltered.code.tex` — understand the filter/handler pattern
2. The sentinel `\pgfkeys@mainstop` should be a `\def\pgfkeys@mainstop{\pgfkeys@mainstop}` (self-referential, caught by `\ifx` comparison, never actually expanded)
3. Check if our code is expanding past `\ifx` comparisons — the likely bug is in conditional evaluation where `\ifx\pgfkeys@mainstop\token` fails to short-circuit
4. Also needs: `datetime.sty` stub (minor — just define `\newdateformat` as no-op)
**Estimate:** Medium complexity. Likely a conditional evaluation edge case.

#### [ ] A4. smfart.cls parameter consumption (1 paper → OK)
**Papers:** 2507.23241
**Root cause:** `smfart.cls` (French math journal class) uses `\mathfrak` in text mode during class initialization, then hits parameter consumption bugs (`<Token> found None`). The class is loaded as raw TeX.
**Approach:**
1. Check if smfart.cls has a LaTeXML binding in Perl — if yes, port it; if no, create stubs
2. Perl handles this with 27 warnings but produces output — likely via error recovery
3. The `<Token> found None` is a parameter-reading crash — may need safe fallback in `readArg`/`readOptional` (return None instead of panic)
4. Alternative: create `smfart.cls.ltxml` binding that loads amsart as base (smfart is similar to amsart)
**Estimate:** Low-medium complexity. A binding stub may suffice.

#### [ ] A5. Stale Perl HTML regeneration
**Root cause:** `generate_all.sh` caches Perl HTML. Existing Perl files were generated with different flags (no `--nodefaultresources`, no ar5iv CSS). This caused 3 false positives in visual comparison.
**Approach:**
1. Delete all cached `perl.html` files: `rm html/arxiv-examples/*/perl.html`
2. Re-run `./arxiv-examples/generate_all.sh perl` with correct flags
3. Retake Perl screenshots
4. Re-run visual comparison — expect dark theme and \MakeUppercase diffs to disappear
**Estimate:** Low complexity. Infrastructure fix.

#### Permanent ignores (4 papers — both Perl and Rust fail)
- **2508.15260** (tcolorbox/minted): minted.sty "listing" parameter type not implemented. Perl: 101 errors + fatal. Only 1KB output.
- **2511.03798** (eqnarray): `\@@eqnarray` recursion in jheppub.sty. Perl: 101 errors + fatal.
- **2603.14602** (minted): Same minted parameter type. Perl: LaTeXML dies.
- Note: 2402.03300 Perl also crashes with different error but produces some output.

---

### Phase B: Improve 37 OK papers toward full parity

Ordered by number of papers affected:

#### [ ] B1. convertBibliography() — 7 papers (59-83% → ~95%)
**Papers:** 1502.04955, 2306.06628, 2401.18036, 2511.11713, 2511.15304, 2512.16911, 2603.19312
**Root cause:** Rust's MakeBibliography is wired (Scan→MakeBib→CrossRef) but `convertBibliography()` — the function that parses raw `.bib` files into `<ltx:bibitem>` XML — is not ported. All citation references show as raw bibkeys (`[author2024]`) instead of resolved numbers (`[1]`).
**Approach:**
1. Port `convertBibliography()` from Perl `LaTeXML::Post::MakeBibliography` (~200 lines)
2. BibTeX entry parsing: author, title, year, journal fields → `<ltx:bibblock>` elements
3. Citation resolution: `\cite{key}` → `<ltx:ref>` with `href="#bib.bibN"` and numeric tag
4. Key functions: `convertBibEntry`, `postProcessBib`, `formatBibEntry`
5. Test with 1502.04955 (simplest — elsarticle with .bib) first
**Estimate:** High complexity. Core bibliography pipeline. Single highest-impact parity improvement.

#### [x] B2. authblkRelocateAffil DOM surgery — 1 paper (2511.14458 affiliations)
**Root cause:** `authblk.sty` binding missing `afterClose` callback that relocates `<ltx:note role="affiliationtext">` into each author's `<ltx:contact>` element.
**Approach:**
1. Port `authblkRelocateAffil` from Perl `authblk.sty.ltxml` lines 71-91
2. Use existing DOM surgery infrastructure (`findnodes`, `unlink`, `append_clone`, `set_attribute`) — same pattern as `relocate_footnote` in `latex_functions.rs:154-196`
3. Add `Tag!("ltx:document", after_close => ...)` in `authblk_sty.rs`
4. Also fix: `\lx@split@authormark` comma splitting for multi-affiliation authors
5. Also fix: `\affil` afterDigest for auto-incrementing mark counter
**Estimate:** Medium complexity. Infrastructure exists; just need to wire it.

#### [ ] B3. Listing per-word styling — 1 paper (2405.19425: 50% → ~80%)
**Root cause:** Perl wraps each listing token in styled `<span>` elements with language-specific keyword coloring. Rust outputs plain text blocks.
**Approach:**
1. Review `listings.sty.ltxml` keyword styling pipeline in Perl
2. Port `lstClassBegin`/`lstClassEnd` token wrapping: each identifier/keyword gets `<ltx:text class="ltx_lst_*">` wrapper
3. Language keyword databases already loaded (C, Pascal, TeX, Perl) — need to emit `<span>` during tokenization
4. Key: the `lst@token` accumulator and `\lst@saveDef` classification
**Estimate:** Medium complexity.

#### [ ] B4. \shortstack/\vtop mode cascade — 1 paper (2508.18544: 43% → ~70%)
**Root cause:** `\shortstack` inside certain contexts (DefConstructor bounded+mode interaction) produces cascading mode errors. Related to `\vtop` mode vs vertical mode.
**Approach:**
1. Trace 2508.18544 errors — identify specific `\shortstack` instances that fail
2. Check Perl `\shortstack` mode: does it use `restricted_horizontal` or `text`?
3. Session 88 already fixed `\shortstack` mode (text→restricted_horizontal) — check if remaining errors are from a different source
4. May need `\vtop` mode restoration after `\shortstack` closes
**Estimate:** Low-medium complexity.

#### [ ] B5. tikzpicture mode corruption — 1 paper (2603.15617: 3% → ~60%)
**Root cause:** A failed tikz command corrupts the parser mode state, causing all subsequent content to be lost.
**Approach:**
1. Run 2603.15617 with verbose logging — find which tikz command fails
2. Check mode stack before/after the failure point
3. Likely fix: save/restore mode state around tikzpicture environments (guard pattern)
4. Related to pgf text boxing (A2) — fixing A2 may partially fix this
**Estimate:** Medium complexity. Depends on A2.

#### [ ] B6. tikz-cd for 2602.18719 (6% → ~80%)
**Depends on:** A1 (arrows.meta). Once arrow tips work, tikz-cd diagrams should render.
**Approach:**
1. After A1, re-run 2602.18719 and assess remaining errors
2. tikz-cd's `\tikzcdmatrixname` and `\halign` processing may need fixes
3. tikz-cd creates matrix-style layouts with arrow decorations between cells
**Estimate:** Medium complexity. Largely unblocked by A1.

---

### Permanent ignores (regression tests)
- **ns1–ns5** (52_namespace) — DTD not supported in Rust port.

### Permanent ignores (arxiv papers — Perl also fails)
- **2508.15260** — tcolorbox + minted cascading. Perl output: 1KB.
- **2511.03798** — jheppub eqnarray recursion. Perl: 101 errors + fatal.

---

> **Reminder:** Every entry ported from Perl must follow tightly the original semantics and nuances. Read the Perl source, translate precisely, preserve edge cases. The Perl code is the ground truth.
