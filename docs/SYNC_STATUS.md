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

1. **pgfkeys.code.tex port gap** — 3+ papers (1511.00722, 1611.04489,
   1612.08368). `pgfkeyslibraryfiltered.code.tex` from TeXLive triggers
   `\pgfkeys@non@outer@newif` / `\pgfkeysalso` / `\ifpgfkeyssuccess`
   undefined-CS cascades that loop until the 60s wall-clock caps. Perl
   also loads the raw TeX (via `InputDefinitions('pgfkeys.code', type=>'tex')`)
   but succeeds — divergence is in how our raw-TeX processor handles
   specific pgfkeys idioms, not a simple stub gap.
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

- [ ] Capture Tier A (~10 papers) + `complex/si.tex` as a standing perf
  corpus in `docs/PERFORMANCE.md`. Regression trigger: wall-clock drift
  > 15% on any corpus entry between commits.

Tier A/B/C slow-paper backlog is resolved (all <4s direct wall-clock after
sessions 116–124). Specific slow-convergence follow-ups:

- [~] **1709.05096** — >90s wall under parallel load (digestion, not
  post-processing). `Info:undefined:… KV:vattach …` scanning pattern
  suggests a keyval loop paying per-row cost in a huge tabular.
- [~] **1710.03688** — OOM at ~19 GB RSS during babel french.ldf load;
  `\bbl@exp@aux` undefined (babel 3.x port gap).

### D3b. Stability — libxml2 node lifetimes

**Policy:** no direct `libxml::bindings::*` FFI calls from latexml. When
a safe API is missing, add it to `~/git/rust-libxml` and
vendor-patch/upstream. Current direct `libxml::bindings::*` call sites:
`latexml_core/src/lib.rs`, `latexml_core/src/document.rs` — 5 total.

- [ ] Route libxml node lifetimes through guardian forbidding unlink without cache invalidation.
- [ ] Replace unsafe-over-FFI with safe wrappers where practical.
- [ ] Migrate the remaining `libxml::bindings::*` callers to high-level
  `rust-libxml` methods; upstream new methods as needed.
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
  — 248 initial sites, ~12 converted (mathchar, pin_char, defined_as).
  Remaining: `state.rs` (17), `binding/content.rs` (5), `keyval.rs` (4,
  tricky — return `Option<Stored>`), `binding/counter/dialect.rs` (3).

### D5. Math parser ambiguity

Callgrind (math-heavy paper, session 105): Marpa dominates — transitive
closure 34.3%, grammar precompute 8.3%, bv_scan 7.1%, AVL 6.8%.
Marpa-related >60% CPU.

- [ ] Avoid `init_grammar()` fallback — reuse existing grammar on reset failure.
- [ ] Audit script attachment ambiguity (`{}^4{}_{12}C^{5+}` — 27 unique trees).
- [ ] Early pruning: fail parses on inconsistency detection rather than post-hoc pragmas.
- [ ] Enumerate grammar rules by parse-tree count contribution.
- [ ] Document grammar ambiguity per category.

Remaining semantic-ambiguity hotspots (see
`docs/MATH_GRAMMAR_FIRST_PRINCIPLES.md`; live audit via
`LATEXML_PARSE_AUDIT=1`):

1. `\sin[XY]` chain — 1022 trees / 10 unique (real semantic ambiguity)
2. `tr ρ / tr(XY) / rank M / …` — 100 / 8 unique
3. `FGHa` OPFUNCTION cascade — 87 / 9 unique (genuine math ambiguity)
4. `a|a|+b|b|+c|c|` VERTBAR — 53 / 10 unique

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

---

> **Reminder:** Every entry ported from Perl must follow tightly the original semantics and nuances. Read the Perl source, translate precisely, preserve edge cases. The Perl code is the ground truth.
