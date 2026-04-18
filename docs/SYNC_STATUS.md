# Engine Sync Status: Perl vs Rust

> **This is a Perl-to-Rust translation project.** Every ported function, macro, and definition must faithfully reproduce the original Perl semantics, control flow, and edge-case behavior. The Perl source (`LaTeXML/` directory) is the ground truth. Only diverge when explicitly documented in `docs/OXIDIZED_DESIGN.md`.

Updated 2026-04-18. **Open gaps & active TODOs only.** Completed work
lives in git log and `memory/project_session_history.md`.

**Test inventory:** 413 integration tests pass (0 failures, 0 ignored).

**arxiv sandbox:** 100+ papers in `arxiv-examples/`. **93+%** catalog OK.

**10k sandbox:** last 512-paper ramp: **93.2% OK** (477 ok / 21 conv_error / 14 timeout / **0 panics**). Runner: `tools/benchmark_10k.sh`; tool: `cortex_worker --standalone`.

**Engine definition coverage:** **99.9%** (2,455/2,457 Perl Engine definitions ported). Only `\directlua` (LuaTeX) and `\ASCII` (niche) missing by design.

**Package bindings:** 100% (all 406+ Perl bindings ported). Zero MISSING.

**Dump:** 25,172 entries serialized; 6,154 installed into state at load time. Add-only policy preserves engine semantics. Unified load order `bootstrap → _base → dump → _constructs`. `LATEXML_NODUMP=1` opts out.

**Production-ready:** Full CorTeX ZIP-to-ZIP pipeline operational.

**See also:** [`KNOWN_PERL_ERRORS.md`](KNOWN_PERL_ERRORS.md) | [`OXIDIZED_DESIGN.md`](OXIDIZED_DESIGN.md) | [`PERFORMANCE.md`](PERFORMANCE.md)

---

## Engine Files — Open Gaps

| File | Status | Open Gaps |
|------|--------|-----------|
| base_parameter_types.rs | MINOR | `DirectoryList`, `CommaList`, `DigestUntil` stubbed |
| tex_box.rs | MINOR | Minor box dimension edge cases |
| tex_fonts.rs | MINOR | Missing: `\fontdimen` full array semantics |
| tex_tables.rs | MINOR | Minor: padding CSS classes (XSLT concern) |

**Cross-cutting:** `FontDef` parameter type simplified to `FontToken` — blocks full `\fontdimen`, per-font `\hyphenchar` tracking.

**Unported:** `AmSTeX.pool.ltxml` (112 defs, ~30%, Plain TeX rare); `BibTeX.pool.ltxml` (956 defs, 0%, skipped via `--nobibtex`).

## Tikz — Known Diffs (vs Perl output)

1. foreignObject transform Y / width/height
2. Arrow tip shape (different path data)
3. SVG viewBox/width — total dimensions differ slightly
4. tikz matrix rendering uses `<svg:g class="ltx_tikzmatrix">` groups (Rust) vs inline-blocks (Perl)

**Permanent sandbox ignores:** ns1–ns5 (52_namespace, no DTD); 2402.03300, 2410.10068, 2511.03798 (Perl also fails).

---

## Work Plan — Active TODOs

### Phase D: 10k-Document Sandbox

Scale testing to ~8,000 arxiv papers. Two stages:
1. **Coverage:** zero non-timeout failures at full scale.
2. **Performance:** eliminate timeouts at 120s cap.

**Process guards:** 60s timeout, 6GB RAM, output 200MB cap, parallelism via GNU parallel (16).
Ramp-up: exponential doubling (4→8→16→…→7898) with 0-error gate.

#### D1. Ramp-up runs — ONGOING

Last: **512 papers: 93.2% OK**. Residual blockers:
- `Missing $` display math (document bugs)
- Content-model `malformed` (`ltx:line` in `ltx:para`, `ltx:g` in `ltx:figure`)
- Raw-class undefined internals in exotic classes
- Rc<RefCell> "shared Node" error in 0805.2376 (tracked in D3b)

#### D2. Coverage fixes — ONGOING

Each cycle adds targeted fixes for specific undefined/misbehaving commands per log analysis. Detailed history in git log.

#### D3. Performance catalog — after D1 reaches 7,898 / 0 errors

1. List tasks >60s with wall-clock time
2. Profile top offenders (flamegraph, token count, loop detection)
3. Targeted optimizations

#### D3b. Stability — eliminate SIGSEGV

Sources: libxml2 FFI (UAF on unlinking), libxslt C (namespaced elements), Rust unsafe in arena, parallel benchmark writes sharing paths.

Outstanding:
- [ ] Route libxml node lifetimes through guardian forbidding unlink without cache invalidation.
- [ ] Replace unsafe-over-FFI with safe wrappers where practical.
- [ ] 0805.2376 "shared Node" error (Rc mutation during tree traversal).

#### D4. Performance — parallel scaling and allocations

**Baseline (session 105, paper 0707.1173):** 1-worker 22.6s → 16-worker 76.8s (29% per-worker efficiency). 14-core/20-thread machine. Peak RSS 570 MB/process.

**Active work:**
- [ ] Audit `.to_string()` (~1900 sites) — replace with `&str` / interned symbols where the value goes into `HashMap<String,String>`.
- [ ] Audit `String::from("...")` literals for interned conversions.
- [ ] Replace `HashMap<String,String>` with `SymHashMap<SymStr>` in hot paths.
- [ ] Audit `.clone()` in `document.rs` (~73), `latex_constructs.rs` (~73), `font.rs` (~39).
- [ ] Review `Tokens` cloning — pass `&Tokens` or `Cow` for read-only iteration.
- [ ] Profile math parser RAM independently (Marpa chart, forest).
- [ ] Investigate shared read-only engine state across processes (mmap dump).
- [ ] Long-running daemon / process pool to amortize 570 MB startup.
- [ ] Fork-based parallelism for CoW memory sharing.

**Callgrind (session 105):** Math parser Marpa dominates — `transitive_closure` 34.3%, `marpa_g_precompute` 8.3%, `bv_scan` 7.1%, AVL ops 6.8%. Marpa-related >60% CPU.

#### D5. Math parser optimizations (HIGHEST PRIORITY per callgrind)

- [ ] Avoid `init_grammar()` fallback — reuse existing grammar on reset failure.
- [ ] Audit script attachment ambiguity (`{}^4{}_{12}C^{5+}` — 27 unique trees).
- [ ] Early pruning: fail parses on inconsistency detection rather than post-hoc pragmas.
- [ ] Enumerate grammar rules by parse-tree count contribution.
- [ ] Document grammar ambiguity per category.

#### D6. Grammar First-Principles Plan

See `docs/MATH_GRAMMAR_FIRST_PRINCIPLES.md`. Live audit: `LATEXML_PARSE_AUDIT=1`.

**Remaining hotspots:**
1. `\sin[XY]` chain — 1022 trees / 10 unique (real semantic ambiguity)
2. `tr ρ / tr(XY) / rank M / …` — 100 / 8 unique
3. `FGHa` OPFUNCTION cascade — 87 / 9 unique (genuine math ambiguity)
4. `a|a|+b|b|+c|c|` VERTBAR — 53 / 10 unique

Primarily **semantic** — inherent to math practice; grammar refactoring has limits.

---

> **Reminder:** Every entry ported from Perl must follow tightly the original semantics and nuances. Read the Perl source, translate precisely, preserve edge cases. The Perl code is the ground truth.
