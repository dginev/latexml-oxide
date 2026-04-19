# Engine Sync Status: Perl vs Rust

> **This is a Perl-to-Rust translation project.** Every ported function, macro, and definition must faithfully reproduce the original Perl semantics, control flow, and edge-case behavior. The Perl source (`LaTeXML/` directory) is the ground truth. Only diverge when explicitly documented in `docs/OXIDIZED_DESIGN.md`.

Updated 2026-04-19. **Open gaps & active TODOs only.** Completed work
lives in git log and `memory/project_session_history.md`.

**Test inventory:** 423 tests pass (0 failures, 0 ignored) via `cargo test --release --tests`.

**arxiv sandbox:** 101 papers in `arxiv-examples/`. **93+%** catalog OK.

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
| base_parameter_types.rs | MINOR | `DirectoryList`/`CommaList`: no Array type in Rust (ported to `{d1}{d2}...` token-stream encoding) |
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

### Phase D0: 2k-sandbox failing articles — HIGH PRIORITY

From `~/data/10k_sandbox_html/results.tsv` (1962 papers run, 1877 ok / 95.7%).
**84 failing articles** total: 19 aborts + 1 error + 64 conversion_errors.
**Goal:** every article below must convert error-free. Resolve article-by-article —
distill minimal `.tex` examples, compare Perl vs Rust, patch the root cause.
**Do not rerun the full sandbox** until every individual issue is solved
(expensive; the list here is the authoritative worklist).

**Aborts + error (20)** — process-level failures (SIGABRT / exit=1):

- [ ] 0710.1208 — 2.5s abort (fast crash — xy-pic OOM)
- [ ] 1004.3503 — 8.3s abort (fast crash — xy-pic OOM companion)
- [x] 1003.0934 — fixed session 119 (`load_class` now calls `maybe_require_dependencies`)
- [x] 0908.4110 — fixed: `find_main_tex` now falls back to extension-less / ≥4-char-ext files (Perl Pack/Dir.pm L47)
- [ ] 0704.2334 — 68.4s timeout
- [ ] 0705.0790 — 63.3s timeout
- [ ] 0705.1522 — 62.2s timeout
- [ ] 0706.0243 — 64.3s timeout
- [ ] 0706.1988 — 67.4s timeout
- [ ] 0708.2154 — 61.2s timeout
- [ ] 0708.4176 — 60.1s timeout
- [ ] 0711.1898 — 66.3s timeout
- [ ] 0802.0544 — 60.1s timeout
- [ ] 0802.1035 — 69.4s timeout
- [ ] 0806.0463 — 65.3s timeout
- [ ] 0810.3087 — 70.5s timeout
- [ ] 0811.0190 — 60.1s timeout
- [ ] 0901.1988 — 60.1s timeout
- [ ] 0902.0261 — 60.1s timeout
- [ ] 0904.1990 — 68.9s timeout

**Conversion errors (64)** — `Status:conversion:2`, exit 0 with errors in log:

- [x] 0704.3480  - [x] 0707.0739  - [x] 0709.4470  - [ ] 0711.4787
- [ ] 0802.3360  - [x] 0803.0466  - [ ] 0805.2376  - [x] 0809.1906
- [x] 0810.0991  - [ ] 0810.1407  - [ ] 0810.4067  - [x] 0811.3209
- [ ] 0811.4212  - [x] 0904.2651  - [x] 0905.4086  - [x] 0906.1883
- [ ] 0908.0398  - [x] 0909.2656  - [ ] 0909.3444  - [ ] 0909.5007
- [x] 0911.1806  - [x] 0911.3337  - [x] 0911.3798  - [x] 0911.4739
- [x] 0912.2337  - [x] 1003.2989  - [x] 1003.3360  - [ ] 1004.2626
- [x] 1005.1610  - [ ] 1006.5231  - [ ] 1007.2309  - [ ] 1007.3314
- [ ] 1007.4392  - [ ] 1008.2152  - [x] 1008.4386  - [x] 1009.1431
- [x] 1010.1244  - [x] 1010.3600  - [x] 1010.4240  - [x] 1011.1955
- [ ] 1011.4834  - [x] 1011.5076  - [ ] 1012.3836  - [ ] 1101.2149
- [x] 1101.2474  - [x] 1103.2925  - [x] 1105.0121  - [ ] 1107.0347
- [ ] 1107.3732  - [ ] 1108.0951  - [ ] 1108.3241  - [ ] 1111.0334
- [ ] 1112.4846  - [ ] 1201.1473  - [x] 1201.4735  - [x] 1202.5647
- [ ] 1203.6616  - [ ] 1204.5278  - [ ] 1206.0536  - [x] 1207.5555
- [ ] 1207.6068  - [x] 1207.6456  - [ ] 1209.1578  - [ ] 1209.2771

**Conversion errors (64)** status: **28 of 64 now convert cleanly** via the
session 120 work (picture-autoOpen fractional priority, DefEnvironment bare
beforeDigest, JHEP figure/table macros, omnibus keywords-to-frontmatter,
\tmspace / \IfFormatAtLeastTF / \bi / \cpc stubs, \DeclareMathSymbol glyph
fallback, LoadClass prefix-match fallback across latexml_package + contrib).

**Per-article diagnosis method:**
1. `tail -200 ~/data/10k_sandbox_html/<id>.log` — identify first cascading error.
2. `unzip ~/data/10k_sandbox/<id>.zip` in a tmp dir; find the offending construct.
3. Distill a minimal `.tex` reproducer; compare `latexml` vs `latexml_oxide` outputs.
4. Trace to a Perl behavior in `LaTeXML/lib/` and patch the Rust translation.
5. Ensure all 423 tests still pass; mark the entry `[x]` here with a one-line note.

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

**Known content-model gap — FIXED (session 119):** Perl's `Tag('ltx:picture', autoOpen => 0.5)` wraps bare picture primitives (`\line`, `\circle`, `\vector`, `\put`) used outside `{picture}`. Ported the fractional-priority model in `compute_indirect_model`/`_aux`: priorities are scaled u32 (100 = full, 50 = half), multiplied at each recursion step, and the best-priority start tag wins. Picture gets 50, everything else gets 100, so picture only wraps when no fuller path exists. `Tag!("ltx:picture", auto_open => true, auto_close => true, …)` is now enabled. 9 `malformed:ltx:g` papers fixed, plus `ltx:line`/`ltx:rect` collateral.

#### D3. Performance catalog — after D1 reaches 7,898 / 0 errors

1. List tasks >60s with wall-clock time
2. Profile top offenders (flamegraph, token count, loop detection)
3. Targeted optimizations

#### D3b. Stability — eliminate SIGSEGV

Sources: libxml2 FFI (UAF on unlinking), libxslt C (namespaced elements), Rust unsafe in arena, parallel benchmark writes sharing paths.

Outstanding:
- [ ] Route libxml node lifetimes through guardian forbidding unlink without cache invalidation.
- [ ] Replace unsafe-over-FFI with safe wrappers where practical.
- [ ] Rc `Can not mutably reference a shared Node "text"` cluster — strong count grows past cap (libxml `set_node_rc_guard`). Raising the cap shifts the symptom one node higher (cap 50 → err at 51; cap 128 → err at 129), so it's a genuine accumulating-holder leak, not benign sharing. Hits all 4 dcpic / pictexwd / curves papers: 0805.2376, 1007.2309, 1108.3241, 1204.5278. Likely in alignment or diagram-cell machinery — the shared handle is always `"text"`. Leaving at guard=50 until the real root cause is found.

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

**Callgrind (session 105, 0707.1173 math-heavy paper):** Math parser
Marpa dominates — `transitive_closure` 34.3%, `marpa_g_precompute`
8.3%, `bv_scan` 7.1%, AVL ops 6.8%. Marpa-related >60% CPU.

**Callgrind (session 116, `complex/si.tex` siunitx-heavy):** Marpa is
**0.0%** of CPU — this fixture has almost no complex math. The
dominant costs are in gullet token reading and VecDeque-based
pushback management:

| Band | Share (Ir) | Site |
|---|---|---|
| Gullet token read path | ~15% | read_x_token + read_internal_token + read_token + read_balanced |
| VecDeque ops (pushback + pending_comments) | ~10% | unread_vec + inner pushback.pop_front / push_front |
| Allocation (mimalloc + memcpy) | ~5% | alloc/free/realloc + raw_vec grow |
| Arena string-interner probes | ~2% | get_or_intern_using + hashbrown |
| state::lookup_meaning | ~1.4% | per-token meaning lookup |
| Stored::clone | ~1.0% | Stored enum clone (Tokens clone internally) |
| Token::defined_as | ~1.2% | per-token cs comparison |
| Parameter::read | ~1.8% | argument-parsing machinery |

Takeaway: **the hot path depends heavily on the document**. Math-heavy
docs are Marpa-bound; siunitx/physics-heavy docs are gullet-bound.
Generalized wins should reduce per-token gullet cost (pushback
structure, RefCell borrow amortization) rather than chase Marpa.

**After `state::with_meaning` conversion** (session 116 commits
0f4797d7 / f3289ad7 / 706eaeaa): `Stored::clone` dropped from 1.02%
to 0.17% (~85% reduction); `lookup_meaning` from 1.38% to 0.17%.
Total instruction count: 17.87B → 17.33B (~3% fewer). The closure-based
borrowing API is now the preferred pattern for Stored-inspecting
callers — use `with_meaning(token, |m| … )` instead of
`lookup_meaning(token)` whenever the caller only inspects the meaning
(not moving ownership forward).

**After pushback VecDeque→Vec (LIFO stack)** (session 117 commit
2f48e7c4): unread_vec + push_front VecDeque overhead dropped from
~4.3% to ~3.0%. Total instruction count: 17.33B → 16.46B (another
~5%). The gullet pushback is pure LIFO in hot paths; the VecDeque
head-pointer arithmetic was paying for a FIFO capability used only
by \\endinput (`flush_mouth`), which is now handled via a single
`splice(0..0, …)` on the rare path.

**Cumulative perf trajectory on si.tex** (direct conversion, not
cargo test):

| Session phase | Ir (billion) | wall-clock |
|---|---|---|
| Session start | 17.87 | ~1.88s |
| After with_meaning refactor | 17.33 | ~1.80s |
| After read_balanced pre-size | 16.94 | ~1.77s |
| After pushback VecDeque→Vec | 16.46 | ~1.74s |
| After arena resolve_unchecked | 15.94 | ~1.70s |
| After dead tracing lookup removal | 15.32 | ~1.71s |
| After Parameter::read destructure | ~15.0 | ~1.67s |

~16% fewer instructions, ~11% faster on this workload. Wall-clock
noise is ~0.05s run-to-run, smaller than the cumulative delta.

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
