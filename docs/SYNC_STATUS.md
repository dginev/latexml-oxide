# Engine Sync Status: Perl vs Rust

> **This is a Perl-to-Rust translation project.** Every ported function, macro, and definition must faithfully reproduce the original Perl semantics, control flow, and edge-case behavior. The Perl source (`LaTeXML/` directory) is the ground truth. Only diverge when explicitly documented in `docs/OXIDIZED_DESIGN.md`.

Updated 2026-04-20. **Open gaps & active TODOs only.** Completed work
lives in git log and `memory/project_session_history.md`.

**Test inventory:** 423 tests pass (0 failures, 0 ignored) via `cargo test --release --tests`.

**arxiv sandbox:** 101 papers in `arxiv-examples/`. **93+%** catalog OK.

**10k sandbox (session 128 post-idstore-fix):** retried 38 aborts
from the prior 7898-paper sweep with the idstore-rebuild-at-finalize
binary at -j 8 parallel: **15 of 38 now pass** (incl. 1605.08055
SIGSEGV, DUPID borderline timeouts 1505.03876 / 1506.09203 /
1511.07586, pgfkeys-library paper 1511.00722, math-parser paper
1403.4135, and several CPU-contention edges that fit in 60 s at
-j 8 vs -j 12). Projected aggregate: **7884/7898 = 99.82%**
exit=0 at -j 8. Remaining 14 aborts: 5 OOM (exit=137 — 1112.6246,
1203.5977, 1710.03688 babel french, 1711.10191, 1711.11576), 9 at
-60 s timeout (exit=134 — 1210.1891, 1407.5769, 1308.5727,
1611.04489, 1702.00409, 1707.01155, 1709.05096, 1710.11417,
1802.08782). Remaining abort classes: pgfkeys.code.tex port gap
(1611.04489), math-parser pathological-ambiguity (1407.5769,
1308.5727), preamble-heavy digestion timeouts (1210.1891), and a
handful of slow-convergence papers. Runner:
`tools/benchmark_10k.sh`; tool: `cortex_worker --standalone --timeout 60`.

**Engine definition coverage:** **99.9%** (2,455/2,457 Perl Engine definitions ported). Only `\directlua` (LuaTeX) and `\ASCII` (niche) missing by design.

**Package bindings:** 100% (all 406+ Perl bindings ported). Zero MISSING.

**Dump:** 25,172 entries serialized; 6,154 installed into state at load time. Add-only policy preserves engine semantics. Unified load order `bootstrap → _base → dump → _constructs`. `LATEXML_NODUMP=1` opts out.

**Production-ready:** Full CorTeX ZIP-to-ZIP pipeline operational.

**See also:** [`KNOWN_PERL_ERRORS.md`](KNOWN_PERL_ERRORS.md) | [`OXIDIZED_DESIGN.md`](OXIDIZED_DESIGN.md) | [`PERFORMANCE.md`](PERFORMANCE.md)

---

## Engine Files — Open Gaps

| File | Status | Open Gaps |
|------|--------|-----------|
| base_parameter_types.rs | MINOR | `DirectoryList`/`CommaList`: no Array type in Rust (ported to `{d1}{d2}...` token-stream encoding); parameterized `CommaList:Type` form still unported |
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

**Perl-error-only papers** (excluded from parity target — Perl itself fails under the
same `--preload=ar5iv.sty --path=~/git/ar5iv-bindings/bindings` profile):

- `1207.6068` — Perl emits 30 errors (acknowledgements-only file, no `\documentclass`)
- `0909.3444` — Perl emits 2 errors (frenchb babel missing)

---

## Work Plan — Open TODOs

Phase D0 (2k-sandbox, 84/84) and the test-suite refactor (round 17) are
closed out; per-paper narration and session diaries for sessions ≤128
live in git log and `memory/project_session_history.md`. What remains:

### D1–D2. Residual sandbox aborts (~30 papers, ~0.4% of 7898)

Three failure classes in the session-128 7933-paper sweep, after the
6 DUPID aborts were addressed:

1. **pgfkeys.code.tex port gap** — **[x] Expected FIXED for
   `\usepackage{lipsum}\usepackage{tikz}`-pattern papers** (session
   post-b0b9852bd, 2026-04-21). The minimal reproducer
   `\usepackage{lipsum}\usepackage{tikz}` went from 8 errors to 0
   after the `load_tex_definitions` expl3 scope-exit cleanup landed
   in `latexml_core::binding::content`. The three named papers
   (1511.00722, 1611.04489, 1612.08368) all share the
   `\pgfutil@xifnch` undefined + `\group_begin:`-frame-mismatch
   symptom set, so the same root cause — a leaked expl3 group from
   a prior `\ProvidesExplPackage`-using package (lipsum in the
   narrowed case; likely the same upstream package in those papers)
   — is addressed. Full 10k sandbox re-run needed to confirm, but
   the minimal reproducer + 1098/0/0 full suite + 5 multi-package
   stress combos (siunitx/mhchem/hyperref/amsmath/xcolor × tikz)
   all clean strongly suggest cascade benefits.
   (Original analysis below kept for historical context.)
   `pgfkeyslibraryfiltered.code.tex` from TeXLive triggers
   `\pgfkeys@non@outer@newif` / `\pgfkeysalso` / `\ifpgfkeyssuccess`
   undefined-CS cascades that loop until the 60s wall-clock caps. Perl
   also loads the raw TeX (via `InputDefinitions('pgfkeys.code', type=>'tex')`)
   but succeeds — divergence is in how our raw-TeX processor handles
   specific pgfkeys idioms, not a simple stub gap.

   Round-17 diagnosis on 1611.04489 (commit `d6789258b` + round-17
   deeper dive): Perl-vs-Rust log comparison pinpoints the root cause.

   **First diverging event**: in Rust the error
   `Error:undefined:\pgfkeys` fires **inside `pgfsys.code.tex`**
   before `pgfkeys.code.tex` has finished loading. The file
   `pgfsys.code.tex` itself (from TeXLive generic/pgf/systemlayer)
   does a nested `\input pgfkeys.code.tex` at line 15, then uses
   `\pgfkeys{/pgf/.is family}` at line 19.

   **Perl handles this correctly**: the log shows
   `pgfkeys.code.tex.ltxml` loads *during* pgfsys.code.tex processing
   (nested `(Loading ...)(Processing ...)` blocks), so `\pgfkeys`
   is defined by line 19. Both files complete in ~0.36 s.

   **Rust mis-orders**: the log shows pgfsys.code.tex errors on
   `\pgfkeys` first, then pgfkeys.code.tex loads AFTERWARDS — i.e.
   the `\input pgfkeys.code.tex` at pgfsys.code.tex:15 isn't
   executing synchronously during the raw-TeX read. The
   pgfkeys load appears deferred to the next outer binding phase.

   **Refined diagnosis — bisected to an expl3-group-stack leak
   (round-17 later probe)**:

   A. Changing `read_x_token(Some(false), …)` → `Some(true)` in
      `load_tex_definitions` didn't move the needle. Ruled out the
      outer-loop-autoclose hypothesis.

   B. Direct reproduction of `\documentclass{amsart} \usepackage{tikz}
      \begin{document} done` — 0 errors. So `\usepackage{tikz}` *alone*
      is fine.

   C. 1611.04489's minimal preamble bisect:
      `\usepackage{lipsum} \usepackage{tikz}` → **988 errors**.
      `\usepackage{tikz}` alone → 0.

   D. Looking at the log, the **first error** is NOT at `\pgfkeys`
      — it's upstream, at the very start of `pgfutil-common.tex`:

      ```
      (Loading "pgfutil-common.tex" definitions...
      Error:undefined:\pgfutil@xifnch …
      Error:unexpected:} Attempt to close boxing group; current
      frame is non-boxing group due to T_CS[\group_begin:]
      ```

      `\group_begin:` is an **expl3** primitive. An unmatched
      `\group_begin:` — left over from the expl3 kernel / l3keys2e /
      xparse load sequence — has corrupted the group-stack frame.
      Every subsequent `{…}` in raw-TeX inputs mismatches, which
      cascades into the \pgfkeys/\pgfmath undefined-CS storm.

   **Root cause**: expl3 group-stack isn't balanced on load. The
   l3keys2e / xparse / expl3.sty chain runs `\group_begin:` without a
   matching `\group_end:` somewhere. The pgfkeys errors are
   downstream symptoms of a broken group frame, not a raw-TeX
   \input or \ifdefined bug.

   **Round-17 narrowing (session post-832710570)**: `\usepackage{X}
   \usepackage{tikz}` for `X ∈ {expl3, xparse, l3keys2e}` each produces
   **zero** `\group_begin:` frame errors. Only `\usepackage{lipsum}
   \usepackage{tikz}` reproduces (8 Errors in 6037-line log, first at
   pgfutil-common.tex L174 `\pgfutil@xifnch` undefined, immediately
   followed by L175 `}` closing on a `\group_begin:` frame). So the
   leak is **lipsum-specific**, not a broad expl3 machinery issue.

   **Fix-hypothesis test (same session)**: injecting `\ExplSyntaxOff`
   between lipsum and tikz clears the cascade (0 errors). Injecting
   bare `\group_end:` leaves 1 error. So the missing action is
   `\ExplSyntaxOff` specifically — which not only closes the group
   but also restores catcodes. The underlying leak is that our raw
   expl3-code.tex load defines `\ProvidesExplPackage` to push a
   `\group_begin:` frame as part of `\ExplSyntaxOn`, but nothing
   pairs `\ExplSyntaxOff` at end-of-file for lipsum.sty.

   **Perl's upstream awareness**: Perl's TeX.pool.ltxml L44-47 comment
   says "these auto-loads are not perfect — if triggered with a raw
   .sty file, the expl3 support will 'expire' at the end of the
   current scope, and e.g. `\ExplSyntaxOn` will once again be
   undefined." So Perl knows this is a known edge-case. The
   difference must be in how Perl vs Rust scope-exits a raw-.sty load
   — Perl's `input_definitions` apparently popping leaked frames
   automatically; ours not.

   **Surgical fix target**: modify `input_definitions` /
   `load_tex_definitions` in `latexml_core::binding::content` to
   detect group-stack-depth-increase across a raw-.sty load and
   auto-pop unclosed `\group_begin:` / `\begingroup` frames (with a
   warning). Alternatively, register `\ExplSyntaxOff` as an
   end-of-input hook whenever a raw-.sty load has `\ExplSyntaxOn` in
   effect at file-end.

   **[x] Landed**: `load_tex_definitions` in
   `latexml_core::binding::content` now digests `\ExplSyntaxOff` at
   end-of-file when (a) `_` catcode is LETTER (expl3 active),
   (b) `\ExplSyntaxOff` is defined, and (c) the file is not expl3 /
   xparse / l3keys2e / expl3-code (those legitimately leave expl3
   active for the caller). Clears the `lipsum+tikz` cascade from 8
   errors → 0. Full suite: 1098 passed, 0 failed, 0 ignored.

   **Fix target** is no longer `latexml_core::binding::content` but
   the expl3 binding in `latexml_package`: find which
   `\group_begin:` call is being stacked without its counterpart. A
   good starting point is `engine/latex_dump.rs`, `engine/latex.rs`
   (xparse / l3keys2e bindings), and the expl3-code.tex loader
   setup. Clears 3+ papers (1511.00722, 1611.04489, 1612.08368) and
   any other expl3+tikz users.

   **User directive (round 17)**: "We want deep and exhaustive parity
   with the LaTeX 3 kernel support (and naturally we accept
   improvements over perl, where possible). Do not take shortcuts,
   we want lipsum to load natively and cleanly." This upgrades the
   pgfkeys repair from a tactical bug fix to a strategic expl3
   kernel audit. Scope:
   - `expl3_sty.rs` currently gates loading behind `dump_has_expl3`
     and uses `SUPPRESS_UNDEFINED_ERRORS` + `SUPPRESS_UNEXPECTED_ERRORS`
     to mask forward references in expl3-code.tex. The error
     suppression itself is a shortcut — a genuine expl3 port
     shouldn't need it.
   - Catcode-restoration safety-nets in `expl3_sty.rs`,
     `xparse_sty.rs`, `l3keys2e_sty.rs` are also shortcuts — the raw
     files' own `\ExplSyntaxOff` should work. If they don't, the
     cause is our catcode-group-scoping.
   - The known PA-alias gate in `dump_reader.rs` blocks every
     `:`-named alias including trivial `\group_begin: PA \begingroup`
     — safe enough in isolation, but reflects that we don't trust
     post-dump expl3 state.
   - A proper port processes expl3-code.tex (~36K lines) cleanly
     end-to-end, all forward references resolve naturally, catcode
     regime restores itself via `\ExplSyntaxOff`, and subsequent
     `.sty` loads (lipsum, xparse, l3keys2e, ...) compose without
     leaving stack frame or catcode residue.

   This is now a tracked long-horizon deliverable: **deep expl3
   kernel parity** — see new entry under "Long-horizon —
   architectural rationalization" below.

   The actual flow, verified by reading the code:
   - `\input` DefMacro (`tex_file_io.rs:184`) calls `input()`
   - `input()` (`binding/content.rs:684`) checks `INTERPRETING_DEFINITIONS`;
     when set (i.e. inside a raw-TeX load), dispatches to
     `input_definitions(...)` — the recursive synchronous path.
   - `input_definitions` → `load_tex_definitions` which does its own
     `reading_from_mouth` + read_x_token loop. Should drain pgfkeys
     fully before returning. Log confirms pgfkeys.code.tex loading
     banner DOES appear nested inside the pgfsys.code.tex block.

   So the `\input` path is structurally correct — pgfkeys.code.tex IS
   being loaded as a synchronous nested input of pgfsys.code.tex.
   The `\pgfkeys` undefined error at pgfsys line 19 must therefore
   fire *before* the nested \input at pgfsys line 15 runs. That
   implies the `\ifdefined\pgfkeysloaded\else\input pgfkeys.code.tex\fi`
   conditional at pgfsys lines 14-16 is taking the WRONG branch —
   skipping the \input when it should be taking the else path.

   Next step: instrument the `\ifdefined` conditional in
   `latexml_package/src/engine/etex.rs` and the conditional
   scan-ahead in the gullet to verify the scan correctly finds \else
   and \fi when the condition is false, and to inspect whether the
   `\pgfkeysloaded` token is erroneously getting a meaning before
   \ifdefined probes it.

   Clears 3+ papers (1511.00722, 1611.04489, 1612.08368) and
   several other pgf/tikz-users currently failing similarly.

   Perl log: /tmp/1611_perl_log.txt (25.4 s, exit=0, 2 warnings).
   Rust log: /tmp/1611_rust_log.txt (60 s timeout, 1004 errors).
2. **Math-parser pathological-ambiguity timeouts** — 2+ papers
   (1403.4135, 1407.5769). 500+-token formulas with 121 parse choices
   each, ~500ms per formula × hundreds of formulas = 60s timeout.
3. **Preamble-heavy digestion timeouts** — e.g. 1210.1891 stuck at
   hyperref → etoolbox → kvoptions → nameref chain load.

[ ] **Per-paper diagnosis method:**
1. Run Perl `latexml` with matching `--preload=ar5iv.sty --path=...`; capture log + error count.
2. If Perl errors with the *same* CS → shared document bug, skip.
3. Otherwise apply `wisdom_upstream_error_attribution`: the divergence is
   earlier than the named symptom. Trace `.sty`/`.cls` conditional /
   option / flag / deferred-hook machinery to find the branch Perl takes
   that Rust doesn't.
4. Ensure 423 tests still pass; mark the entry `[x]` with a one-line note.
5. Re-run 12-way parallel sweep after every landed fix to catch cascaded
   benefits and regressions.

### D3. Performance corpus

- [x] Capture Tier A (~10 papers) + `complex/si.tex` as a standing perf
  corpus in `docs/PERFORMANCE.md`. Regression trigger: wall-clock drift
  > 15% on any corpus entry between commits. Reproducer:
  `tools/run_perf_corpus.sh` (idle-serial, no parallelism). Round-17
  baseline is the dated table in `PERFORMANCE.md`.
- [x] **0911.4739** (5.04 s) and **1005.1610** (7.38 s) Tier A
  outliers — **root-caused**: both are Graphics-phase bound (PDF/EPS
  → PNG conversion via the external `convert` CLI from ImageMagick).
  Round-17 per-phase audit with a new `LATEXML_POST_AUDIT=1` env
  flag (commit `8df9c7b53`):

  | paper     | Graphics | MathML | XSLT | Scan | total post |
  |-----------|---------:|-------:|-----:|-----:|-----------:|
  | 1005.1610 | 6614 ms  |  23 ms | 76ms | 30ms |     ~6.8 s |
  | 0911.4739 | 4087 ms  |  29 ms |130ms | 40ms |     ~4.3 s |

  1005.1610 has 10 PDF figures totalling ~7.5 MB; 0911.4739 has 31
  mixed EPS/PDF. Each `convert` subprocess fork-execs ImageMagick,
  rasterises the PDF at density=150, and writes PNG. At 132-660 ms
  per image this is intrinsic rasterisation cost, not a Rust perf
  problem. MathML::Presentation itself is 15-29 ms total across
  365-771 nodes — trivial.

  Use `LATEXML_POST_AUDIT=1 latexml_oxide --post …` to reproduce the
  per-phase breakdown on any paper.

- [x] **Parallelise Graphics phase** — landed `aa3c7c1bb`. 3-phase
  refactor (serial DOM read / parallel subprocess / serial DOM
  write) via `std::thread::scope`, worker cap
  `min(available_parallelism, 8)`. Measured: 1005.1610 Graphics
  6614 → 1665 ms (4×), 0911.4739 4087 → 742 ms (5.5×). Full-pipeline
  wall-clock −66% / −65% on the two outliers respectively.

- [~] **Vector-preserving PDF/EPS → SVG via inkscape/pdf2svg**
  (tracks upstream [brucemiller/LaTeXML#902](https://github.com/brucemiller/LaTeXML/issues/902)).

  **Landed round 17** (opt-in, default off):
  - `--graphics-svg-threshold-kb N` CLI flag on `latexml_oxide`
  - `Graphics::with_svg_threshold_kb()` builder on the post processor
  - `convert_image_svg` shells out to `inkscape --export-type=svg
    --export-plain-svg` inside the round-17 parallel worker slot
  - `read_svg_dimensions` parses the root `<svg viewBox="…">` for
    width/height
  - `should_try_svg_path` — file-size heuristic (< N KB, PDF only)
  - Runtime falls back to ImageMagick `convert` when inkscape fails
    / is missing (silent fallback; no hard dependency)
  - README adds `inkscape` to the "Optional" apt install
  - CI (`.github/workflows/CI.yml`) installs inkscape so the branch
    is always covered on PRs
  - `latexml_post/tests/fixtures/cifar10_vector.pdf` (41 KB,
    matplotlib vector plot from issue #902) — the canonical test PDF
  - New `test_vector_svg_graphics_path` integration test: runs the
    Graphics processor on the fixture, asserts an `.svg` file is
    emitted with a `<svg>` root and under 2 MB (silently skipped if
    inkscape is absent — e.g. on bare dev laptops)

  **Empirical classification data (round 17)**:
  - CEP.pdf (30 KB vector): inkscape 0.43 s / 519 KB SVG; convert
    1.4 s / 61 KB PNG — SVG wins on fidelity, PNG on size
  - cifar10_vector.pdf (41 KB vector): inkscape 0.21 s / 60 KB SVG;
    convert 0.20 s / 149 KB PNG — SVG wins on both
  - Fade.pdf (1.7 MB raster-disguised-vector): inkscape 46 s / 102 MB
    (!!); convert 1.4 s / 61 KB — classic case the threshold must
    exclude; the 200 KB threshold we suggest does

  **Still to do**:
  - [x] Timeout on the inkscape subprocess — landed. Default 15 s
    (overridable via `LATEXML_INKSCAPE_TIMEOUT_SECS`). Rust-side
    spawn + poll + SIGKILL in `Graphics::run_with_timeout`; 6 new
    unit tests lock in the timeout/kill semantics, the file-size
    heuristic, and SVG viewBox parsing (also fixed a bug where the
    XML-prolog `?>` was mis-matched as the root-tag close).
  - [x] Benchmark the pathological-convert PDFs from issue #902 —
    landed with `fig8.pdf` fixture from arxiv:1807.01606 and a new
    `test_vector_svg_pathological_convert_case` regression test.
    Measured **130× speedup** end-to-end (32.4 s → 0.29 s via
    inkscape on a minimal doc containing just
    `\includegraphics{fig8.pdf}`). PERFORMANCE.md records the
    validation table.
  - [ ] EPS support via the same path. **Blocked upstream**:
    Inkscape 1.x dropped direct EPS/PS reading (relied on
    ghostscript glue that was removed). `inkscape source.eps`
    reports "Failed to open". Workarounds would be either (a) pipe
    through `epstopdf source.eps stage.pdf && inkscape stage.pdf`,
    adding a conversion step + `epstopdf` dependency, or (b) try
    `pstoedit` which can emit SVG directly from PS/EPS. Neither is
    compelling given that EPS files are often already raster-wrapped
    (EPSF-of-a-TIFF style). Leaving EPS on the ImageMagick raster
    path for now.
  - [ ] Consider pdf2svg as a cheaper fallback when inkscape is
    absent but pdf2svg is available — smaller install, simpler
    flags. (Not blocking; inkscape covers the use cases.)


Specific slow-convergence follow-ups:

- [~] **1709.05096** — >90s wall under parallel load (digestion, not
  post-processing). `Info:undefined:… KV:vattach …` scanning pattern
  suggests a keyval loop paying per-row cost in a huge tabular.
- [~] **1710.03688** — OOM at ~19 GB RSS during babel french.ldf load;
  `\bbl@exp@aux` undefined (babel 3.x port gap).

### D3b. Stability — libxml2 node lifetimes

**Policy:** no direct `libxml::bindings::*` FFI calls from latexml. When
a safe API is missing, add it to `~/git/rust-libxml` and
vendor-patch/upstream. **Zero direct FFI call sites remaining** as of
round 17 commit (see below).

- [ ] Route libxml node lifetimes through guardian forbidding unlink without cache invalidation.
- [x] Replace unsafe-over-FFI with safe wrappers where practical.
  Landed round 17. The last raw `extern "C" { fn exsltRegisterAll();
  } unsafe { … }` block in `latexml_post::xslt` was moved upstream to
  `~/git/rust-libxslt` (branch `latexml-oxide-contributions`, commit
  `a61d0c43`): new `pub fn exsltRegisterAll();` binding + `build.rs`
  linkage for libexslt + safe top-level `libxslt::register_exslt()`
  Once-guarded wrapper. `[patch.crates-io]` entry added for libxslt
  alongside the existing libxml patch. Verified: zero remaining
  `unsafe { … }` blocks across `latexml_post` and `latexml_oxide`
  crates. Core still has `unsafe` in `arena.rs` (resolve_unchecked),
  `store.rs`/`state.rs`/`error.rs` (Send/Sync impls) — those are
  intentional internal invariants, not FFI.
- [x] Migrate the remaining `libxml::bindings::*` callers to high-level
  `rust-libxml` methods; upstream new methods as needed.
  Landed round 17. Two wrappers pushed upstream to
  `~/git/rust-libxml` branch `latexml-oxide-contributions` (commit
  `db3a5fec`): `Node::new_comment(content, doc)` mirrors the
  existing `Node::new_text` but wraps `xmlNewDocComment`; top-level
  `libxml::init_parser()` is a Once-guarded wrapper for
  `xmlInitParser`. latexml-oxide workspace
  `Cargo.toml` gains a `[patch.crates-io] libxml = { path =
  "../rust-libxml" }` entry so the wrappers are available without a
  crates.io round-trip. `Document::add_comment_ffi` replaced by
  `Document::add_comment` which composes `Node::new_comment` with
  existing `add_child` / `add_prev_sibling`. `ensure_libxml_init`
  now calls `libxml::init_parser()` instead of reaching into
  `bindings::xmlInitParser` behind its own `Once`. Verified: zero
  remaining `libxml::bindings::` references across latexml-oxide
  src. 1098 tests, 0 fail.
- [~] Rc `Can not mutably reference a shared Node "text"` cluster — guard
  raised to 8192 (diagnostic, not safety). dcpic cluster 0805.2376 /
  1007.2309 / 1108.3241 / 1204.5278 all converge now. Lower-priority
  follow-up: identify the semantic cause of high `"text"`-node ref counts
  on dcpic diagrams (2000–8000 range).
- [x] **1605.08055 Finalizing-phase SIGSEGV** — FIXED (session
  128, commit `337c1ef52`). Root cause was a dangling-Node entry
  in `idstore`: upstream passes (math-parser `replace_tree`,
  various `unbind_node()` sites) drop xml:id-bearing subtrees
  without calling `unrecord_id`, so `mark_xmnode_visibility` later
  dereferences a freed libxml2 `Node` when it recurses through an
  `XMRef` whose idref resolves via the cache. Fix: new
  `rebuild_idstore_from_dom()` (clear + fresh DOM walk) called at
  the top of `finalize()` before `prune_xmduals`. 1605.08055:
  SIGSEGV → exit=0 in 0.8 s. No regressions on adjacent papers.
  **Do NOT also call `rebuild_idstore_from_dom` at the start of
  the Rewriting phase** — tried in session 128, broke split_test.
  When the DOM has duplicate xml:ids (rare but possible during
  math-parse), `findnodes` visits in document order so the
  FIRST-OCCURRENCE node wins the cache entry, but the prior
  idstore state may have had the LAST-OCCURRENCE node — which
  some rewrites depend on. Finalize is late enough that those
  rewrites have already fired, so the rebuild there is safe; at
  Rewriting entry it isn't.
  The full audit of every unlink-without-unrecord path remains
  open — this is a belt-and-suspenders fix that makes the
  downstream passes robust to stale idstore entries regardless of
  upstream behavior.

### Dump — deferred alias retry (session 128)

- [ ] `\a → \@tabacckludge` — still hand-written `Let!` in
  `latex_constructs.rs`. The dump serializer captured `\a` as an
  Expandable `E` record (with `\@changed@cmd`-wrapped body), not a
  PA let-alias, so the deferred-alias retry pass added in
  `91c82d5a4` doesn't help. Either (a) teach the serializer to
  detect "let-aliases preserved under _constructs" and emit them as
  PA records, or (b) widen the outer M-gate to admit E records whose
  body is a specific safe `\@changed@cmd` pattern. Deferred for now
  — the hand-written `Let!` mirrors latex.ltx L10007 exactly and
  has no behavioral downside.

### D4. Performance — parallel scaling & allocations

**Baseline (session 105, paper 0707.1173):** 1-worker 22.6s → 16-worker
76.8s (29% per-worker efficiency). 14-core/20-thread machine. Peak RSS
570 MB/process.

- [~] Audit `.to_string()` (~1900 sites) — replace with `&str` /
  interned symbols where the value goes into `HashMap<String,String>`.
  **61 sites converted round 17** (commits `741809e6e` through
  `7a5433cd4`), across 21 files spanning core, math parser, package
  engine, and individual packages (amsmath, xcolor, listings,
  enumitem, thmtools, xy, pgfsys, amscd, hyperref, fontenc, etc.).
  Key tooling added to the core:
  - `Tokens::eq_text` / `Tokens::starts_with_text`
  - `ArgWrap::eq_text` / `ArgWrap::starts_with_text`
  - `Stored::eq_text` / `Stored::starts_with_text` / `Stored::ends_with_text`
  - `state::graphics_paths_contains` (zero-alloc membership)
  Measured payoff (PERFORMANCE.md round-17 second refresh):
  complex/si.tex −9%, several Tier A papers −2-7%, others flat.
  Remaining sites are dominated by `Vec<String>` collections, format!
  results, and other legitimate owned-allocations. Further cleanup
  should be callgrind-guided, not blanket. **Pivoting** — next
  perf work should target Stored-enum rationalisation (separate
  long-horizon task) or Tokens deep-copy reduction.
- [ ] Audit `String::from("...")` literals for interned conversions.
- [ ] Replace `HashMap<String,String>` with `SymHashMap<SymStr>` in hot paths.
- [~] Audit `.clone()` in `document.rs` (~73), `latex_constructs.rs` (~73), `font.rs` (~39).
  **Audit finding (round 17)**: the raw clone counts are misleading.
  - `font.rs` (~39): almost all clones are on
    `Option<Cow<'static, str>>` fields whose common-case variant is
    `Cow::Borrowed("serif")`-style static-literal pointers (3 pointer
    reads, no heap). The `.clone().or_else(|| other.clone())` pattern
    in `make_concrete` / `merge_ref` looks redundant but costs
    exactly one real clone either way.
  - `document.rs` (~71): dominated by `self.node.clone()` patterns
    (24+ sites). `libxml::tree::Node` is `Rc<RefCell<_Node>>` so
    `.clone()` is an Rc refcount bump, not a DOM copy.
  Real optimisation would require consuming-self API overloads and
  an Rc-vs-Arc refcount audit — both invasive. Deferred as
  low-ROI. The D4 allocation-hotspot work should instead chase
  `.to_string()` on interner symbols and `Tokens` deep-copies —
  those are where the real heap churn is.
- [ ] Review `Tokens` cloning — pass `&Tokens` or `Cow` for read-only iteration.
- [ ] Profile math parser RAM independently (Marpa chart, forest).
- [ ] Investigate shared read-only engine state across processes (mmap dump).
- [ ] Long-running daemon / process pool to amortize 570 MB startup.
- [ ] Fork-based parallelism for CoW memory sharing.
- [~] `lookup_value(key)` → `with_value(key, |v| …)` closure refactor
  — 248 initial sites, ~12 converted (mathchar, pin_char, defined_as).
  Remaining: `state.rs` (17), `binding/content.rs` (5), `keyval.rs` (4,
  tricky — return `Option<Stored>`), `binding/counter/dialect.rs` (3).

### D5. Math parser ambiguity

Callgrind (math-heavy paper, session 105): Marpa dominates — transitive
closure 34.3%, grammar precompute 8.3%, bv_scan 7.1%, AVL 6.8%.
Marpa-related >60% CPU.

- [~] Avoid `init_grammar()` fallback — reuse existing grammar on reset failure.
  **Partial landing round 17.** The fallback path is still needed
  (`testscripts_test` fixture demonstrates grammar-corruption patterns
  that only clear after a fresh precompute), but the recovery ladder
  is now: (1) clone `self.grammar` + trivial parse, (2) retry once —
  covers transient state hiccups without reaching for init_grammar,
  (3) full `init_grammar()` rebuild if both clone attempts fail, (4)
  log + keep previous engine if init_grammar itself errors. Removed
  the `init_grammar().unwrap()` panic — subsequent formula parses
  fail cleanly (0 trees) rather than crashing the whole conversion.
  The "avoid" goal remains aspirational; the real win is graceful
  degradation, not elimination. Still-open refinement: instrument the
  fallback call count on the 10k sandbox and identify the triggering
  grammar cases to see if they are addressable at the grammar level.
- [ ] Audit script attachment ambiguity (`{}^4{}_{12}C^{5+}` — 27 unique trees).
- [ ] Early pruning: fail parses on inconsistency detection rather than post-hoc pragmas.
- [ ] Enumerate grammar rules by parse-tree count contribution.
- [ ] Document grammar ambiguity per category.
- [x] **Latent no-op pragmas — audit complete (round 17)**. Seven sites
  in `pragmatics.rs` previously matched only `XM::Lexeme("x.invisible_operator", …)`
  for the invisible-times operator head, but `apply_invisible_times`
  produces `XM::Token { role: MULOP, meaning: "times" }`, so they
  silently never fired on real parses.

  Landed:
  - `pragma_consistency_via_key` (session 128, `dfc0f263a`)
  - `pragma_fenced_letters_are_function_arguments` (`b786d85d4`) — 7 tests
  - `pragma_higher_order_invisible_ops_are_exceptions` (`c0c0720b6`) — 4 tests
  - `pragma_adjacent_numbers_dont_use_invisible_times` (`c0c0720b6`) — 3 tests
  - `check_invisible_times_recursive`, `is_invisible_times_apply`,
    `all_simple_identifiers` (`282870c9d`) — 6 tests; also extracted
    module-private `is_invisible_times_op` helper and DRY-replaced
    four earlier inlined match blocks (incl. both
    `pragma_functions_prefer_wider_absorption` sites).

  The MULOP-contains-RELOP check in `pragma_relops_are_outermost` at
  ~L1240 still has a `name == "x.invisible_operator"` OR-fallback, but
  the primary `name.starts_with("MULOP")` predicate already fires on
  the Token shape via `base_operator_name()`, so that branch is
  harmless dead code — left in place. Full workspace: 1090 tests, 0
  fail, 0 ignored.

Remaining semantic-ambiguity hotspots (see
`docs/MATH_GRAMMAR_FIRST_PRINCIPLES.md`; live audit via
`LATEXML_PARSE_AUDIT=1`):

1. `\sin[XY]` chain — 1022 trees / 10 unique (real semantic ambiguity)
2. `tr ρ / tr(XY) / rank M / …` — 100 / 8 unique
3. `FGHa` OPFUNCTION cascade — 87 / 9 unique (genuine math ambiguity)
4. `a|a|+b|b|+c|c|` VERTBAR — 53 / 10 unique

### Long-horizon — architectural rationalization

- [x] **l3hooks — Perl-parity stub port** (round 17, landed
  this session). Discovered: Perl LaTeXML's entire l3hooks support
  is a block of **no-op stubs** in `latex_base.pool.ltxml` L829-855
  that absorb expl3 hook-API syntax and expand to nothing. There is
  no hook storage, no `\hook_use:n` dispatch, no ordering engine.
  The prior SYNC_STATUS plan for a "minimal native port" with
  `state::push_value` storage was **Rust-side speculation**, not
  parity.

  Landed (Perl-parity):
  1. Added `\hook_gput_code:nnn{}{}{}` as a no-op `DefMacro!` in
     `latex_base.rs` (Perl L829, was missing). This was the one
     true gap in our hook stub block. Used the `DefMacroI`-style
     branch `DefMacro!(T_CS!("\\hook_gput_code:nnn"), "{}{}{}", "")`
     so the CS name is pre-tokenized as one unit — the string-
     prototype branch would otherwise split on `:` (OTHER) and `_`
     (SUB) under default catcodes and produce `\hook` + garbage.

  **Gate kept (pragmatic deviation, not removed)**: the
  `latex_constructs.rs:2501` `\hook_use:n{begindocument}` dispatch
  is a Rust-only compensator for a different deviation — our raw
  `expl3-code.tex` load path (active when the dump doesn't short-
  circuit it) really does define `\hook_use:n` and enqueues hook
  code against it. Perl doesn't load that file, so doesn't need
  the dispatch. We keep the gate with a comment noting the
  deviation; removing it would silently regress the raw-load path.

  Canary: `83_expl3` passes 2/2 in 0.00s. Full
  `cargo test --release --tests` reports **1098 passed, 0 failed,
  0 ignored** across 44 binaries.

  **Load-bearing consequence for the dumper staircase**: with the
  `\hook_gput_code:nnn` no-op stub in place, PA/MPA/E records in
  the dump whose bodies reference it now resolve cleanly (arg-
  swallow) instead of undefined-CS'ing. Retrying the dumper
  step-4/5 widening with this stub in place is the next diagnostic
  action.

  See also: "Future-facing / not-wired" section below for the
  native-storage port that was deferred.

- [ ] **Kernel-first discipline for dumper widening** (user directive,
  round 17). Cross-links the dumper audit and the expl3 kernel
  parity tasks. Each failed staircase step in the dumper widening
  is a signal that a specific runtime primitive is missing:

  - Step 4 (1-CS body E records) regressed because the referenced
    CS targets — often `\hook_*`, `\__regex_*`, `\__cmd_*` — are
    not defined by our engine, so the admitted wrapper cascades
    on expansion.
  - Step 5 (all :-named E) regressed for the same root cause at
    scale.

  **Discipline going forward**: every dumper widening step that
  fails must be paired with a targeted Rust port in
  `latexml_package/src/engine/` (or `latexml_core/` if truly
  kernel-level) of the underlying primitive family. The dumper
  and the engine advance together — the dumper can't admit
  records whose bodies call primitives the engine doesn't execute.

  Immediate primitive candidates from step 4/5 body analysis:
  - ~~`\hook_*:n` family~~ — **resolved via Perl-parity stubs**,
    see the completed l3hooks entry above. Not a native port;
    Perl itself doesn't emulate hook storage.
  - `\group_begin:`, `\group_end:` (already aliased to
    `\begingroup`/`\endgroup` via PA, but blocked from the dump
    under the `:`-key gate — see the "Known antipattern" below
    for why they can't be admitted solo)
  - `\exp_args:N*` family (expansion control)
  - `\__kernel_*` internals used by the kernel setup
  - `\cs_new_eq:NN`, `\cs_gset_protected:Npn`, etc. (cs-aliasing
    and definition primitives)

  Targeted port suggestion: after the hook-stub parity fix, retry
  the step-4/5 widening to re-profile which primitive family now
  dominates the cascade. The hook fix is expected to unblock a
  large fraction of PA/MPA records whose bodies called
  `\hook_gput_code:nnn`; what remains should point to the next
  family to port.

- [ ] **Dump writer ↔ reader simplicity audit** (user directive,
  round 17). Separate question from widening: are both the Rust
  writer and reader as simple and universal as the Perl
  equivalents?

  Current state:
  - Rust writer (`dump_writer.rs`, 525 lines): emits tab-separated
    records from `(TableName, SymStr, Stored)` tuples. Schema
    documented in the file header (M/E, M/PA, M/T, M/R, M/N, V,
    C, LC/UC/SC/MC/DC). All state types covered.
  - Rust reader (`dump_reader.rs`, 950 lines): text parser with
    gates, deferred-alias retry pass, type dispatch.
  - Perl writer+reader (`LaTeXML::Core::Dumper.pm`, 392 lines):
    emits **Perl source code**. The dump is a `.pl` file that's
    `require`-d. Each record becomes one sub call — `I(defn)`,
    `V(key, value)`, `Lt(key, target)`, `N(...)`, etc. No parser,
    no gates. Writer and reader share one object model with the
    runtime.

  Coverage parity exists. **Conceptual simplicity parity doesn't.**
  Perl's dump is "code, not data"; ours is "data, not code". The
  data choice forces a reader with gates because the reader can't
  execute what it can't safely parse.

  **Proposed direction** (to evaluate):
  - Option A — keep text format but tighten semantics so the reader
    becomes a flat dispatch table with no gates. Each record's
    safety is the writer's responsibility (don't emit what can't
    load); the reader just applies each record to the state.
  - Option B — swap the text format for generated Rust source
    (compiled via build.rs into a kernel_dump.rs module). Zero
    runtime parsing, structural typing of records. Mirrors Perl's
    code-not-data philosophy; downside is dump regen requires a
    build.
  - Option C — hybrid: keep text for arxiv-sandbox dumps (portable
    across developer machines), compile to Rust for the kernel
    dump that's embedded in the binary.

  Cross-ref: the Deep Dumper-Reader Parity task below focuses on
  runtime behavior (which records load cleanly); this task focuses
  on architectural simplicity of the pair itself.

- [ ] **Deep dumper-reader parity audit** (user directive round 17).
  Parallel to the expl3 task below. Perl's `LaTeXML::Core::Dumper`
  is 392 lines, single-dispatch, no special cases: each dump record
  just calls `assign_internal($STATE, $table, $key, $value, 'global')`.
  Our `dump_reader.rs` is 950 lines with three gates
  (`is_at_internal`, `is_public_register`, `is_safe_let_alias`), a
  deferred-alias retry pass, and explicit rejection of `:`-named
  entries. The gap isn't in the data — both sides speak the same
  tab-separated format. The gap is in the **runtime semantics
  downstream of the reader**: every gate exists because enabling it
  triggered a cascade (undefined-CS recovery loop, infinite expl3
  expansion, etc.). Systematic path to parity:

  **Step 1 — enumerate and classify the current gates**:
  - `is_at_internal` — `@`-named CS, no `:` in key. Admits
    `\@tabacckludge`-style aliases. Should always be safe.
  - `is_public_register` — data starts with `R\t…` (CharDef /
    Register). No hook/cascade risk; always safe.
  - `is_safe_let_alias` — PA/MPA where neither key nor target
    contains `:`. Recovers ~170 plain-LaTeX public aliases.
  - Everything with `:` in the key — blocked. Includes ~8,914 M
    entries and ~156 PA entries in our current latex.dump.txt.

  **Step 2 — attempt narrow widenings, one class at a time**,
  with the 83_expl3 test as the canary. Round-17 confirmed that
  widening PA-with-colon-name-to-non-colon-target alone regresses
  the test into an infinite loop because expl3.sty's own guards
  misfire. PA and `:`-style M entries must be widened together,
  and the expl3_sty.rs short-circuit must know about the new
  coverage.

  **Step 3 — for each regression, identify the specific runtime
  feature missing in core/engine/package** and port it properly:
  - If the symptom is "undefined CS" on a `:`-named expl3
    primitive, port that primitive in `latexml_package/src/engine/`.
  - If the symptom is a hook cascade, understand the hook
    mechanism and port the hook primitives (`\hook_gput_code:nnn`,
    `\hook_use:n`, etc.) natively.
  - If the symptom is expansion looping, trace which forward-ref
    closes the loop in Perl but not in Rust.

  **Step 4 — when a gate is demonstrably not needed (all records
  of that class load safely), remove it**. Target end state:
  `dump_reader.rs` down to ~400 lines, no special gates, single
  uniform dispatch matching Perl's streamlined pattern.

  **Acceptance**:
  1. `dump_reader.rs` line count halved (950 → ~400-500).
  2. All three custom gates removed.
  3. Deferred-alias retry pass no longer needed.
  4. Full dump loads without error; every M/PA/C/LC/UC/SC/MC/DC
     record round-trips cleanly.
  5. Byte-identical latex.dump.txt consumption produces matching
     state between Perl's `assign_internal` path and Rust's.

  Tracks alongside the expl3 kernel parity task — the two share a
  common root (faithful runtime semantics for everything the dump
  can contain). Work here often directly unblocks work there.

- [ ] **Deep expl3 / LaTeX 3 kernel parity** (round 17 directive,
  `58617b6b6` diagnostic). Goal: `\usepackage{lipsum}` — or any
  other expl3-first package — loads cleanly without error
  suppression, catcode safety-nets, or dump-loader gate
  exceptions. Subsequent package loads (tikz, etc.) should
  compose cleanly.

  **Evidence of current gaps**:
  - `expl3_sty.rs` sets `SUPPRESS_UNDEFINED_ERRORS`,
    `SUPPRESS_UNEXPECTED_ERRORS`, and `set_suppress_log_output`
    around the raw `expl3-code.tex` load. Forward-ref errors are
    being swallowed rather than avoided.
  - `expl3_sty.rs`, `xparse_sty.rs`, `l3keys2e_sty.rs` all have
    catcode restoration safety-nets after raw-file loading (space,
    underscore, at, colon). If `\ExplSyntaxOff` worked correctly
    these wouldn't be needed.
  - `dump_reader.rs` blocks every `:`-named PA alias from loading,
    even trivial ones like `\group_begin: PA \begingroup` — reflects
    low trust in the downstream expl3 machinery.
  - 1611.04489's failure cascade (`\group_begin:` on the stack
    persisting through pgfutil-common.tex) is a *symptom* of the
    partial port, not an isolated pgfkeys bug.

  **Acceptance criteria**:
  1. `\usepackage{lipsum}` loads with 0 errors, 0 warnings,
     without the SUPPRESS_* flags or catcode safety-nets.
  2. `\usepackage{lipsum}\usepackage{tikz}` loads both cleanly
     and neither leaves state residue.
  3. `dump_reader.rs` gate can admit `:`-named aliases that point
     at safe targets (trivial primitives, not expl3 hooks) without
     the current "both-non-colon" guard.
  4. arxiv:1611.04489 / 1511.00722 / 1612.08368 convert with the
     sandbox-expected error profile (which matches Perl's — for
     1611.04489 that's 2 warnings, exit=0, 25 s).
  5. Broader sandbox: any paper currently aborting with `\pgfkeys`
     / `\pgfmath` undefined-CS cascades recovers. Session 128
     counted these collectively as one of the three dominant
     remaining abort classes (~8 papers in the 14-abort
     residual).

  Deferred — deep work. Requires a careful expl3-code.tex
  run-through, identifying each forward-ref / hook / cascade that
  currently breaks, and porting enough kernel primitives that the
  raw loading succeeds on its own merit.

  **Known antipattern to avoid** (round-17 experiment):
  widening the `dump_reader.rs` gate to admit `:`-named PA aliases
  like `\tex_let:D PA \let` IN ISOLATION (without also admitting
  `:`-style M entries and adjusting the `expl3_sty.rs`
  short-circuit) regresses the 83_expl3 test into an infinite loop
  / 60 s timeout. The mechanism: `\tex_let:D` gets let-aliased to
  `\let` → `expl3.sty`'s own guard thinks expl3 is "loaded" →
  skips raw `\input expl3-code.tex` → post-guard code hits
  `\__kernel_dependency_version_check:Nn`, `\ProcessOptions`,
  `\keys_define:nn { sys }` which our gate still doesn't define
  → undefined-CS recovery loop. PA widening and M widening must
  land **together**, coordinated with the expl3_sty.rs
  short-circuit logic.



- [ ] **Rationalize pragma / semantics / grammar categories from first
  principles.** Observation from round 17: many of the recently-added
  pragmas (ConsistentLetterBlocks, AdjacentNumbersDontMultiply,
  FencedLettersAreFunctionArguments, HigherOrderInvisibleOpsAreExceptions,
  FlattenSimpleInvisibleTimesChains, …) are downstream *guards* that
  correct for grammar over-expression — they exist because the Marpa
  grammar admits parses that would not even be theoretically
  reachable under a better-factored categorical hierarchy of
  mathematical notation. `xy` as "function application vs
  multiplication of two bare letters", `(x)` as "fenced expression vs
  single-arg call", `f × f` as a flat binary product etc. are not
  genuine mathematical ambiguities — they are artefacts of conflating
  role-bearing categories (operator, function, coefficient, variable)
  at the same lexical surface.

  Near-term (done): enable the pragmas that already encode the correct
  preferences so the forest is narrowed before post-processing.

  Long-term: sit down with `docs/MATH_GRAMMAR_FIRST_PRINCIPLES.md` and
  redraw the boundary between (a) what the grammar produces, (b) what
  semantic enrichment (role, fences, NUMBER/ID/OPERATOR distinctions)
  adds, and (c) what pragmas legitimately prune. The goal is to push
  work *earlier*: if a phenomenon can be made unreachable by a
  sharper category (e.g. "bare letters cannot be operators unless
  lexically marked as such"), the pragma that guards against it
  becomes obsolete and can be deleted. Pragmas should only remain for
  *genuine* semantic ambiguities that notation cannot disambiguate
  — the four remaining hotspots listed above are candidates
  (`\sin[XY]`, `tr ρ`, `FGHa`, `a|a|+b|b|+c|c|`).

  Deliverable: a design doc (probably extending
  `MATH_GRAMMAR_FIRST_PRINCIPLES.md`) that classifies every current
  pragma into {obsolete under redesign, still needed for genuine
  ambiguity, still needed as engineering compromise}, with a migration
  plan for each. Not scheduled — tracked here so the categorical
  rethink isn't forgotten when the near-term pragma list stabilises.

- [ ] **Rationalize the `Stored` enum** (`latexml_core::common::store::Stored`).
  Nearly every step in the core business logic assigns or looks up a
  `Stored` value through the state API — macro bodies, counters,
  graphics paths, keyval stores, registers, font specs, the lot.
  Because the enum is the universal value currency, its memory
  footprint and method-dispatch cost is a first-order driver of
  overall conversion speed and RSS.

  Round-17 observations supporting this:
  - `arena::to_string(sym)` heap-allocates each time — motivated the
    zero-alloc `state::graphics_paths_contains` helper landed today
    (`<round-17>`), but the same waste pattern recurs across every
    caller of `get_graphics_paths` / `get_search_paths` / etc.
  - `Stored::clone()` shows up in callgrind as a non-trivial band
    (session 116-117 already cut it from 1.02% → 0.17% by routing
    hot paths through `state::with_meaning`, but the remaining 0.17%
    is still a per-token cost at conversion scale).
  - The variant set has grown organically (`String`, `Strings`,
    `VecDequeStored`, `Number`, `Dimension`, `Glue`, `MuGlue`,
    `RegisterValue`, `Tokens`, `Expandable`, `Meaning`, …) with
    ad-hoc packing; likely overlap we could coalesce.

  Proposed investigation:
  1. Measure `size_of::<Stored>()` and the histogram of live-instance
    variants on a representative paper (complex/si.tex and a
    math-heavy paper).
  2. Split hot variants into `Copy`-able small forms vs heap-carrying
    forms; consider an enum-with-small-variants pattern
    (`SmallStored` fitting in a `u64`/`u128`).
  3. Add `with_*`-style closure accessors for every variant that
    currently hands out owned data — mirror the `state::with_value`
    refactor done for `lookup_value` (D4 tracks that separately).
  4. Audit method dispatch: many `match`-on-variant call sites can be
    pushed into inherent methods on `Stored` (`as_string()`,
    `as_int()`, `as_tokens()`) with inlined fast-paths.

  This is a high-payoff but invasive refactor — deferred until D4
  allocation hotspot work has more per-variant data to guide the
  redesign. Not scheduled.

### Other structural follow-ups

- [ ] **Dump Let-alias preservation.** Perl serialises
  `Lt('\cs','\target')` (Let alias) separately from full Expandable
  `I(E(...))` records; our Rust dump collapses both to `M E`, which
  forces the loader's safety gate at `dump_reader.rs:177-191` to admit
  *all* public-CS M records (cascades expl3/hook bodies) or *none*
  (misses plain Let aliases latex.ltx relies on, e.g. `\let\a=\@tabacckludge`
  at L10007). Workaround: explicit `Let!("\\a", ...)` in
  `latex_constructs.rs`. Proper fix: new `L <cs> <target>` record type
  with a narrow loader gate. Would recover `\filecontents`, `\fbox`,
  `\itshape`, `\ae`, `\shipout`, etc. wholesale.

### Future-facing / not-wired exploration

The following designs are **intentionally kept out of the active
engine wiring** — they describe beyond-Perl directions worth
revisiting once the parity baseline is cleaner. Not loaded, not
referenced by any compiled code path.

- [ ] **Native l3hook storage** (post-parity beyond-Perl direction).
  Perl's bindings handle l3hooks as no-op stubs (see the completed
  "l3hooks — Perl-parity stub port" entry above). A richer Rust
  implementation would actually store hook code per name, fire it at
  `\hook_use:n{…}`, and let `\AtBeginDocument` / `\AtEndDocument` /
  `\AtBeginEnvironment` route through it. Sketch:

  1. `\hook_new:n{name}` — declare a hook (lazy: first `gput` creates).
  2. `\hook_gput_code:nnn{name}{label}{code}` — `state::push_value("@l3hook:{name}", code)`.
  3. `\hook_use:n{name}` — `state::with_vecdeque` + digest each.
  4. `\hook_if_exist:nTF{name}{T}{F}`.
  5. Parallel: `\AddToHook` → maps to `\hook_gput_code:nnn`, etc.

  **Why not now**: Perl doesn't do this, so adopting it inside the
  core engine risks divergent render output whenever a package
  registers code into a hook that Perl silently drops. Any pursuit
  should be behind a feature flag (e.g. `LATEXML_OXIDE_L3_HOOKS`)
  and validated with a dedicated test corpus, not the parity test
  suite.

  **Placement when pursued**: a new standalone crate or a
  `latexml_package/src/engine/latex_lthooks.rs` module behind a
  `#[cfg(feature = "l3hooks")]` flag. Engine wiring must NOT be
  added to the default path.

  **Prerequisites before any of this is reasonable**:
  - Dumper staircase complete / dump_reader simplified (this
    unblocks the ambient test surface).
  - A purpose-built hook test corpus with Perl/Rust A/B parity to
    show the cases where "store and fire" changes output vs
    "silently drop" — and confirm the changes are always
    improvements, never regressions.

---

> **Reminder:** Every entry ported from Perl must follow tightly the original semantics and nuances. Read the Perl source, translate precisely, preserve edge cases. The Perl code is the ground truth.
