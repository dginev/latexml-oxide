# Engine Sync Status: Perl vs Rust

> **This is a Perl-to-Rust translation project.** Every ported function, macro, and definition must faithfully reproduce the original Perl semantics, control flow, and edge-case behavior. The Perl source (`LaTeXML/` directory) is the ground truth. Only diverge when explicitly documented in `docs/OXIDIZED_DESIGN.md`.

Updated 2026-04-04. Only lists open gaps & TODOs; completed items live in git history.

**Test inventory:** 407 tests pass (338 integration + 1 post + 39+7+6+15 latexml_post unit tests + 1 post integration). All integration tests zero-diff against Rust reference XMLs. Perl reference parity: 246/314 effective zero-diff (78%), ~18K meaningful diff lines across 68 non-zero tests. Top diff sources: siunitx (3.5K), SVG/tikz (4.3K), beamer (1.2K), physics (1.2K).

**arxiv sandbox:** See [`arxiv-examples/CATALOG.md`](../arxiv-examples/CATALOG.md) for the full 48-paper test catalog with per-paper status, errors, and visual comparison results.

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
| base_xmath.rs | MINOR | Missing: `MathWhatsit()` |
| tex_box.rs | MINOR | Minor box dimension edge cases |
| tex_fonts.rs | MINOR | Missing: `\fontdimen` full array semantics, `getFontDimen()` helper |
| tex_tables.rs | MINOR | Minor: padding CSS classes |
| latex_ch4_sectioning_and_toc.rs | MINOR | Missing: `LABEL_MAPPING_HOOK` |

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

## Work Plan — arxiv Visual Parity

Follow the [`arxiv-examples/CATALOG.md`](../arxiv-examples/CATALOG.md) for per-paper status.

### Permanent ignores (5)
- **ns1–ns5** (52_namespace) — DTD not supported in Rust port.

---

> **Reminder:** Every entry ported from Perl must follow tightly the original semantics and nuances. Read the Perl source, translate precisely, preserve edge cases. The Perl code is the ground truth.
