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

### Phase D0: 2k-sandbox failing articles — **COMPLETE (84/84)**

From `~/data/10k_sandbox_html/results.tsv` (1962 papers, 1877 ok / 95.7%).
The original 84-paper worklist (19 aborts + 1 error + 64 conversion_errors)
is fully resolved. The per-paper [x] checklist previously here has been
retired — the only entry kept as reference is the Perl-error-only exclusion:

> `[~] 1207.6068` — Perl emits 30 errors on this fragment (acknowledgements-only
> file, no `\documentclass`). Per the sandbox baseline rule — only
> Perl-error-free cases count — this paper is excluded from the parity target.
> `[~] 0909.3444` — Perl emits 2 errors (frenchb babel missing).

**New D1 ramp-up discoveries (session 124):**

- [x] **1311.6082** — FIXED: Rust engine/tex.rs auto-registered `\listfiles`
  as an autoload trigger, but never ported Perl's
  `DefPrimitive('\listfiles', undef)` (latex_constructs.pool.ltxml L4354).
  Post-load, the trigger kept self-re-emitting; 50M pin-count sentinel
  fired. Added the no-op primitive (latex_constructs.rs L6188).
- [x] **1611.10101** — FIXED (rewrite.rs L475): `tree.get_parent().unwrap()`
  in the Replace clause panicked on root-level / detached match nodes.
  Perl's `$tree->parentNode` returns undef silently and subsequent
  `$parent->lastChild`/`->childNodes` calls no-op; Rust now early-returns
  `Ok(())` on None, matching Perl's effective skip.
- [x] **hep-ph/9210235** — FIXED (semantics.rs:1619): math-parser
  single-arg delimited branch did `create_xmrefs(...).remove(0)`.
  `create_xmrefs` filters out XMHint and other ephemeral variants, so
  when the sole arg was a spacing hint the ref-vec came back empty and
  `.remove(0)` panicked. Fall back to a bare XMWrap when refs is empty
  — the Dual with an XMRef to nothing would be meaningless anyway.
  Found in D1 2048-sample.
- [~] **1210.4211** — INTERMITTENT under parallel load. Serial run
  clean (0.09s). Flaky reproduction under GNU parallel / sandbox
  stress — error cascade (`\ref / \UG / \If / \caption / \thesubsection`
  undefined) triggers an unbounded recovery loop. Perl sees the same
  undefined-CS cascade under the same context (51 errors, 27 undefined
  macros) and completes in 7.35s. Arena sentinel false-positive removed
  (session 124); the underlying recovery-loop divergence vs Perl is
  still open.
- [~] **1212.2052** — SIGSEGV, reproducible serially.
  Minimal repro:
  ```latex
  \documentclass{article}
  \begin{document}
  $\,\,\,\,\,\,\,\,\,\,\,\,\,\,\,\,\,$
  \end{document}
  ```
  Exactly 17 consecutive math-space tokens (`\,`, `\quad`, etc.) inside
  a math environment trigger the crash (16 works, 17 fails). Crash is
  NOT in math parsing (reproduces with `--nomathparse`). Session-125
  narrowing: the DOM is corrupted *by the end of the Building phase* —
  a plain `document.to_string()` immediately after absorb already
  SIGSEGVs in libxml2's `xmlSaveDoc`. That is, the rewrite-time
  `findnodes` crash observed earlier is a downstream symptom, not the
  root cause: the Rust-side DOM construction for 17 consecutive XMHint
  siblings leaves a dangling pointer that libxml2 then chases. Perl
  processes the identical input cleanly (output: 17 thin-space chars
  inside a `<p>`). Same `Rc<Node>` aliasing family as the D3b cluster
  and 1404.1913. Root-cause is in Rust's absorb/open_element pipeline.
- [~] **1710.03688** — OOM kill at ~19 GB RSS during babel french.ldf
  loading. Triggered by `\bbl@exp@aux` undefined CS in modern babel
  internals. Likely a babel 3.x port gap.
- [~] **1404.1913** — "double free or corruption (fasttop)" during
  Finalizing. libxml2 aliasing-class bug, similar family to the
  prior `Rc<Node>` `weak_count == 0` cluster.
- [~] **1605.01946 / 1709.05096 / 1805.09247** — watchdog timeouts under
  parallel load, converge cleanly in serial (same pattern as
  1210.4211). System-scheduling interaction, not per-paper.

**Phase D0 cumulative fixes (session 123-124):**

Ported/patched Perl-parity gaps across: xy-pic curve deciphering,
omnibus autoload stub guards, pstricks support, cp1251 reloadability,
fontenc cyrillic stubs, libxml NODE_RC_MAX_GUARD, `\begin{document}`
ExplSyntaxOff preamble cleanup, `def_math_constructor` isMath on
Whatsit props, revtex4 amsmath gating, expl3 short-circuit,
`\listfiles` no-op primitive. The detailed per-fix narration previously
tracked here (pstricks, cp1251, fontenc cyrillic, node_rc_guard,
ExplSyntaxOff preamble, revtex4 amsmath gate, expl3 short-circuit,
DefEnvironment beforeDigest, `\braket` token identity, ref_step_id
auto-counter, graphics rotatebox ordering, JHEP hyperref, xy-pic
crv_decipher body read, etc.) is now retired — consult git log if a
historical fix needs verification.

**High-fidelity parity tasks retained:**
- [x] **1209.2771 Figure 6** — EPS BoundingBox port of Perl's
  `LaTeXML::Util::Image::image_size`, `read_image_dimensions` now
  parses `%%HiResBoundingBox:`/`%%BoundingBox:` with DOS EPSI preview
  offset support. `\resizebox{6cm}{!}{\includegraphics*{.eps}}` now
  matches Perl to 10 significant digits.

**Session 124 verification & new work:**

- **D0 84/84 parallel verification (`parallel -j 12`):** all 84 worklist
  papers via direct `latexml_oxide --timeout=60` — exit 0, 0 errors,
  0 warnings. Avg 3.0s, max 16.84s (0704.2334).
- **D1 128-sample (even spread across 7898):** 128/128 clean after
  `\listfiles` fix.
- **D1 512-sample (even spread):** 520/521 real papers clean.
  1210.4211 hangs under certain parallel-wrap contexts (see "New D1
  discoveries" above).
- **D1 1024-sample (session 124):** 976/993 = 98.3% clean.
- **D1 2048-sample (session 125):** 1932/1977 = 97.7% clean. 15 distinct
  failures: 2 SIGSEGV (1212.2052 family), 1 panic (hep-ph/9210235,
  FIXED session 125), 11 watchdog-timeouts, 1 OOM (1710.03688 babel
  french). Watchdog-timeout papers tend to clear in serial runs —
  they are parallel-scheduling-sensitive, not per-paper bugs.
- **Harness caveat:** the shell glob `ls $d/*.tex | head -1` missed
  uppercase `.TEX`, `.ltx`, `.latex`, extension-less, and
  `\documentstyle` (LaTeX 2.09) main files. With a broader search the
  same 2048-sample measured 1960/1982 = 98.9% clean under direct
  `latexml_oxide`. The remaining deficit (2048 → ~1982) is sampling
  arithmetic (`awk 'NR % 4 == 1'` on 7898 picks 1975 rows) plus a few
  bundles with `\documentstyle`-only sources.
- **Session 125 cortex_worker ZIP→ZIP sweep (ar5iv profile, 1100 papers):**
  1016/1151 = 88.3% clean. This is the *production-path* measurement
  (the direct `latexml_oxide` runs use the plain profile and thus
  skip ar5iv-specific preloads); the ~10-pp gap captures papers that
  succeed standalone but fail or emit errors under the cortex ar5iv
  profile — primarily "`Script _ can only appear in math mode`"
  cascades, XMTok-in-text model violations, and watchdog timeouts.
  **1016 clean papers comfortably exceeds the 1000-document-parity
  PR target.**

- **Session 126 cortex_worker rebuild + re-sweep (after math-parser
  hep-ph/9210235 fix landed):** binary at Apr 19 22:07 was stale; the
  `cortex` feature gate meant my earlier `cargo build --release` didn't
  rebuild the cortex binary. After `cargo build --release --features
  cortex --bin cortex_worker`: 1024/1159 = 88.4% clean on same sample.
  Failure breakdown:
  - 51 watchdog (exit 134) — pathological serial hangs / very slow.
  - 8 SIGSEGV (exit 139) — **all 8 in MathML post-processing** (the
    conversion log reports "Conversion complete: No obvious problems"
    then `latexml_post MathML::Presentation` abort). Papers: 0709.2286,
    0710.1208, 1110.2158, 1212.2052, 1402.6805, 1504.04055, 1605.07431,
    1611.00957. Verified: `--no-pmml` suppresses the crash on 1611.00957.
    Open task: find the specific XMath node shape that trips the
    `pmml`/`pmml_apply`/`pmml_array` recursion.
  - 2 panic (exit 101) — 1410.8508, 1608.08252. Not yet triaged.
- **Arena pin-count sentinel replaced with symbol-count sentinel**
  (`common/arena.rs`): the pin-call-count metric was a false positive
  — dedup-heavy hot loops would trip it without any actual arena
  overflow risk. The correct signal is `arena.len()` (distinct symbol
  count); threshold raised to 10M which is still two orders below the
  danger zone (~100M symbols given average string length). The real
  recovery-loop bug 1210.4211 exhibits is now silent on the arena
  side, and surfaces only as the main-level wall-clock watchdog abort.
- **Mouth `Anonymous String {gid}` → `Anonymous String`:** the
  per-instance gid was pinning a unique SymStr per anonymous mouth;
  it served no functional purpose beyond visual disambiguation. All
  anonymous mouths now share one arena entry regardless of call count.

**Session 124 xy-pic fix:** `\lx@xy@crv@decipher` (xylatexml_tex.rs L799) was
calling `macro_string` (which runs `do_expand`) on `\xycrvdrop@` and `\xycrvconn@`
to inspect what drop/connection was requested. The Perl source uses
`ToString(LookupDefinition(T_CS('\xycrvdrop@'))->getExpansion)` — reading the
macro body, NOT expanding it. When `@/curve/` and `@{-->}` appear together,
expanding `\xycrvconn@` re-invokes `\dir{-->}` which feeds back into the curve
pipeline and re-invokes `\lx@xy@crv@decipher`, looping unbounded (21GB RSS / 19s
wall before OOM kill). New `macro_body` helper returns the raw Tokens body via
`lookup_definition(...)->get_expansion()`. Minimal repro
`$$\xymatrix{A \ar@/^5ex/@{-->}[r] & B}$$`: 21GB OOM → 59MB / 0.13s.

**Papers removed from worklist** — Perl also emits errors under
`--preload=ar5iv.sty --path=/home/deyan/git/ar5iv-bindings/bindings`
(the apples-to-apples comparison profile cortex_worker uses), so we
can't converge on them without also fixing the upstream Perl side:

- **0909.3444** — 2 Perl errors (frenchb babel missing)

**Per-article diagnosis method:**
1. Run Perl `latexml` on the paper; capture its log + error count.
2. If Perl errors too with the *same* CS, skip — likely a shared document bug.
3. If Perl succeeds (or gets further), apply `wisdom_upstream_error_attribution`:
   the divergence is earlier than the named symptom. Read the `.sty`/`.cls` source,
   trace the conditional / option / flag / deferred-hook machinery, identify what
   branch Perl takes that Rust doesn't.
4. Ensure all 423 tests still pass; mark the entry `[x]` here with a one-line note.
5. Use the parallel sweep (`parallel -j 12`) after every landed fix to catch cascaded
   benefits and regressions across the full 64-paper set.
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

#### D3. Performance catalog — slow-paper backlog (session 124 refresh)

**Tier A revisit (session 124, direct `latexml_oxide` wall-clock, idle):**

| id | dt (s) Orig | dt (s) Cur | speedup |
|----|-----------:|-----------:|--------:|
| 0906.1883 | 31.2 | 0.55 | 57× |
| 1011.1955 | 20.9 | 2.61 | 8× |
| 1009.1431 | 19.5 | 1.54 | 13× |
| 1008.4386 | 17.4 | 2.04 | 8× |
| 0909.2656 | 14.5 | 0.29 | 50× |
| 0911.4739 | 11.1 | 0.70 | 16× |
| 1005.1610 | 10.3 | 0.57 | 18× |
| 0803.0466 | 10.0 | 0.59 | 17× |

Note: original baselines were measured under `cortex_worker --standalone`
with zip archive I/O at `-j 12` parallel; the refreshed run is direct
`latexml_oxide --timeout=60` under idle load. Wrapper overhead ≈0.3-0.5s
and parallel contention additionally inflate the cortex_worker numbers.
Still, all Tier A papers now clear 3s on the bare binary — the slow-paper
backlog is effectively resolved for this tier under the cumulative session
116-124 engine fixes (omnibus stub guard, pin_char cache, expl3
short-circuit, xy-pic crv_decipher body read).

**Tier C revisit (session 124, same methodology as Tier A above):**

| id | dt (s) Orig | errs Orig | dt (s) Cur | errs Cur |
|----|-----------:|----------:|-----------:|---------:|
| 0802.3360 | 27.0 | 3   | 1.83 | 0 |
| 1209.1578 | 25.1 | 130 | 3.08 | 0 |
| 1107.3732 | 22.1 | 1   | 2.67 | 0 |
| 1203.6616 | 15.8 | 2   | 1.49 | 0 |
| 0909.5007 | 14.4 | 2   | 1.19 | 0 |
| 0711.4787 | 11.8 | 2   | 1.25 | 0 |
| 1108.0951 |  8.1 | 1   | 0.68 | 0 |
| 1004.2626 |  6.5 | 6   | 0.88 | 0 |

All 8 Tier C papers now clean (0 errors, 0 warnings) under 3.1s. The
session 120-124 per-paper Perl-parity fixes (recorded above under the
64 conversion_error list) resolved each root cause and the cumulative
engine perf wins dropped the wall-clock alongside. **Tier B (3 papers)
was a subset of now-resolved entries** — retired.

**Active perf tasks (D3) — post-124 status:**
- [x] Tier A/B/C backlog resolved. Remaining papers all under 4s on
  direct `latexml_oxide` (cortex_worker zip + -j 12 adds ~0.5s fixed
  overhead + contention). No individual paper is a performance outlier.
- [ ] Capture Tier A (~10 papers) + `complex/si.tex` as a standing perf
  corpus. File: `docs/PERFORMANCE.md` when a tracked regression surfaces.

**Method (after session 120 feedback_parallel_sweeps memory):**
```bash
printf '%s\n' $ids | parallel -j 12 --line-buffer \
  "t0=\$(date +%s.%N); errs=\$(./target/release/cortex_worker --standalone \
    --input ~/data/10k_sandbox/{}.zip --output /tmp/{}.zip --timeout 30 2>&1 \
    | grep -cE 'Error:'); t1=\$(date +%s.%N); \
   dt=\$(echo \"\$t1-\$t0\" | bc -l); \
   printf '%s errs=%s dt=%.1fs\\n' '{}' \"\$errs\" \"\$dt\""
```

#### D3b. Stability — eliminate SIGSEGV

Sources: libxml2 FFI (UAF on unlinking), libxslt C (namespaced elements), Rust unsafe in arena, parallel benchmark writes sharing paths.

**Policy:** no direct `libxml::bindings::*` FFI calls from the latexml
project. When a safe API is missing, add it to the `rust-libxml`
wrapper crate at `~/git/rust-libxml` and vendor-patch or upstream the
addition. That keeps unsafety isolated to one dependency and lets
future stability work on libxml2 node lifetimes happen in one place
rather than scattered across latexml_core / latexml_post / etc. Current
direct `libxml::bindings::*` call sites (should shrink to 0 over time):
`latexml_core/src/lib.rs`, `latexml_core/src/document.rs` — 5 total.

Outstanding:
- [ ] Route libxml node lifetimes through guardian forbidding unlink without cache invalidation.
- [ ] Replace unsafe-over-FFI with safe wrappers where practical.
- [ ] Migrate the remaining `libxml::bindings::*` callers to high-level
  `rust-libxml` methods; upstream new methods as needed.
- [~] Rc `Can not mutably reference a shared Node "text"` cluster — session 123
  raised `set_node_rc_guard` to 8192 after confirming the guard is a
  diagnostic heuristic (real aliasing is caught by `weak_count == 0`).
  dcpic papers 0805.2376 (ergkaehler25), 1007.2309, 1108.3241, 1204.5278
  now all converge cleanly. Lower-priority follow-up: identify the
  *semantic* cause of high ref counts on `"text"` nodes (libxml's own
  `document.nodes` hash accounts for some, but dcpic diagrams push to
  ~2000-8000 — may indicate redundant caching). Not a correctness
  blocker now.

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
- [~] `lookup_value(key)` → `with_value(key, |v| …)` closure refactor
  (248 sites). Session 123 did: `mathchar.rs` (8 sites), the
  `pin_char` ASCII cache, and `defined_as` (session 116). Pattern:
  `Option<Stored>` allocates the enum envelope when all you need is a
  Copy variant (Token, Int, Bool) or an Rc bump (Font, Tokens). Saves
  a Stored::clone per call. Remaining sites: `state.rs` (17),
  `binding/content.rs` (5), `keyval.rs` (4, but tricky — they return
  Option<Stored> APIs), `binding/counter/dialect.rs` (3).

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
| Session 123 start (D0 engine fixes accumulated) | (n/m) | ~2.80s |
| After `pin_char` ASCII cache (session 123) | (n/m) | ~2.29s |
| After arena sentinel removal + Mouth gid removal (session 126) | (n/m) | ~1.88s |

The ~+1.13s drift between sessions 117 and 123 correlates with D0
engine-level fixes (error-recovery rate-limiting, arena pin-count
sentinel with `arena.len()` probe, `NODE_RC_MAX_GUARD` bump). Session
123-126 walked back ~0.92s of that via `pin_char` caching and the
sentinel/gid simplifications. Net round-16 change vs round-15 start:
~same wall-clock at higher correctness (84 paper D0 + 16 D1 paper
fixes landed).

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
