# Engine Sync Status: Perl vs Rust

> **This is a Perl-to-Rust translation project.** Every ported function, macro, and definition must faithfully reproduce the original Perl semantics, control flow, and edge-case behavior. The Perl source (`LaTeXML/` directory) is the ground truth. Only diverge when explicitly documented in `docs/OXIDIZED_DESIGN.md`.

Updated 2026-04-07. Only lists open gaps & TODOs; completed items live in git history.

**Test inventory:** 313+ non-tikz tests pass (90 workspace + 223 integration); 7/10 tikz tests pass (3 pre-existing loops). MakeBibliography pipeline fully operational.

**arxiv sandbox:** 100+ papers in `arxiv-examples/`. **43/47 OK (91%)** on original 47-paper catalog. Remaining 3 EMPTY all fail in Perl too. New 50-paper batch being benchmarked.

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

## Tikz — Known Diffs

1. foreignObject transform Y / width/height
2. Arrow tip shape (different path data)
3. SVG viewBox/width — total dimensions differ slightly
4. 3 tikz tests loop (dominoes, various_colors, unit_tests_by_silviu — pre-existing)

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

#### [ ] C1. Benchmark all 100+ papers — generate Rust vs Perl HTML comparison
**Approach:**
1. Run all papers in `arxiv-examples/` through both Rust and Perl pipelines
2. Record: output size, error count, visual comparison
3. Identify papers with >10% size gap or structural differences
4. Triage into fixable (binding/engine gaps) vs unfixable (deep pgf/tikz infrastructure)
**Deliverable:** Updated CATALOG.md with full 100-paper status table.

#### [ ] C2. Fix high-impact Rust-specific failures
**Approach:**
1. For each paper where Perl succeeds but Rust fails or has significant gaps:
   - Identify the root cause (missing binding, engine bug, parameter type)
   - Port the fix faithfully from Perl
   - Add regression test if applicable
2. Priority: papers with 0KB output → papers with <50% size parity → papers with >10 errors

#### [ ] C3. Upstream Perl sync — continuous
**Approach:**
1. Check `LaTeXML/` git log for new commits
2. Port relevant fixes to Rust (engine, bindings, test files)
3. Update expected XMLs when Perl test output changes

---

> **Reminder:** Every entry ported from Perl must follow tightly the original semantics and nuances. Read the Perl source, translate precisely, preserve edge cases. The Perl code is the ground truth.
