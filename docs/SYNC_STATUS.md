# Engine Sync Status: Perl vs Rust

> **This is a Perl-to-Rust translation project.** Every ported function, macro, and definition must faithfully reproduce the original Perl semantics, control flow, and edge-case behavior. The Perl source (`LaTeXML/` directory) is the ground truth. Only diverge when explicitly documented in `docs/OXIDIZED_DESIGN.md`.

Updated 2026-04-18. Open gaps & active TODOs only; completed items live in git history.

**Test inventory:** 409 integration tests pass (0 failures, 0 ignored); all 10 tikz tests pass. MakeBibliography pipeline fully operational.

**arxiv sandbox:** 100+ papers in `arxiv-examples/`. **93+%** on full catalog.

**10k sandbox:** 7,898 arxiv ZIPs in `$HOME/data/10k_sandbox/`. Last 512-paper ramp: **93.2% OK** (477 ok / 21 conv_error / 14 timeout / **0 panics**).

**Engine definition coverage:** **99.9%** (2,455/2,457 Perl Engine definitions ported). Only `\directlua` (LuaTeX) and `\ASCII` (niche) missing by design.

**Dump loading:** 25,172 entries on disk; 6,154 installed into state at load time (19,018 skipped — `@`-internal gate for M, add-only skip for V/R/CD/LC/UC/SC already-defined by `_base.rs`). Add-only policy preserves engine semantics.

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

**100% coverage: all 406+ Perl bindings ported to Rust.** Zero MISSING. Three `todo!()` remain as deliberate invariant asserts on unreachable branches (be_absorbed for Alignment; get_meta for Ref/Arg XM variants).

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
- **Phase E (kernel dump integration)** — 25,172 entries serialized (latex.ltx kernel + expl3 state + register snapshot); 6,154 installed into state at load time (others shadowed by `_base.rs`'s closures via add-only policy). Gated to `@`-internal M entries, non-ASCII catcodes, Register + CharDef + LCode/UCode/SCode; MC/DC skipped (expl3 init corrupts them). Format v2: E-entries carry a 5th `<proto>` field so DefToken/Optional/Until/Match parameter types round-trip. Auto-generated at build time via `build.rs`.
- **Phase F (engine file reorganization)** — 1:1 match with Perl's `Engine/` directory. `math_common.rs`, `plain_bootstrap.rs`, `plain_base.rs`, `plain_constructs.rs`, `latex_bootstrap.rs`, `latex_base.rs`, `latex_constructs.rs` (7800 lines, merged from 36 `latex_ch*.rs`). All Rust engine files now match Perl file names exactly.
- **Phase G (SVG post-processor)** — inline SVG injection for `\begin{picture}` via post-XSLT regex replacement (avoids libxml2 UAF); covers lines, vectors, circles, ovals, framebox, qbezier, multiput.

Detailed fix history for phases above lives in git log. See the corresponding session commits on `claude-round-15` (e.g., `da8b66358` ch* consolidation, dump-loading commits via Session 102, SVG commits in Session 102, etc.).

---

## Work Plan — Active TODO List

### D0. Raw-binding fidelity — COMPLETE

**Status (2026-04-18):** `tests/babel/page545` passes (un-ignored, commit
96d4bfbe4); three landmark items closed in session 110:
- `\openin`-based `.ini` loading works (latin-1, cyrillic via T2A).
- `\initiate@active@char` active-char lifecycle works end-to-end.
- `AtBeginDocument` hook chain ordering fixed (commit 56b0c35d2 — root
  cause was `@currname` leakage from plain `\input`).

Language registers (`\l@english`, `\l@german`, …) all carried in the
kernel dump (108 `\l@<lang>` entries, regenerated from ambient texlive
via `tools/make_formats.sh`). Dump is .gitignored; CI runs
`make_formats.sh` before tests so the runtime dump always matches
the test-runtime texlive.

**Historical work on mutual-exclusive `LoadFormat`** (Perl-style
`bootstrap+dump+constructs` XOR `bootstrap+base+constructs` branching)
has been **abandoned**. Our Rust port's `_base.rs` runs in ~3-5 ms of
compiled code; skipping it saves no meaningful time and risks losing
closure-backed defs. The unified load order
(`bootstrap → _base → dump → _constructs`) always all four is simpler
and correct. The `LATEXML_NODUMP=1` env var (Perl-parity opt-out) is
still honored; the short-lived `LATEXML_DUMP_ONLY=1` experiment was
removed in commit 4a9e213d5. See `memory/project_load_order_design.md`
for the authoritative design note.

**v3 structured Parameter encoding** (commits 3e1f89eb2, 0be9641bf)
**stays landed** — independent of mutex-mode, it keeps
`Until:\end{verbatim}` and `Match:...` delimiter tokens round-tripping
through the dump correctly. See `docs/DUMP_FORMAT_PERL_ANALYSIS.md` for
the Perl dumper analysis that motivated it.

**Closure-backed primitive relocations** (commits 76569b75f,
42dac16dc): 10 CSes moved from `latex_base.rs` to `latex_constructs.rs`
matching Perl's source layout (`\makeatletter`, `\makeatother`,
`\@ifnextchar`, `\Package{Error,Warning,Info}`, `\Generic{Error,…}`,
`\@onlypreamble`, `\@setsize`, `\fontsize`, `\check@mathfonts`,
`\@setfontsize`, `\kernel@ifnextchar`, `\@ifnext`). Our closure
distribution now matches Perl's pattern: ~2 in `_base`, ~100 in
`_constructs`.

**Babel Rust bindings** slimmed 405 → 62 lines (85%) in session 110
via relocation to per-language ports (`french_ldf.rs`, `german_sty.rs`,
`english_sty.rs`, `ngerman_sty.rs`). Remaining ~100 lines are
rationalized engine-gap workarounds.

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

**Measured impact of session 110 perf micro-optimizations (2026-04-18):**
Six commits eliminated hot-path `String` allocations in the MathML
post-processing pipeline (`adjust_spacing`, `adjust_pair`,
`is_invisible_op` via new non-allocating variant, + introduced
`PostDocument::is_qname` to replace 10 `.as_deref() == Some("ltx:X")`
sites across `document.rs`, `open_math.rs`, `unicode_math.rs`,
`mathml/presentation.rs`, `mathml/content.rs`). Re-ran 3 previously-
timed-out papers from the 512-sample set:

| Paper | Formulas | Pre-S110 | Post-S110 (60s cap) | Post-S110 (300s cap) |
|---|---|---|---|---|
| 0704.2334 | 1,550 | TIMEOUT | **34.9s OK** | — |
| 0706.0243 | 3,508 | TIMEOUT | **55.0s OK** | — |
| 0705.1522 | 4,416 | TIMEOUT | still TIMEOUT | **85.8s OK** |

So 2 of 14 previously-timed-out papers now complete within the 60s cap,
and at least 1 more is under Stage 2's proposed 120s cap.

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
- [x] `pin!(literal)` macro for call-site-cached `SymStr` interning
  (~10× faster than `pin_static` for repeated state-key lookups).
  Sym-keyed state API (`lookup_bool_sym`, `lookup_string_from_sym`,
  `assign_value_sym`, `with_value_sym`, `with_stacked_values_sym`,
  `lookup_token_sym`, `lookup_mathcode_sym`) take `SymStr` by value.
  Removed all 29 `XXX_SYM` pre-pinned constants from arena.rs —
  call sites now write `pin!("key")` inline.
- [x] `extend_from_slice(tokens.unlist_ref())` pattern for Tokens —
  skips Vec<Token> clone+move at 30+ hot sites across engine / package
  bindings. `ArgWrap::unlist_cow()` for keyval-arg sites.
- [x] Token text comparison via `t.text == pin!("...")` instead of
  `t.to_string() == "..."` — avoids per-compare String allocation.
- [x] `Tokens::untex` — O(n²) prepend loop fixed; `VecDeque::from(Vec)`
  reuses heap buffer instead of `.into_iter().collect()`.
- [x] Token constructor macros (`T_CS!`, `T_LETTER!`, `T_OTHER!`,
  `T_SPACE!`, `Token!(lit)`) route the literal arm through `pin!` —
  these macros are invoked hundreds of times per document in macro
  expansion / argument packing / template construction, and each
  previously paid an arena RefCell borrow + hash probe.
- [x] Math parser `parser.rs` ltx:XM* dispatch, `gullet::read_float` /
  `read_factor` / `read_optional_signs` number-scan loops, and
  `document.rs` / `alignment.rs` / `document/helpers.rs` qname probes
  all switched from `arena::pin_static("lit")` to `pin!("lit")`.
- [x] `assign_value_inplace_sym` sym-keyed variant — route MODE-switch
  call sites (enter_horizontal, leave_horizontal, par end) to it.

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

Kept compact — full per-session detail lives in git log and
`memory/project_session_history.md`.

- **Session 111 (2026-04-18)**: dump format v3 (structured Parameter
  encoding writer/reader + 6 roundtrip tests); mutual-exclusivity
  experiment (`LATEXML_DUMP_ONLY=1`) built up to 414/415 then
  **abandoned** in favour of unified `bootstrap → _base → dump →
  _constructs` load order; closure-backed primitive relocations for
  Perl-parity (`\makeatletter`, `\@ifnextchar`, `\fontsize`, …);
  hot-path allocation removals (per-char String in `font.rs`,
  METRIC_MAP format!, namespace prefix check in `document.rs`,
  Tokens clones in dump_writer).
- **Session 110 (2026-04-17)**: D0 FUNCTIONALLY COMPLETE via
  `@currname` leakage fix (commit 56b0c35d2); babel_sty.rs cut 405 →
  62 lines; three D0 sub-items closed (`.ini` loading, active-char
  lifecycle, AtBeginDocument ordering).
- **Session 109 (2026-04-17)**: `page545_test` un-ignored; root cause
  was Rust-only `\let\@nil\relax` in `latex_base.rs` breaking babel's
  `\bbl@fornext` termination check. Removed. Ripple-fixed the French
  `:;!?` spacing + `ltx_align_left` para class as side effects.
- **Session 108 (2026-04-17)**: 50+ package-parity commits (elsart,
  mn2e, aa, texvc, sv_support, etc.); xcolor RGB case-sensitivity
  fix; RefCell panic fix in `with_font_info`. 512-paper sandbox:
  93.2% ok, 0 panics.
- **Sessions 42-107**: earlier work archived in git log and
  `memory/project_session_history.md`.

---

> **Reminder:** Every entry ported from Perl must follow tightly the original semantics and nuances. Read the Perl source, translate precisely, preserve edge cases. The Perl code is the ground truth.
