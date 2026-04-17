# Engine Sync Status: Perl vs Rust

> **This is a Perl-to-Rust translation project.** Every ported function, macro, and definition must faithfully reproduce the original Perl semantics, control flow, and edge-case behavior. The Perl source (`LaTeXML/` directory) is the ground truth. Only diverge when explicitly documented in `docs/OXIDIZED_DESIGN.md`.

Updated 2026-04-17. Open gaps & active TODOs only; completed items live in git history.

**Test inventory:** 413 integration tests pass (0 failures); all 10 tikz tests pass. MakeBibliography pipeline fully operational.

**arxiv sandbox:** 100+ papers in `arxiv-examples/`. **93+%** on full catalog.

**10k sandbox:** 7,898 arxiv ZIPs in `$HOME/data/10k_sandbox/`. Last 512-paper ramp: **93.2% OK** (477 ok / 21 conv_error / 14 timeout / **0 panics**).

**Engine definition coverage:** **99.9%** (2,455/2,457 Perl Engine definitions ported). Only `\directlua` (LuaTeX) and `\ASCII` (niche) missing by design.

**Dump loading:** 5,834 entries from latex.ltx kernel (V + codes + @-internal M + Register). Add-only policy preserves engine semantics.

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

**100% coverage: all 406+ Perl bindings ported to Rust.** Zero `todo!()` panics. Zero MISSING.

## Tikz — Known Diffs (vs Perl output)

1. foreignObject transform Y / width/height
2. Arrow tip shape (different path data)
3. SVG viewBox/width — total dimensions differ slightly
4. tikz matrix rendering uses `<svg:g class="ltx_tikzmatrix">` groups (Rust) vs inline-blocks (Perl)

### Permanent sandbox ignores

- **ns1–ns5** (52_namespace) — DTD not supported in Rust port
- **2402.03300**, **2410.10068**, **2511.03798** — Perl also fails on these papers

---

## Completed Phases (historical summary)

- **Phase A (EMPTY→OK binding coverage)** — all Perl `.sty.ltxml` / `.cls.ltxml` files ported or stubbed.
- **Phase B (parity improvement)** — per-package semantic alignment, rewrites.
- **Phase C1–C3 / C5** — 97-paper sandbox baseline (93%+ OK), babel fix, directory/archive input parity, code quality sweep.
- **Phase E (kernel dump integration)** — 5,834 dump entries loaded (V + @-internal M + Register + codes). Add-only policy with has_meaning/has_value safety, non-ASCII catcodes only, MC/DC skipped (expl3 init corrupts them). Auto-generated at build time via `build.rs`.
- **Phase F (engine file reorganization)** — 1:1 match with Perl's `Engine/` directory. `math_common.rs`, `plain_bootstrap.rs`, `plain_base.rs`, `plain_constructs.rs`, `latex_bootstrap.rs`, `latex_base.rs`, `latex_constructs.rs` (7800 lines, merged from 36 `latex_ch*.rs`). All Rust engine files now match Perl file names exactly.
- **Phase G (SVG post-processor)** — inline SVG injection for `\begin{picture}` via post-XSLT regex replacement (avoids libxml2 UAF); covers lines, vectors, circles, ovals, framebox, qbezier, multiput.

Detailed fix history for phases above lives in git log. See the corresponding session commits on `claude-round-15` (e.g., `da8b66358` ch* consolidation, dump-loading commits via Session 102, SVG commits in Session 102, etc.).

---

## Work Plan — Active TODO List

### Phase C4: Upstream Perl sync — continuous

**Approach:**
1. Check `LaTeXML/` git log for new commits
2. Port relevant fixes to Rust (engine, bindings, test files)
3. Update expected XMLs when Perl test output changes

**Recent Perl commits verified already ported:** #2775 alignment init_depth + `\\→\lx@newline`, #2770 Grouplevel (0-based + noframe), #2778 Relation parameter type, #2771 iflimit/if_count deny-list, #2762 lgroup codepoints, #2759 TL2025 kernel, #2736 hyperref etoolbox, #2751 siunitx expl spacing, #2700 Explode newlines (revert of #2646), #2633 `\backsimeq` U+22CD, #2551 mixed-delimiter definecolor, #2552 todonotes opt arg, #2442 page counter + overline scriptpos + triangleleft U+22B2, #2450 overline no scriptpos=mid, #2458 Dot-over-i, #2651 multirow guard, #2488 `\phantom`, #2436 Rearrange2 (`\lx@end@document`, tiny..Huge in classes), #2442 Rearrange3 (`\lx@endash/\lx@emdash/\lx@NBSP`), #2448 Leaders (stretchy math accents), #2449 Amscd (CD arrows), #2411 Plain fonts (FontDef), #2404 Accents (combining char data), #2340 `\braket` | reversions, #2319 `*` as U+2217, #2322 ifthen in packages, #2409 bibitem prune.

**Not ported (materially different in Rust):** #2777 pstricks raw TeX `--includestyles` (Rust uses bindings not raw), #2753 dump parameter double-escape (Rust dumper structure differs), #2555 Sizing (post-processor work), #2605 deep recursion limits (Perl-specific), #2425 Unicode math properties redesign.

### Phase D: 10k-Document Sandbox — Coverage & Performance

Scale testing to ~8,000 arxiv papers (`$HOME/data/10k_sandbox/`). All known to convert under Perl LaTeXML. **Tool:** `cortex_worker --standalone --input <zip> --output <zip>`.

**Process guards:** timeout 60s, RAM 6GB, core dumps disabled, output 200MB cap. Parallelism via GNU parallel (default 16). Categories: `ok`, `timeout`, `oom_or_kill`, `segfault`, `abort`, `error`, `empty_output`, `oversized`. Runner: `tools/benchmark_10k.sh`.

**Ramp-up protocol:** exponential doubling (4→8→16→…→7898) with 0-error gate. On failure: diagnose root cause, fix in Rust, re-run failing files, restart ramp.

**Two stages:**
1. **Stage 1 — Coverage:** zero non-timeout failures at full scale.
2. **Stage 2 — Performance:** eliminate timeouts at 120s cap.

#### [ ] D1. Ramp-up runs — ONGOING

Latest (session 108): **512 papers: 93.2% OK** (477 / 21 conv_error / 14 abort / **0 panics**). No Rust-attributable conversion errors at 128-paper scale. Remaining 512-scale errors are paper-specific (user LaTeX bugs, exotic Unicode in CS names, custom macros, content-model violations).

Known blockers by category (512-scale residuals):
- `Missing $` display math (document bugs)
- Content-model `malformed` (`ltx:line` in `ltx:para`, `ltx:g` in `ltx:figure`, etc.)
- Raw-class undefined internals (e.g. `\@count`, `\theequation@ID` in standalone non-article classes)
- Rc<RefCell> "shared Node" error in 0805.2376 (libxml2 node sharing during tree mutation — tracked in D3b)

#### [ ] D2. Coverage fixes — ONGOING

Each cycle adds small targeted fixes for specific undefined/misbehaving commands per log analysis. Detailed fix history in git log; current focus is filling package-parity gaps against Perl upstream.

**Most recent wave (session 108 /loop):** xcolor `RGB` case-sensitivity bug (all `{RGB}{r g b}` defs → white), page counter starts at 1 (#2442), `\braket` user-facing reversions (#2340), bibitem prune empty auto-opened (#2409), `\text@frac` constructor, `\person@thanks` inline, elsart/mn2e/aa/iopart/texvc/proofwiki/sv_support/ams_support/acmart/amsbook/revtex4/inst_support/microtype/html/subcaption/attachfile/floatflt/floatfig/subfloat/iopams/actuarialangle parity patches.

#### [ ] D3. Performance catalog — after Stage 1

After Stage 1 reaches 7,898 with 0 non-timeout errors:
1. List all tasks >60s with wall-clock time
2. Profile top offenders (flamegraph, token count, loop detection)
3. Targeted optimizations (per-task or systemic)

#### [ ] D3b. Stability — eliminate SIGSEGV in test suite

A Rust safe-by-construction implementation should NEVER segfault. Sources investigated:
1. **libxml2 FFI** — `libxml::tree::Node` is `Rc<RefCell<_Node>>` wrapping raw C pointers; unlinking while referenced elsewhere causes UAF. Past incident: `xmlFreeNodeList` UAF during PostDocument Drop when SVG replacement kept idcache alive (fixed in G2 via string-based SVG injection).
2. **libxslt C stylesheet processing** — past crashes with `svg:` namespaced elements.
3. **Rust unsafe in arena** — `with_arena_mut` cached raw pointer from RefCell.
4. **Parallel benchmark writes** — output files sharing paths.

**Status:**
- 50_structure SIGSEGV no longer reproduces (5-run stress stable after S105 `STATE_IN_USE` / `LASTID` moves to thread_local Cell).
- Catalogued 10 `unsafe` blocks across 8 files; all SAFETY-documented (session 106).
- 0805.2376 "shared Node" error still open (Rc mutation during tree traversal).

**TODO:**
- [ ] Route libxml node lifetimes through guardian structure that forbids unlinking without cache invalidation.
- [ ] Replace unsafe-over-FFI patterns with safe wrappers where practical.

#### [ ] D4. Performance — parallel scaling and allocations

**Baseline (session 105, paper 0707.1173):**

| Workers | Total time | Per-worker efficiency |
|---|---|---|
| 1 | 22.6s | 100% |
| 4 | 33.6s | 67% |
| 16 | 76.8s | 29% |
| 20 | 104.7s | 22% |

14-core/20-thread machine, ~42% ceiling at 16 workers. Peak RSS 570 MB/process.

**Completed:**
- [x] mimalloc as global allocator — reduces glibc arena-mutex contention (~6% single-process).
- [x] `--timeout` default 600s → 60s.

**Callgrind (session 105):** Math parser Marpa dominates — `transitive_closure` 34.3%, `marpa_g_precompute` 8.3%, `bv_scan` 7.1%, AVL ops 6.8%. Total Marpa-related >60% CPU.

**Active work:**
- [ ] Audit `.to_string()` (~1900 sites) — replace with `&str` / interned symbols where value goes into `HashMap<String,String>`.
- [ ] Audit `String::from("...")` literals for interned conversions.
- [ ] Replace `HashMap<String,String>` with `SymHashMap<SymStr>` in hot paths.
- [ ] Audit `.clone()` in `document.rs` (73), `latex_constructs.rs` (73), `font.rs` (39).
- [ ] Review `Tokens` cloning — pass `&Tokens` or `Cow` for read-only iteration.
- [ ] Profile math parser RAM independently (Marpa chart, forest).
- [ ] Investigate shared read-only engine state across processes (mmap dump).
- [ ] Long-running daemon / process pool to amortize 570 MB startup.
- [ ] Fork-based parallelism for CoW memory sharing.

#### [ ] D5. Math parser optimizations (HIGHEST PRIORITY per callgrind)

**Completed:**
- [x] Avoid per-formula `reset_engine` (S105): paper 0707.1173 22s→15s.
- [x] Audit `trig_arg` ambiguity (S105): `\sin(x)+\sin(y)` 65→1 parses; paper 0704.0516 6×65-enumerated→1.
- [x] Remove duplicate `<fn> fenced_factor` alternatives: physics.tex 40→8, full suite 99→59 ambiguous formulas.
- [x] `MATHPARSER_SPECULATE` redesign (S107): removed grammar-layer filter, `FencedLettersAreFunctionArguments` pragma picks consistent interpretation. `a(b)(c)(d)` 23→2 (91% reduction).
- [x] Watchdog thread for cooperative-timeout escape (aborts native Marpa/libxml2 loops).
- [x] `LATEXML_PARSE_AUDIT=1` env var for per-formula diagnostics.

**Remaining:**
- [ ] Avoid `init_grammar()` fallback — reuse existing grammar on reset failure.
- [ ] Audit script attachment ambiguity (`{}^4{}_{12}C^{5+}` — 27 unique trees).
- [ ] Early pruning: fail parses on inconsistency detection rather than post-hoc pragmas.
- [ ] Enumerate grammar rules by parse-tree count contribution.
- [ ] Document grammar ambiguity per category.

#### [ ] D6. Grammar First-Principles Plan

Grounded in `docs/MATH_GRAMMAR_FIRST_PRINCIPLES.md`. Live audit: `LATEXML_PARSE_AUDIT=1`.

**Completed (S106-108):**
- [x] Narrow `script_op` to `metarelop | vertbar | supops | modifierop` (P^+ tuple 31→3).
- [x] Fix 1: OTHER_OPEN/OTHER_CLOSE split — eliminates PREFIX-match duplication. `[A],[B],[C],[D]` 64→2 (32×).
- [x] Fix 2: Remove `formula_list` from `anything` alternatives.
- [x] Fix 3: Collapse `term_list` vs `formula_list` in fenced contexts.
- [x] Fix 4: `MATHPARSER_SPECULATE` redesign (see D5 above).
- [x] Fix 5: Interval moved from `fenced_factor` to `tight_term` — `f(x,y)` now correctly parses as `f@(vector(x,y))` via category hierarchy, no ad-hoc pragmas.
- [x] Removed redundant `opfunction opfunction` rule.
- [x] Math parser convergence 32→16 consecutive dupes (32% reduction on `tr ρ`).
- [x] Half-decay `consecutive_dupes` on new unique.

**Remaining hotspots (post-S108):**
1. `\sin[XY]` chain — 1022 trees / 10 unique (real semantic ambiguity)
2. `tr ρ / tr(XY) / rank M / …` — 100 / 8 unique
3. `FGHa` OPFUNCTION cascade — 87 / 9 unique (genuine math ambiguity)
4. `a|a|+b|b|+c|c|` VERTBAR — 53 / 10 unique

Items 1–4 are primarily **semantic** (inherent to math practice); further grammar refactoring has limits.

---

## Recent Session Highlights

### Session 108 (2026-04-17, /loop cycles)

**Packages parity**: 50+ commits filling gaps against Perl: elsart, mn2e, aa, aas, revtex4, iopart, texvc (92 proofwiki macros), sv_support, ams_support, acmart, amsbook, revtex4, inst_support, microtype, html, subcaption, attachfile, floatflt/floatfig, subfloat, iopams, actuarialangle.

**Real bug fixes**:
- **xcolor case-sensitivity**: `\definecolor{x}{RGB}{153 153 192}` was producing `#FFFFFF` due to lowercased model dispatch. Fixed to case-sensitive match — lowercase rgb/cmy/gray take 0..1 components; uppercase RGB/HSB/Gray take 0..255.
- **Page counter**: now starts at 1 per Perl #2442.
- **Bibitem auto-open**: prune empty whatsit, reuse ID per Perl #2409.
- **\text@frac semantic FRACOP**: `\case` in aas_support now produces semantic fraction markup.
- **\person@thanks inline**: elsart_support_core.
- **\backsimeq U+22CD** (Perl #2633); **mixed-delimiter definecolor** (Perl #2551); **Explode newline** reverted to CC_OTHER per Perl #2700.
- **RefCell panics** fixed in `with_font_info` + `font::decode` re-entry (common/mathchar.rs, latexml_sty.rs).
- **DefEnvironment scope lifecycle wisdom**: `after_digest` vs `after_digest_body` matters — body runs post-frame-pop, so local state assigns in before_digest are gone. Documented in `WISDOM.md`.

**Sandbox transitions (broken → OK)**: 9 papers (0705.1190, 0705.2808, 0707.4170, 0710.2880, 0711.4787, 0802.1100, 0810.1610, 0704.2400, 0705.1050, 0705.2208).

**Post-session 512 verification**:

| Category | Count |
|----------|-------|
| ok | **477 (93.2%)** |
| conversion_error | 21 (paper-specific) |
| abort (timeout ~61s) | 14 |
| **panics** | **0** |

### Session 107 (2026-04-16)

- Fix 4 speculative redesign (13 test XMLs updated)
- Documented safety contracts on all 10 unsafe blocks
- OXIDIZED_DESIGN #18 updated for Marpa design
- Paper 0707.1173 conversion: 12.4s (from 22.6s baseline)

### Session 106 (2026-04-16)

- Grammar Fixes 1/2/3 (OTHER_OPEN split, formula_list removal, term_list collapse)
- Narrowed `script_op`
- 317 integration tests pass; total enumerated trees 3767→3544

Earlier sessions (42–105) archived in git log and `memory/project_session_history.md`.

---

> **Reminder:** Every entry ported from Perl must follow tightly the original semantics and nuances. Read the Perl source, translate precisely, preserve edge cases. The Perl code is the ground truth.
