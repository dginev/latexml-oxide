# Engine Sync Status — Active Worklist

> **DO NOT downgrade Errors to cheat the task.** If Perl LaTeXML converts a paper
> without a downgrade, the Rust translation must match by improving the core
> engine — never by silencing diagnostics. New downgrades require explicit proof
> Perl emits the same severity on the SAME paper, else they hide a real gap.
> (User directive 2026-05-15.) Always classify with `latexml --verbose`, never
> `--quiet` (which hides Perl's `Error:` lines); cross-check pathological inputs
> with `pdflatex`.

## How to read this file

**Start at "Ranked worklist" below and take the top unblocked row.** That is the
whole intent of this file; everything after it is supporting detail.

| section | what it is | when you read it |
|---|---|---|
| **Ranked worklist** | every open item, ordered, with size + where the detail lives | **first, always** |
| Current status | suite count, the last session, release state | to orient |
| Open items | the detail behind the ranked rows | when you pick that row |
| Standing policies | rules that constrain *how* you fix things | before adding a CLI flag, a stub, or a divergence |
| Parked families | pointers to four extracted docs | only when starting that family |
| Reference | stable facts, not work | when something surprises you |

Three rules that keep this file honest:

1. **Verify a status label before acting on it — and before deleting it.** Four
   entries here have pointed at work that did not exist: a `13 commits NOT
   PUSHED` banner (merged as PR #323), a "#312 → render under MathJax 4" step
   (issue closed; that screenshot was out of scope), a
   `CLI options (#191) — ACTIVE` heading (issue closed), and a "PR #310 … ready
   to merge" line (already merged). Check the **named guard test** in the tree,
   or `gh issue view <N>` / `gh pr view <N>`. **SHA-ancestry does not work** as a
   check — the repo squash-merges, so a branch SHA quoted here is never an
   ancestor of `main`.
2. **This is the BRIEF ACTIONABLE LIST.** Day-by-day logs live in `git log` and
   `docs/archive/`. When you close an item, delete it here and lift anything
   worth re-reading into `docs/archive/SYNC_SESSIONS_YYYY-MM.md`.
3. **Keep it under ~500 lines.** When a section outgrows ~100 lines it has become
   its own subject — give it a doc under `docs/` and leave a one-line pointer.

*Last compaction: 2026-07-25 — 1979 → ~500 lines. 23 completed sections lifted to
`SYNC_SESSIONS_2026-07.md`; four standing families extracted (see Parked
families).*

## Ranked worklist — start here

Ordered by: **does it reproduce today** → **is a real user affected** → **is it
unblocked** → **effort**. R1 is a review nudge (no code); R2's cheap half landed
2026-07-29 and what remains of it, like the R5+ rows, needs a session of its own
(R4 itself is ✅ FIXED). Re-verify a row before planning on it (rule 1).

| # | item | state | size | detail |
|---|---|---|---|---|
| **R1** | Upstream `brucemiller/LaTeXML#2852` — subfile `\documentclass` options | **OPEN upstream**, ours merged as #310; **CI all-green + mergeable, re-verified 2026-07-29** | nothing left but a review nudge — no code, no automatable step | Open items |
| **R2** | `--preload=<cls>` trips the LaTeX hook stack (`Extra \PopDefaultHookLabel`) | **OPEN**, re-verified 2026-07-29 (1 error with `--preload=article.cls`, 0 without). The row's *second* divergence — the preload PI kept `[opts]`/`.cls` and never emitted `options=` — is ✅ **FIXED 2026-07-29** | hook half is **not** small: five measured dead ends, `(c)` now collapsed into the rejected `(a)`, and any real fix is TeX-side | Open items |
| **R3** | **Bibliography-absence campaign** (PR #444) — **16 fixes landed**, **291 of the 533** known articles recovered / 20 338 entries, re-verified by reconversion. **242 still empty, all characterized** — plan R3a-R3g below. Corpus scope 50 777 | **R3a next** | per-item | [`BIB_ABSENCE_AUDIT_2026-07-29.md`](parity/BIB_ABSENCE_AUDIT_2026-07-29.md), [`RESIDUAL.md`](parity/bib_absence_2026-07-29/RESIDUAL.md) |
| **R4** | biblatex `.bbl` `TokenLimit` loop (2605.17646) | ✅ **FIXED 2026-07-25** — self-referential `\let` on `setupPseudoBibitem` re-arm; shared with Perl | — | Open items |
| **R5** | Bibliography targets + MakeBibliography re-port | **the re-port is DONE** — items 1 and 3 landed 2026-07-26/27 (recursive BibTeX session on the LIVE core state, the 727-line string route deleted, the 13-field digest whitelist gone: the `\bib@field@default@*` name sets match Perl exactly, 45 each; `.bib`-as-DATA closed as divergences #74/#78/#79/**#80**), and **item 2 landed 2026-07-29** (citestyle `AY`, short-name `{ay}`, collating `unisort`, format-order NUMBER). Remaining: the missing-references target list | **targets only** | [`BIBLIOGRAPHY_WORKLIST.md`](parity/BIBLIOGRAPHY_WORKLIST.md) |
| **R3** | Presentation-MathML **F17 ✅ CLOSED 2026-07-29**; **F5** Linebreaker still open | F17 fully settled: 4 fixed (`pmml_text_aux`, `outerWrapper` altimg/RDFa + the missing `CrossRef::fill_in_RDFa_refs`, `pmml_scriptsize_padded`, `preprocess` plane1 + new `--plane1`/`--noplane1`/`--hackplane1`), 3 closed as do-not-port/N-A (ADDOP flatten is dead in Perl too — porting would DIVERGE), `combineParallel` BLOCKED on the absent `--openmath`/`--mathimages`/`--mathsvg`, `nestmath` unreachable in both engines. **What remains on this row is F5 alone.** A math-parser `scriptpos` bug and a FUNCTION-APPLICATION over-insertion witness found en route are **other rows** | **per item, small**; F5 alone is a **family** needing a scope decision | Open items |
| **R6** | `ltx_env_<name>` env-markup class | user-requested, **PHASE 2 — do NOT start yet** (user directive 2026-07-29) | medium code, **large golden churn** → own branch | Open items |
| **R7** | Beyond-Perl performance levers BP-1…BP-6 | POST-RELEASE; internal order BP-2 → BP-3 → BP-1 | **family** | [`BEYOND_PERL_LEVERS.md`](performance/BEYOND_PERL_LEVERS.md) |
| **R8** | Content-MathML / math-parser gaps | **deferred by user directive 2026-06-20** | **family** — do not pick off in isolation | [`CONTENT_MATHML_GAPS.md`](math/CONTENT_MATHML_GAPS.md) |
| **R9** | Deep deferred families (`.bst`, xy-pic, mode-frame, …) | parked; several carry explicit "do NOT start". The `.bst` row's "`.bst` files *vendor macro definitions*" premise was **RETRACTED 2026-07-27** (`alpha.bst` has zero `Dbar`; the macro is `mathscinet.sty`'s) — it survives on label style / sort order / **field selection**, and the prerequisite is a corpus measurement of the `.bib`+`.bst`-with-no-`.bbl` population | **family** | [`DEFERRED_FAMILIES.md`](parity/DEFERRED_FAMILIES.md), and R9-BST below |
| — | `\gls`/`\acrshort` in math mode (1705.10306) | **PARITY, blocked** on unrunnable Perl | — | do not chase; Open items |
| — | **Streaming CORE (fragmented conversion)** | **IMPLEMENTED + PRODUCTION-PROVEN on `feat-streaming-xml-core`** (2026-07-30, one PR pending): stomach yield → spill/placeholder → pass 2 → recursive splice; `--streaming` + auto-when-doomed; byte-identical to eager (suite 1782/0, 8 sweep suites, 19.8 MB witness `cmp`-equal at 613,104,457 B). Leak fix halved eager (rust-libxml unlink never frees → `Node::free_subtree`, libxml **0.3.17 published**; eager 63.7→34.9 GB; streamed@24 GB-cap 16.97 GB/8:28). **The 131 MB Nasser witness CONVERTS: exit 0, 28.1 GB peak @ 48 GB cap, 2.66 GB well-formed XML, 1:10 wall** (node_boxes stale sweep, nested spills stay nested, spilled-id dedup, malloc_trim/mi_collect, token backstop ×200/byte). Remaining error: `{nowrap}` (issue 297 binding). **Perf consolidation 2026-07-31 (`perf-streaming-pass2-segment-coalescing`): 1:10:29 → 32:56 wall (−53 %), log −99.2 %, output md5-identical to base; soft-yield floor + flat spill + shared label index + streaming telemetry phases — see STREAMING_CORE_DESIGN "PERF CONSOLIDATION". Measured split: MathParse 41 % is the only remaining ≥40 % block.** | — | [`STREAMING_CORE_DESIGN_2026-07-29.md`](performance/STREAMING_CORE_DESIGN_2026-07-29.md) |
| — | **POST half: page-major rendering** | **LANDED 2026-07-30 (PR #451)**: the driver was phase-major, so every page stayed alive at every boundary (~1.6 MB/page of `xmlDoc` overhead) — 614 MB core XML with `--splitat` went **80 GB/exit 137/0 pages → flat 15.98 GB/exit 0/40,201 pages/2.25 GB HTML/0 errors**. Same PR: whole-document `//X[pred]` answered by traversal (six queries had been FAILING silently → no MathML, no crossrefs, 0-byte HTML, exit 0), and an unanswerable query is now an `Error`. Markup verified against Perl 0.8.8 on a 10-page fragment. | — | [`STREAMING_POST_DESIGN_2026-07-06.md`](performance/STREAMING_POST_DESIGN_2026-07-06.md) §2a |
| — | **POST half: two-pass streaming split** | **IMPLEMENTED + WITNESS-PROVEN on `feat-streaming-post-split` (2026-07-31)**: the 131 MB witness's post half now COMPLETES on the 31 GB laptop — 2.68 GB core XML → **115,519 pages / 11 GB HTML / exit 0 / 37:31 wall / 17.4 GB peak** (`--splitat=subsubsection --max-memory=26000`; was: OOM during the whole-DOM parse, 0 pages). Mechanism: the last whole-DOM stage (Split's one-time parse — >26.6 GB for the witness's 2.68 GB core XML, OOM on a 31 GB laptop with 0 pages) replaced by a `TextReader` stream that spills pages as text + a pre-order Scan sweep; peak = open-ancestor chain + one content subtree. Byte-identical to the DOM split (guard `118_streaming_split_parity`); auto ≥ 1 GiB file input, `LATEXML_POST_STREAM_SPLIT` forces. Oversized in-memory handoffs also spill at the threshold so the single-invocation `.tex → .htm` flow streams too. **Single-invocation `.tex → .htm` PROVEN 2026-08-02**: the full witness in ONE call — 115,519 pages, exit 0, `Conversion complete` as the last line, 1:16:52 wall, **22.95 GiB peak** vs the 24.58 GiB fuse at `--max-memory=32000` (libxml 0.3.21's `get_namespaces` leak fix collapsed render retention 152 → 3.1 KB/page; render RSS flat at ~14.3 GB across all pages). Log: `~/data/nasser-post-2026-07-31/joint2.log`. The tally residual was fixed by the single-vehicle diagnostics rework (#484): the instrumented re-run (2026-08-03, tag commit `69ec59620f`) reports **`Conversion complete: 12094 warnings`**, exit 0, 23.0 GiB peak, 1:24:25 wall — verdict and log now agree. Fixed en route: DOM split dropped inherited `xml:lang` (namespaced-attr read); **post parses now pass `XML_PARSE_HUGE`** — without it libxml2's per-document dictionary cap poisons the ID table from ~1.47 GB (the witness's 237,732 bogus `ID … already defined` reports, each id occurring exactly once — the rc4 mystery) and the parse dies at ~1.71 GB (`xmllint --stream` reproduces; `--huge` clears); libxml 0.3.18 (`expand_to_document` `default:`-prefix mint + 5 reader APIs — **needs crates.io publish before merge**). A mid-stream split failure now fails LOUD instead of falling back into a guaranteed whole-DOM OOM. Follow-up (perf): flat handoff serialization (50.9 % of witness core XML is indentation). | — | [`STREAMING_POST_DESIGN_2026-07-06.md`](performance/STREAMING_POST_DESIGN_2026-07-06.md) §3 |

### R3 mini plan — the remaining bibliography failures

Detail and per-paper rows: [`RESIDUAL.md`](parity/bib_absence_2026-07-29/RESIDUAL.md).
Red/green for every item is the bibliography length (0 = red); reconvert with
`tools/bib_recheck.sh` and never trust a first-error label alone — 96 of the 123
classes are singletons and the first error is often incidental.

| # | item | papers | state |
|---|---|---|---|
| ~~R3a~~ **19 of 29 LANDED** (`\DeclareCiteCommand` defines its command; `\addbibresource` harvested from a shipped class; **OmniBus now autoloads biblatex from `\addbibresource`/`\printbibliography`** — 7 papers, 548 entries, for classes that load biblatex themselves but have no binding, e.g. now-journal.cls building the package name from macros). Four of those 7 had an unrelated first error, so first-error clustering had scattered one cause across four buckets | **biblatex, document-level** | 10 left |
| **R3b** | **No diagnostic at all** — silent loss, complete document. **13 of 31 LANDED**: `\nocite{*}` now includes the whole library as bibtex does (7), a native `xpatch.sty` binding stopped an expl3 sentinel-delimited scan from eating the document to EOF (5 — audit F12; the 10 xpatch papers' other 5 causes are each unrelated and listed there), and **2606.01320's `\pretocmd{\cite}{\stepcounter{cite}}` now assigns through the core `\cite` CS-lock** (2026-08-05; etoolbox hooks are non-destructive, so they open a scoped unlock window — divergence #88; guard `06_cluster_bibliography::etoolbox_pretocmd_assigns_through_cite_lock`). Of the rest, **14 have real `\cite` calls** (chase these) and **4 cite nothing at all** (0 entries is correct — exclude). One is characterized-but-unfixed with the remedy written down in `RESIDUAL.md`: 2605.08378 (submission ships no `PurdueThesis.cls`). **Batch re-triaged 2026-08-17** (27 no-diagnostic candidates, fresh worker): **4 already fixed** (2606.23302, 2606.30032, 2605.28547, 2605.25157), **2 orphan-`.bbl`** (commented/unreferenced `\bibliography` → exclude: 2606.09394, 2605.03978, 2606.31667), and **one genuine Rust-only fix landed** — **2606.11493 (0→31):** a `comment.sty` `\begin{comment}…text.\end{comment}` with the end MID-LINE overran to EOF and swallowed the inline `thebibliography`; fixed by detecting `\end{comment}` mid-line (surpass-Perl #133, guard `comment_midline_end_keeps_bibliography`). Remaining chase candidates (still-0, real `\cite`, present bib source): 2605.19817, 2606.01320, 2605.14990, 2606.05629, 2606.10056, 2606.17491, 2606.00231, 2605.29754 | **~8 left** | comment fix landed |
| **R3c** | `\ce` inside a `p{}` column leaks a mode → `\@end@tabular` cannot close. 7-line repro in [`repros/f8_ce_in_p_column/`](parity/bib_absence_2026-07-29/repros/f8_ce_in_p_column/). **CONFIRMED SHARED (2026-08-04):** `latexml --includestyles` raw-loads mhchem and gives the byte-identical 7-error cascade (bare Perl was a false-clean — it can't find raw mhchem without `--includestyles`, so `\ce` was inert). **Surpass-only** — mode-robust `\ce` or an mhchem binding, needs the 3 qualifying tests + user escalation; deprioritized | **7** | ✅ triaged → surpass-only |
| **R3d** | **PARTLY FIXED — 14 of 28, 961 entries.** A `&` inside a **delimiter-fenced** macro argument split the alignment row and truncated the document. `tex.web` §394 `macro_call` disables tab marks while scanning parameters; we — and Perl, which raises the identical error — did not. `SuppressedTabMarks` armed inside alignments on physics.sty's `phys_read_arg` fixes the 16 `\mqty` users. The general `Parameters::read_arguments` site is TeX-correct but regresses 5 tests (`cells_test` 17 errors, `numprints_test`, `xytest_test`, `consort_flowchart_test`, `unit_tests_by_silviu_test`) because that path also reads alignment cell content — needs a parameter-scan-vs-cell-read distinction. Divergence #90 | **12 left** | needs the distinction |
| **R3e** | `N bibentries, 0 cited` — citation records never attach, so an empty References heading renders. The raw-`\cite`-clobber half landed as #88; this is the narrower residue | **22** | needs a reproducer |
| **R3f** | Ships **no** bibliography source (`.bbl`/`.bib`/`thebibliography`/biblatex). Verify each against its own PDF, then exclude — the PDF says what the *toolchain* produced, not what we may emit ([`README_residual_triage.md`](parity/bib_absence_2026-07-29/repros/README_residual_triage.md)) | **17** | triage to close |
| **R3g** | amsrefs bare `\begin{biblist}` with no `{bibdiv}` wrapper → `malformed:ltx:biblist`. Two dead ends recorded (naive `auto_open`, and a conditional wrapper) — needs the `BACKMATTER_ELEMENT` route | **4** | dead ends recorded |
| — | No output at all (`Stomach:Recursion`, `Timeout`, `MemoryBudget`) → the general **fatal-mining** mission, not bibliography work | 9 | routed away |
| — | Truncation residue behind R3d, plus a ~96-strong singleton tail of per-template defects | rest | work after R3a-R3g |


## Current status

- **2026-08-02 (latest) — rc4-recut full rerun of sandbox-arxiv-2605+2606 (60,505
  docs): success at parity, every recorded fatal-witness set improved, one new
  REAL cluster (a fleet-only libxml panic ×3) and one REAL seed (Perl-0-vs-
  Rust-101 error floods).** Fleet: fresh `maxperf-cortex` worker on main
  `69ec59620f`, 72 workers, both corpora drained in ~1 h. Headline (2605 /
  2606): no_problem 6,078/6,359 · warning 19,744/19,724 · error 3,991/4,102 ·
  fatal 266/241. The huge `no_problem→warning` shift (2605: 16,791→6,078 np,
  9,043→19,744 warn vs the prior run) is **#484's lossless diagnostic tally
  recording mostly math warnings that were previously lost — visibility, not
  regression** (owner-confirmed; do not chase).
  **Witness sets** (per-id via `api/corpus/…/document/`): the 17
  `Fatal:TooManyErrors` papers are now ALL plain `error` at 1–10 errors; the
  bxcoloremoji 18/18 non-fatal (2605.14271 at its recorded post-fix 12); the
  `\lx@tag@intags` pair resolved (2605.12842 warning; 2605.01731 one unrelated
  `undefined:\nequiv`); the flip-4 all `warning`/0-error.
  **Fatal clusters** (sampled 8 each, `parity_check.sh` 180 s, protocol
  cluster-classify):
  | cluster | size | verdict |
  |---|---|---|
  | `panic:caught` | 3 | **REAL — ROOT-CAUSED + FIXED 2026-08-03.** All three (2605.08935, 2606.01083, 2606.22705) panic at libxml-0.3.21 `node.rs:1292` `ptr_as_option` `.upgrade().unwrap()` on a dead docref. Cause: the pooled worker reuses threads with NO thread reset (`cortex_worker.rs:721-729`, deliberate — persisted definitions), and the math parser's `PENDING_DISCARDS` thread-local (deferred-free `Vec<Node>`, `data.rs`) is drained only AFTER the formula loop — the resource-fatal abort path (`parser.rs` Err route) returned early without draining, so the NEXT paper on that thread walked handles into the prior paper's freed document. **The panicking papers are innocent — the poison is the predecessor's.** Only 3/60k because the walk's UAF read must find a surviving non-null child pointer (verified: a clean-heap repro reads empty and does NOT panic — traversal is UAF-content-dependent), and `take_pending_discards` empties the queue before walking, so each incident self-heals (no `PANIC_STREAK` cascade). Fix: drain on the abort path while the document is alive + a wrapper-only entry sweep (`sweep_stale_math_state`: `set_linked()` each stale handle — pure bookkeeping, no FFI on the dead tree — then drop; `Info:cleanup:stale_math_state`, never an error against the innocent paper). Guard: `data::tests::stale_handles_from_a_dead_document_are_swept_without_panic`. **Same-class residuals (recorded, unfixed):** `ALIGNING_NODE`/`ALIGNING_PREV_CHILD` (`latex_constructs.rs:600-651` — build-time `Stored::Node` never popped; `\@add@flushright` at :5225-5240 reads it bare), `Stored::Alignment` cell nodes (`state.rs:1796` / `cell.rs:44`, single clear at `tex_box.rs:1369`), latent `STAGED_SNAPSHOTS` (`state.rs:3659`). Fork-side `ptr_as_option` hardening (dead docref → `None` + debug assert) remains an optional defense-in-depth, owner's call. |
  | `TooManyErrors:MaxLimit(100)` | 117 | **MIXED, REAL seed** — 4/8 REAL incl. **2605.22927 and 2606.11121: Perl 0 errors vs Rust 101-capped flood** (also 2606.01136 P63/R101, 2605.10685 P7/R101); 3/8 both-capped (uncomparable), 1 Rust-win. Root-cause the two P=0 witnesses first; cap-bucket ⇒ sub-group by first-error class before scoping. |
  | `Stomach:Recursion` | 55 | **MIXED** — 3/8 REAL-by-count (2605.17696 R144/P56, 2606.05321 R35/P15, 2606.08524 R94/P50), 2 Rust-wins, 1 Perl-capped, 1 shared-timeout, 1 clean. NOTE: none hit the recursion *fatal* locally — the fleet's memory-governed guard levels are part of the trigger; sub-group by next-error class. |
  | `Timeout:PushbackLimit` | 120 | **Environmental/policy** — 3/8 both-clean locally (same binary!), 3 Rust-wins/shared, 1 trivial R2-vs-P1. Fleet budget caps, not conversion bugs. |
  | `Timeout:TokenLimit` | 88 | **Not a bug cluster** — 7/8 Rust-wins (P 3–101+ vs R 0–2) but 6/8 with local `RUST_TIMEOUT` partials: legitimately heavy papers, Rust cleaner-but-slow; perf domain. |
  | `cortex:never_completed` | 26 | **Fleet-environmental** — 0/8 REAL, 5/8 Rust-wins, 2 both-clean; the known pattern. |
  Rust-wins recorded this round (do NOT "fix"): 2606.31920, 2605.27614,
  2606.28450, 2605.16450, 2605.03102, 2606.28253, 2605.16752, 2606.19764,
  2605.22335, 2606.26525, 2606.15193, 2606.25331, 2606.31502, 2606.28434,
  2605.26647, 2605.26895, 2605.15001, 2606.18685.
  **Actionable queue from this round**: (1) the libxml panic teardown (above);
  (2) `2605.22927`/`2606.11121` Perl-0-vs-Rust-101 floods; (3) the
  Stomach:Recursion REAL trio. Everything else is policy/perf/upstream.

- **2026-07-30 — a font selected by FAMILY decoded through OT1, emitting the
  slot's TEXT character, at zero errors.** Rust's `\selectfont` was missing Perl's
  middle branch (`latex_constructs.pool.ltxml` L5207-5209): when the family is not a
  known typeface, `LoadFontMap($family)` and, on success, `MergeFont(encoding =>
  $family)`. `bbding`'s `\dingfamily` is `\fontencoding{U}\fontfamily{ding}\selectfont`
  and no `u.fontmap` exists, so that branch is the *only* path from family `ding` to
  `ding_fontmap` — which was therefore **dead code**, correct table and registered
  loader notwithstanding. Every `\@chooseSymbol{N}` fell to the OT1 fallback: `'045`
  (`\XSolidBrush`) became a literal `%`, `'041` (`\Checkmark`) became `!`.
  **Invisible to every log-based signal** — witness 2503.04421 converted at
  `Status:conversion:0`, zero `Error:`/`Warning:`, telemetry `errors:0`, while 28 cells
  spanning the "pretrained?" column of *both* its main results tables inverted their
  meaning. Fixed with the branch plus Perl's three `reported_unrecognized_font_*`
  report-once guards (`already_reported`, `latexml_engine/src/base_utilities.rs`), which
  had turned one unrecognized family into 28 identical `Info` lines; the same guard on
  `\lx@fontencoding` takes `Info:missing_font_encoding:U` from 28 to 1.
  Post-fix the witness is **99.92% token-identical** to same-host Perl 0.8.8 (10 trivial
  hunks: an affiliation comma, invisible-times placement) with dingbat counts matching
  exactly, 15 ✗ / 13 ✓ on both sides. Guards:
  `latexml_oxide/tests/fonts/bbding.{tex,xml}` (golden verified against Perl) and
  `tests/116_bbding_family_fontmap.rs`; method + the `pdftotext`-agrees-with-the-bug
  trap in [`WISDOM.md`](parity/WISDOM.md) §80.
  Still open, same defect class: `DeclareFontMap`'s
  `(uppercase|lowercase|digit)_mathstyle` options are unported — Rust writes
  `OMS_uppercase_mathstyle` and `amsb_fontmap.rs` comments out a dropped
  `uppercase_mathstyle => { family => 'blackboard' }`, but nothing reads either key.

- **2026-07-29 — an arg-taking `\fnum@<type>` absorbed the rest of the document
  into an unclosed `<figure>`.** The one concrete, analysed-but-unfixed item of
  R5's "missing references" family, parked since 2026-07-14 on "needs a user
  decision + a full-suite diff". Both are now done: user-approved per the
  `surpass-perl` protocol, and the full-suite diff is **empty**.
  LaTeX's `\@makecaption` is `\sbox\@tempboxa{#1: #2}`, so the widely-copied
  "`Fig. 1:` → `Fig. 1.`" hack — `\renewcommand*{\fnum@figure}[1]{...}` — works
  under pdflatex by having the hook eat that `:` **token**. LaTeXML's separator
  is a tag **attribute** (`\lx@tag[][: ]`), so the argument scan ran past the
  hook and took the caption group's closing brace: the `<figure>` never closed
  and every following section, **the bibliography included**, was absorbed into
  it. That is the truncation mechanism, not a bad caption.
  Fixed by expanding the hook as `\csname fnum@#1\endcsname{}` at all three
  `fnum@` sites (`\lx@fnum@@`, `\lx@fnum@toc@@`, the theorem-header formatter);
  `\lx@typerefnum@@` shares the shape and is deliberately excluded — no LaTeX
  kernel feeds it a separator token. **SHARED-FAILURE, so this is a surpass**:
  pdflatex 0 errors, same-host Perl 0.8.8 **9**, pre-fix Rust **7** on the
  two-hook minimal form; the three-hook guard fixture goes **10 → 0**.
  **The deferral's premise was wrong and that is the lesson**: "`\lx@fnum@@`
  formats every figure/table caption in every document" — it does not. The
  changed branch fires only where `\fnum@<type>` is *defined*; the untouched
  `\lx@@fnum@@` default serves nearly every caption. Measured blast radius:
  **106/106 targets, zero goldens re-blessed**, and three real papers whose
  classes `\def\fnum@figure` (svjour3, aastex631, llncs) are byte-identical
  before vs after.
  **A second recorded claim did NOT survive re-measurement.** The 2026-07-14
  note said "18 papers corpus-wide, 5 with no References", counted by
  `grep 'lx@tag@intags'`. Live against the current fleet run that proxy gives
  **23** papers over 2605+2606 (60,505 docs), but only **2** carry this cause's
  actual signature — 2605.01731 (18 figures × 3 errors, confirmed live) and
  2605.12842 (10 × 3). `\lx@tag@intags` has several causes; the proxy
  over-attributes, and the "5 with no References" sub-claim is withdrawn. The
  fix is justified on being right and free, not on breadth. Does NOT fix the
  `close=": "` separator, so the caption still reads `Figure 1.: A caption.`.
  OXIDIZED_DESIGN **#85**, KNOWN_PERL_ERRORS **#68**, guard
  `06_cluster_regressions::cluster_fnum_arg_hook`.

- **2026-07-29 — `\bibliographystyle{alpha}` produced the wrong label
  shape, and duplicate author-years were never disambiguated.** R5 item 2, the
  four secondary `MakeBibliography` parity gaps, all in
  `latexml_post/src/make_bibliography.rs`. The reaching one: Perl branches on
  `citestyle` three ways (L481-517) and **`AY` is the abbreviated `[AS64]`
  label**, class `ltx_bib_abbrv` — Rust read `AY` as the spelled-out
  author-year and `alpha`, a string nothing emits, as the abbreviated one, so
  every `\bibliographystyle{alpha}` document got author-year refnums (and
  natbib's `super` fell to numbers instead of author-year). Second: Perl keys
  disambiguation and the split bucket off the SHORT name form (`"Smith et al"`,
  L326-337) and only the SORT off the full names; Rust used the full names for
  all three, so two 3+-author entries sharing a first author and year never
  collided and **neither got its `a`/`b` suffix**. Third: `unisort` collates
  (`Ångström` belongs between `Adams` and `Baker`, not after `Smith`) — ported
  at UCA's primary level with no new dependency, divergence **#84**. Fourth:
  `NUMBER` is assigned in FORMAT order, which is initial-major under
  `--splitbibliography`, not in document-global sortkey order (non-split output
  unchanged). Also fixed because the citestyle repair makes it reachable for
  every alpha document: `make_alpha_label` byte-indexed the per-author initials
  (`&aa[..3]`), a char-boundary panic on `Ångström`.
  **One of the four was NOT a gap** — `Formatter::Year` correctly omits the
  suffix. Perl's `do_year` reads the ARRAY `@…::SUFFIX` while `formatBibEntry`
  binds the SCALAR `$…::SUFFIX`, so the letter never reached the body upstream
  either; measured Perl prints ` (1999)` and `alpha.bst` agrees.
  **KNOWN_PERL_ERRORS #67** — the audit item was read off the sigil, which is
  the same "verify the file, not the recollection" trap R9-BST already records.
  Every expectation ground-truthed against same-host Perl 0.8.8 on the fixture,
  after which the two engines' bibliographies are byte-identical there. Guard
  `06_cluster_bibliography::cluster_bib_alpha_style_labels` (verified RED on
  the pre-fix tree: author-year classes, `Ångström` last, no suffixes).
  **Found, then fixed 2026-08-04:** a Rust `<ltx:biblist>` had `xml:id` but no
  `fragid`, and `add_id` emits the HTML `id` from `@fragid` only, so Perl's
  `<ul id="bib.L1">` was a bare `<ul>` here. The `fragid` assigner is
  `CrossRef::fill_in_frags` ("Any nodes with an ID will get a fragid",
  CrossRef.pm L312-324) — faithfully ported — which stamps only nodes that have
  an `ID:<id>` ObjectDB entry. Perl's Scan runs AFTER MakeBibliography and
  registers every id'd node; our port hand-registers the *bibitems* at that
  seam (because Perl's Scan is what registers them) but never the enclosing
  list, so the list alone had no entry. MakeBibliography now registers each
  `ltx:biblist` too. Output `<ul id="bib.L1" class="ltx_biblist">` is
  byte-identical to same-host Perl. Guard:
  `06_cluster_bibliography::bib_entry_ids_are_bib_rooted_like_perl`.

### Session logs (2026-06-22 … 2026-07-27) — ARCHIVED

Completed "Landed this session" entries, the slowest-100 batch triage, the
finished upstream-sync U1–U11 mission log, and the mined-out methodology
history now live in the dated session archives:

- [`archive/SYNC_SESSIONS_2026-08.md`](archive/SYNC_SESSIONS_2026-08.md) —
  the 2026-07-09 … 07-27 window (lifted 2026-08-14): the `unexpected:fi`/chardef
  `\meaning` fix, spconf keywords, the xparse `\c`-cedilla clobber, kernel
  autoload-on-undefined, the `.bib`-as-DATA family close, silence.sty/bare-`&`
  bib fields, the 07-26 resilience-mining + email-reported missing-References
  clusters, siunitx v3, the 0.7.4-rc4 crates.io tag, and the `\AtBeginDocument`
  #2846 re-port.
- [`archive/SYNC_SESSIONS_2026-07.md`](archive/SYNC_SESSIONS_2026-07.md) —
  the 2026-07-02 … 07-08 window: upstream PR #2829 "Framing", the MathML-post
  exhaustive line audit (waves 1+2), live-run fatal/error mining rounds,
  author/affiliation frontmatter split, width-based figure-panel arrangement,
  and the `\AtBeginDocument`/`\RequirePackage` #2846-port regression fix.
- [`archive/SYNC_SESSIONS_2026-06.md`](archive/SYNC_SESSIONS_2026-06.md) —
  the 2026-06-22 … 07-01 window plus the slowest-100 batch triage and the
  2026-06 cortex-cross-join methodology history.

(Upstream-sync catalog also at
[`archive/UPSTREAM_SYNC_2767_to_2833_2026-06-26.md`](archive/UPSTREAM_SYNC_2767_to_2833_2026-06-26.md).)

## Standing policies & method — read before changing behaviour

### Methodology & the cortex cross-join

Working method (2026-06): **re-triage LARGE-error papers** (the single-error tail
is exhausted) → bisect the doc to the trigger line → verify Perl with `--verbose`
→ fix the divergence. Random sweeps are low-yield.

**Clustering trap — a bibliography sub-conversion's fatal has no fatal message.**
It is deliberately downgraded to a trailing `Error:bibliography:convert` on the
parent (`bib_session.rs` uses the post-phase reporters; Perl's
`convertBibliography` returns empty-handed the same way,
`MakeBibliography.pm` L240-242). So a sweep that clusters on **fatal messages**
files those documents as "no fatal recorded" — measured **~80** in one run.
Cluster on `Status:conversion:3` or the last `Error:` line. Mechanism and what
DOES cross the boundary (`MergeStatus`): [`WISDOM.md`](parity/WISDOM.md) **#72**.

**Cortex agentic API (reads open, no token):** `http://127.0.0.1:8000/api`.
Recipe: `GET /api/reports/<corpus>/oxidized-tex-to-html/<severity>` → categories;
`…/<severity>/<category>` → per-`what`; `…/<category>/<what>` → paper list. Then
`GET /api/corpus/<corpus>/tex_to_html/document/<id>` for Perl status — a Rust-only
win is **Perl=no_problem/warning but Rust=error/fatal**. Corpus
`sandbox-arxiv-10k-shuffle`. URL-encode `\`→`%5C`, `^`→`%5E`.

### CLI options — the option-C policy (issue #191 CLOSED 2026-07-09) + `validate()`

Issue #191 "support the original latexmlc/latexmlpost options" is **closed**;
what survives here is the standing **option-C policy** it established, plus the
one feature deliberately left undone (`validate()`, below). The policy: wire only
options whose engine feature genuinely works end-to-end; keep the clap parser
**strict** (no accept-and-warn stubs); deferred/missing features stay hard parse
errors. Consult it before adding any CLI flag.

#### Deferred — feature genuinely NOT supported (do NOT stub)
- `--parse=STRATEGY` — grammar selection unsupported (one Marpa grammar);
  `--nomathparse` / `--mathparse` is the real interface. (Attempted + removed.)
- `--svg` / `--nosvg` — **deferred (verified 2026-07-09):** the HTML5 XSLT
  already renders `<ltx:picture>` as inline `<svg>` by default, so the standalone
  `svg.rs` post-processor (`impl Processor for SVG`, unwired) is redundant and
  produces divergent, unverified output (25 vs 27 `<svg>` on `tests/graphics/
  picture.tex`). Wiring it was built + reverted.
- `--pictureimages` / `--nopictureimages` — `picture_images.rs` delegates to the
  **unwired LaTeXImages latex+dvipng pipeline** (`latex_images.rs`); same
  category/effort as `--mathimages`.
- `--openmath|om` — no OpenMath serializer. (User: defer.)
- daemon net (`--port` / `--address` / `--expire` / `--autoflush` / `--cache_key`)
  — socket-daemon model; we ship `--server` (stdio LSP). (User: defer.)
- `--mode` (= alias for `--profile`); `--profile=NAME` — needs a preset registry.
- `--mathimages` / `--mathsvg` / `--mathimagemagnification` — needs a
  latex+dvipng math-render pipeline.
- `--unicodemath` / `--plane1` / `--hackplane1` / `--linelength` — plain/unicode
  math output modes.
- crossref cluster (`--crossref` / `--scan` / `--noscan` / `--urlstyle` /
  `--prescan` / `--dbfile` / `--bibliography` / `--splitbibliography`) + index
  cluster (`--index` / `--permutedindex` / `--splitindex`) — multi-doc site-DB
  features. (Scan IS wired as post Phase 2, so `--noscan` is a real-but-risky
  off-switch; parked with the cluster.)
- `--tex` / `--box` — intermediate box/tex serializers absent.
- `--omitdoctype` — DTD-only in Perl; Rust has no DTD (moot).

#### `validate()` / `--validate` — POSTPONED to the NEXT release (decided 2026-07-09)
Today `Post::Document::validate()` (`latexml_post/src/document.rs:1717`) is a
STUB: it logs "Would validate against RelaxNG schema" and returns `Ok(())`.
Real RelaxNG validation is wanted, but is **deferred to the next release** because
it is gated on a `rust-libxml` crates.io publish (see below). Reference: Perl
`LaTeXML/lib/LaTeXML/Common/XML/RelaxNG.pm` + `LaTeXML/lib/LaTeXML/Post.pm`.

**Architecture decision (owner, 2026-07-09): `rust-libxml` provides the public,
safe Rust RelaxNG interface; `latexml-oxide` is a pure consumer.** All libxml2
`unsafe`/FFI stays in the fork — the alternative (raw `xmlRelaxNG*` FFI inline in
`latexml_post`, which would compile against the shipped crates.io `libxml 0.3.15`
with no publish) was **rejected**. So this feature cannot fully land until the
fork's RelaxNG module is published as `libxml 0.3.16`.

Constraint: the schema is **modular** (`LaTeXML.rng` `<include>`s
`LaTeXML-common.rng`, `-structure`, `-math`, …) and the binary is
**self-contained** — no on-disk schema. Includes MUST resolve through the
embedded table (`latexml_core::common::relaxng::embedded::lookup`), served via
the fork's existing `libxml::io::register_input_callback` (built for exactly this
— "bundles RNG schemas via include_bytes! … RelaxNG `<include>` via
`xmlRelaxNGParse`"), NOT disk.

Steps (next-release session):
1. **rust-libxml fork — add a safe `relaxng` module.** The fork's `schemas`
   module is **XSD-only** (`xmlSchema*`). Mirror it: `relaxng/{parser,schema,
   validation}.rs` wrapping `xmlRelaxNGNewParserCtxt`(URL — so relative includes
   resolve through the callback) / `xmlRelaxNGNewMemParserCtxt` + `xmlRelaxNGParse`
   (→ `RelaxNGSchema`) and `xmlRelaxNGNewValidCtxt` + `xmlRelaxNGValidateDoc`
   (→ `RelaxNGValidationContext`), with `xmlRelaxNGSetValidStructuredErrors`
   capture. Fork unit test (valid + invalid doc). **Publish `libxml 0.3.16`.**
2. **Embedded-include resolution** via `libxml::io::register_input_callback`
   (`embed:///RelaxNG/LaTeXML-*.rng` → `embedded::lookup`); verify with the
   renamed-`resources/` smoke that no schema is read from disk.
3. **Consume in workspace** — bump the `libxml` dep `0.3.15` → `0.3.16`; `cargo test`.
4. **Flesh out `validate()`** — parse+cache the schema once; run `validate_doc`;
   map each captured `StructuredError` to a `Warn!` / `post_error` in the project
   logging convention (Perl reports schema violations).
5. **Wire `--validate` / `--novalidate`** — CLI flags + `PostOptions.validate`;
   call `validate()` in `run_post_processing_impl` when enabled. DEFAULT
   decision: Perl defaults ON; propose **opt-in** in Rust (validation cost +
   corpus warning noise) as a documented divergence — confirm with owner before
   flipping the default on.
6. **Tests** — a valid fixture validates clean; an intentionally schema-invalid
   doc reports the expected violation; `--novalidate` skips.

### Archived-audit residuals (2026-07-09 docs compaction) — still-open leftovers

Two completed diagnostic snapshots were dated + archived; their still-open
residuals stay here so the live worklist keeps them visible:

- **MathML-post line audit** (sweep complete; →
  `archive/MATHML_POST_LINE_AUDIT_2026-07-05.md`). **This list was stale until
  2026-07-29** — it named F11/F14/F15/F16 as open when the archive marks all four
  ✅ and the code confirms it (`filter_row` in `mathml/mod.rs`, `do_cfrac` in
  `presentation.rs`, the `0x2A50`→Cat C / `0x27A1` / `0x0331` rows plus their
  guard in `operator_dictionary.rs`). **F17 is now also CLOSED (2026-07-29)** — see
  R3 for the per-item disposition. What is genuinely open is **F5** Linebreaker
  (full feature gap — the sketch used the wrong strategy; needs a port-or-drop
  scope decision), the **F14 residual** (`m:share` hrefs use the primary ID
  suffix; the `MATHPROCESSOR->IDSuffix` secondary-suffix wiring is unconnected),
  and PARTIAL inherited-context bindings on `pmml_top`/`pmml_parenthesize`.
  (Content-MathML items obey the defer-to-a-dedicated-session directive above.)
- **arXiv velocity-fork audit** (items 1–4 landed 2026-07-03; →
  `archive/ARXIV_FORK_AUDIT_2026-07-03.md`). Sole residual: **item G** —
  `readBalanced` drops comment tokens (fork `4e1578d1`); Rust `read_balanced`
  still keeps comments in its result via `CommentSink::Into` (gullet.rs ~L1363,
  `CommentSink` at ~L565). Low urgency (`INCLUDE_COMMENTS=false` default); port
  at the next gullet-seam session.

### `fragid` parity audit — id preservation (2026-08-04)

Full Perl↔Rust comparison of the `fragid` mechanism, since `fragid` — not
`xml:id` — is what the HTML5 XSLT's `add_id` emits the HTML `id` from, so a
node with an `xml:id` but no `fragid` reaches HTML with **no id at all**.
`CrossRef::fill_in_frags` (CrossRef.pm L312-324) is the assigner and is
faithfully ported; it stamps only nodes carrying an ObjectDB `ID:` entry.

**Measured end state** (same-host Perl 0.8.8, id-rich document with sections,
labels, footnotes, equation/align, table, figure, refs): HTML id sets are
`24 rust / 23 perl`, **zero ids present in Perl and missing here**, zero
dangling in-page anchors in either engine; the one extra is our `abstract1`
(we make the abstract linkable, Perl does not). Under `--splitat=section` the
per-page sets match exactly (`S1.html` 22/22) with no broken cross-page
anchors. Witness dirs under the session scratchpad.

**Two audit claims did NOT survive checking** — do not re-chase:
- *"Scan skips XM* descendants ⇒ hrefless `<m:share/>` in all Content MathML"*:
  **refuted for documents.** A `$a<b<c$` document emits
  `<share href="#p1.m1.sh1">` in BOTH engines, byte-identical. The hrefless
  `<share/>` appears only in the standalone `latexmlmath_oxide --cmml` path —
  where Perl's `latexmlmath` emits `href="#Ex1.m1.sh1"` pointing at **nothing**
  (its own output has zero ids). Shared limitation, Perl's arguably worse (a
  dangling href vs a missing one). Not a Rust-only gap.
- *"`generate_node_id`'s fragid half is missing"*: **present and faithful**
  (document.rs:1361-1366 vs Post.pm L1490-1492). It was merely never reached,
  because the parent walk above it used a bare `xml:id` read that always
  missed; the accessor sweep activated it.

**Fixed here:** id-bearing markup CLONED into a generated bibliography (an
`ltx:Math` in a `.bib` title) reached HTML with no id, because Perl's
`Collector::rescan` (Collector.pm L97, called from MakeBibliography.pm L71/78)
re-runs the whole Scan over the generated subtree and we have no rescan.
MakeBibliography now registers every id-bearing node of that subtree
(id/fragid half only, never overwriting an existing entry). Witness went
`4 rust / 5 perl` ids → `5 / 5`. Guards:
`06_cluster_bibliography::{bib_entry_ids_are_bib_rooted_like_perl,
bib_entry_cloned_markup_keeps_its_id}` (both red-checked).

**Still open, ranked** (each verified as real code, blast radius NOT
re-measured except where noted):
1. **No `Collector::rescan`.** The fix above is a bibliography-local stand-in.
   The general wiring — re-run `Scan` from `MakeIndex`/`MakeBibliography` —
   also restores `labels`/relations/per-type values a full Scan derives, and
   would let the three ad-hoc registrations be deleted. Blocked on `Scan`
   owning its `ObjectDB` by value (`scan.rs:49`), so it needs a borrow-based
   or take/restore refactor.
2. **`associateNode` (Post.pm L508-585) unported** — generated MathML/OpenMath
   nodes carry no `xml:id`, so `convertedIDs` and pmml↔cmml parallel
   cross-linking do not exist. This is also the only real caller of
   `generate_node_id`, which currently has **zero callers**.
3. **`in_page_id` lacks the `labelids` branch** (Scan.pm L176-184) — affects
   `--splitnaming=label*` only; `Scan::new` takes no options to carry it.
4. **`in_page_id` lacks the `split_from_id` fallback** (Scan.pm L191-192);
   `PostDocument::split_from_id` is read by nobody (the `in_page_id` fallback is
   missing). It IS set on eager split subdocs (`document.rs:709`) but never on
   streaming pages. Anchor-naming only — links stay self-consistent.
5. **`strip_ref_display_fragids`** (crossref.rs:131) is Rust-only and matches
   `//ltx:ref//*[@fragid]` wholesale, so genuine id'd content inside an
   `ltx:ref` loses its fragid too. Narrow it to ids absent from the ObjectDB.
6. **`make_sub_collection_documents` returns `vec![]`** (collector.rs:141) —
   `--splitindex`/`--splitbibliography` drop every entry past the first
   initial. Note its only callers are its own unit tests, one of which is
   named `…_currently_returns_empty`, so the accessor fix landed there is
   inert until this is implemented.
7. **Glossary term/description flattened to text** (make_index.rs:553-596)
   where Perl deep-clones the phrase markup with suffix `glo`, keeping ids.

### `--format=xml` emits no `ltx:bibitem` — untriaged (2026-08-04)

`--format=xml` output carries no `ltx:bibitem` elements at all, while
`--format=html5` on the same source renders the bibliography — so the two
format paths do not run the same post chain. Not yet diagnosed; check before
trusting an `xml`-format dump as a bibliography oracle. (The `ltx:biblist`
`fragid` gap noticed alongside it is FIXED — see the 2026-07-28 entry above.)

### Streaming diverged on a fancyvrb `fontsize=` + `numbers=` Verbatim — ✅ FIXED 2026-08-04

Under a small streaming budget the XML was **not** byte-identical to the eager
path when a fancyvrb verbatim carried `fontsize=` **and** `numbers=` together
(either alone was clean). Root cause was **not** package-load-related, despite
being found while guarding #500: `tex_glue::dimension_to_spaces` renders a width
as Unicode space glyphs and does the arithmetic in **ems**, but read the live
`lookup_font()` from callers that are `DefConstructor`s — i.e. CONSTRUCTION
time. The eager path builds only after the whole document is digested, so that
is the font the document *ends* in; appending `\small` before `\end{document}`
changes the glyph chosen for a skip pages earlier (Perl 0.8.8 too — it has the
same read, `TeX_Glue.pool.ltxml` L44). Depending on WHEN the build ran is what
broke byte-identity: streaming builds mid-document, read the local font, and
emitted U+2004 where eager emitted Perl's U+2009.

Fixed by passing the whatsit's own digest-time font at the constructor/whatsit
sites (`\hskip`, `\kern`, `\lx@text@intercol`, `digested_to_text`); digest-time
callers keep the ambient read, which is already the digest font. Surpass-Perl
divergence [OXIDIZED_DESIGN #96](parity/OXIDIZED_DESIGN_DIVERGENCES.md),
Perl-side record [KNOWN_PERL_ERRORS #74](parity/KNOWN_PERL_ERRORS.md). Zero
golden churn. Guards:
`06_cluster_regressions::faked_space_is_sized_by_the_font_it_was_digested_in`
(value) and `114_streaming_cluster_regressions::streaming_matches_eager_on_cluster_regressions`
(eager == streaming; it caught the defect).

**Reproduction note worth keeping:** the CLI cannot reach the sweep's aggressive
3-box budget — `--streaming` sizes the budget from `--max-memory`, and a ceiling
small enough trips the RSS fuse first. Drive `streaming_sweep::convert_with(src,
Some(3), …)`, or run the built sweep binary directly: it globs its fixture dir at
RUNTIME, so fixtures can be added or removed without a rebuild.

### A `robust` DefConstructor reverted under its munged cs — ✅ FIXED 2026-07-29

Rust wrote `tex="x+\text{see \ref {sec:one}}"` where Perl writes
`\ref{sec:one}` — a space between the control word and its `{`. Semantically
equivalent TeX, but `tex=` flows into the MathML `alttext`, the screen-reader /
no-MathML fallback, and into golden comparisons.

**Root cause, and it is not a serializer bug.** `robust => true` installs the
real definition under the MUNGED cs `\ref` + a literal trailing SPACE — LaTeX2e's
`\DeclareRobustCommand` idiom, where `\ref` expands to `\protect\ref␣` and `\ref␣`
holds the body (`def_robust_cs`, Perl `Package.pm:1143-1149 defRobustCS`). The
name really does contain a space, and `Whatsit::revert` printed it. Perl avoids
this in `DefConstructorI` (L1480-1481):

```perl
alias => (defined $options{alias} ? coerceCS($options{alias})
          : ($options{robust} ? $cs : undef)),
```

i.e. the pre-munge cs becomes the alias, and reversion prefers the alias.
`dialect.rs::def_constructor` never set it — note the **commented-out
`csname_alias` block at `dialect.rs:729`** was an earlier attempt at exactly this,
left disabled in the DefMath path.

Scope is exactly `\ref` (plus `\pageref`, `Let!` to it): it is the tree's ONLY
`robust` `DefConstructor!`. The `robust` DefMath entries (`tex_math.rs:1270/1273`,
`\overbrace`/`\underbrace`) pass an explicit `alias`, which the fix's
`if options.alias.is_none()` guard respects, and everything else `robust` is
`DefMacro!`/`DefPrimitive!` — Perl deliberately does NOT apply this fallback to
`DefPrimitiveI` (L1318), so neither do we.

**The fix changes the reversion only.** The definition is still installed under
the munged cs, so `get_cs_name()` still reports `\ref ` — code that identifies a
whatsit by cs must keep accepting both spellings. `lxrdfa_sty.rs`'s
`cs == "\\ref" || cs == "\\ref "` (L15, L118) is therefore still correct and was
left alone; it is the fingerprint of someone hitting this before and papering
over it. `get_cs_or_alias()` is the clean accessor.

Guard `06_cluster_regressions::cluster_robust_cs_reverts_unmunged` over
`tests/cluster_regressions/robust_cs_reversion.tex`, ground-truthed against
same-host Perl 0.8.8: zero errors in both engines and all three `tex=` attributes
byte-identical. Found by end-to-end-diffing Perl `latexml`+`latexmlpost` against
the Rust pipeline while verifying R3/F17 — the post-stage golden could not have
caught it, because that test feeds Perl's core XML into the Rust post stage.
Note the *inverse* bug — a Tokens round-trip **eating** a control word's
terminating space — is a separate, already-fixed issue
(`\bib@field@unknownasdata`, `SYNC_SESSIONS_2026-07.md`); do not conflate.

## Open items — detail for the ranked rows

### R1 — upstream `brucemiller/LaTeXML#2852`: a subfile's `\documentclass` options are not packages

**OPEN upstream**; **our half is already merged as PR #310**, so nothing is
pending here in this repo. **CI re-verified 2026-07-29: all 15 checks SUCCESS**
(TeX Live none/2021/2022/2023/2024/2025 × Perl 5.34-5.42, Linux + Windows),
`mergeable`, not a draft, **zero reviews**, untouched since 2026-07-20. So the
code side of this row is done and green; what is left is a review nudge to the
maintainer, which is a human-voice action, not a task to automate. The
allowlist was hand-split on
`,` and missed every valued form (`[varwidth=5cm]` → `Error:undefined:{varwidth}`,
pdflatex clean); it now reads `OptionalKeyVals` and matches on the key. The same
fix is ported to Perl (`OptionalKeyVals` + `getPairs`) with a `t/structure` case
that actually guards it, pushed to `dginev/LaTeXML`. **Action: check its CI, then
ask for review** — no code work expected. *(This entry read "PR #310 … Ready to
merge" until 2026-07-25, long after it merged.)*


### R2 — `--preload=<cls>` alone trips the hook stack — OPEN (re-verified 2026-07-29)

*(The "class-name divergence" this heading used to also name was the second
divergence below; it is fixed. Only the hook stack is still open.)*

**Symptom.** `--preload=<any>.cls` prints `LaTeX hooks Error: Extra \PopDefaultHookLabel`
(article/book/report; `.sty` clean; `\documentclass` clean; `LATEXML_NODUMP=1` clean).
Perl is silent for the same preload.

**Mechanism (traced, 2026-07-17).** Push/pop are perfectly balanced and nested — the trace
is `PUSH article → (LaTeX.pool loads) → PUSH textcomp → POP textcomp → POP article → error`.
The bug is that **`\@pushfilename` changes MEANING mid-load**: `article` is pushed *before*
`LaTeX.pool` (and the kernel dump behind it) loads, so it uses a pre-pool `\@pushfilename`
that never touches `\g__hook_name_stack_seq`; the pool then installs the real expl3
`\@popfilename`, so `article`'s pop hits a seq holding only the *inner* packages' pushes,
finds it empty, and errors. `\documentclass` escapes because the pool is already loaded, so
both sites use the same meaning.

**A definedness check cannot see this** — the CS is defined at both sites; only its meaning
changes. Perl's `$pushpop` (Package.pm L2595, computed once and reused at L2637) is a
definedness check too, so Perl has the same hole; it is silent only because its dump omits
`\g__hook_name_stack_seq`, and `\seq_gpop:NNTF` on an *undefined* seq does not complain.
Ours dumps it as `\c_empty_seq`, so the real expl3 code correctly notices.

**Mitigated, not fixed (2026-07-17):** `util::preset::new_test_engine` now preloads
`LaTeX.pool` first (the order ar5iv's list already used), so `latexmlmath_oxide` stops
provoking it. `--preload=article.cls` on its own STILL errors.

**Dead ends — measured, do not retry:**
* Filtering the L3-hook stubs + filename stack from the dump (write+read): symptom gone,
  preloads clean — but `cluster_mhchem_cf_author_macro` 0 → **1003 errors** (suite 1581/0 →
  1572/9). The dump REPLACES base (DUMP_DESIGN rule 1), so filtering leaves a HOLE, not a
  fallback to `latex_base.rs`.
* Filtering ONLY `\g__hook_name_stack_seq` to match Perl's dump exactly: symptom gone,
  mhchem still fails — that record is load-bearing for our expl3 emulation.
* Threading Perl's `$pushpop` from push to pop instead of re-deciding (more Perl-faithful,
  worth doing anyway): does NOT fix it — the flag is `true` at both sites; the *meaning*
  moved underneath.
* Filtering `\PopDefaultHookLabel` alone: inert. The erroring caller is the internal
  `\__hook_curr_name_pop:`.

**Where the failing pair is NOT (2026-07-25, measured — corrects an earlier note
in this entry).** The Rust binding has exactly two push/pop sites:
`binding/content.rs:1000-1015` (push) and `:826-831` (pop). Probing both on the
failing `--preload=article.cls` run shows **only `textcomp`'s push reaches them —
`article`'s push and BOTH pops never do.** So the erroring pair is not a Rust-side
`digest`; it runs inside TeX (expl3's `\__hook_curr_name_pop:`, as noted above).

**Consequence: a Rust-side "thread the push's answer to the pop" fix cannot work**
— there is no Rust-side pop for the failing frame to pair with. An earlier version
of this entry proposed exactly that (inspect `\@pushfilename`'s body for
`\@expl@push@filename@aux@@` at push, carry it to the pop); it is a **fifth dead
end**, disproved before implementation. Any real fix has to act on the TeX side —
which points back at (c), ordering the pool load before the class's own
`\@onefilewithoptions` push, rather than at the Rust seams.

**Candidate fixes.** (a) Ensure a class/package preload cannot be the thing that drags in
the pool — auto-prepend `LaTeX.pool` when any `.sty`/`.cls` is preloaded. Rejected for the
release: Perl prepends only `TeX.pool` (LaTeXML.pm L710) and never auto-loads `LaTeX.pool`,
so this is a Rust-only divergence, and it would drag the LaTeX kernel into a `.sty` preload
on a plain-TeX document (the LaTeX-2.09 class `graphicx_sty.rs` already guards against). If
adopted, make it conditional on the pool being unloaded and LOG it. (b) Pair the pop to the
push's actual *meaning* rather than to definedness. (c) Make the pool load before any
handleoptions push. (b)/(c) address the cause.

**(c) collapses into (a) — checked 2026-07-29.** The pool load is not *ours* to reorder:
`LoadPool('LaTeX')` is the **class binding's own first statement**, in Perl
(`article.cls.ltxml` L5) exactly as in Rust (`article_cls.rs:4`), and it runs *inside*
`InputDefinitions`, i.e. after the `\@pushfilename`. Perl's ordering is therefore
identical to ours — Perl is silent only for the dump reason above. So "load the pool
earlier" means hoisting it out of the binding, which is the same Rust-only divergence
that got (a) rejected. That leaves (b), or a TeX-side repair of the stack at the moment
`LoadPool` swaps `\@pushfilename`'s meaning underneath an already-open frame.

**Second divergence, same area — ✅ FIXED 2026-07-29.** It was wider than recorded here:
the preload-PI loop (`core_interface.rs`, the `for preload in preloads` block) is a
translation of Perl `Core.pm` L268-277, whose three `s///` rewrite `$preload` **in place**.
`Regex::replace_all` *returns* the rewritten string instead, and all three results were
discarded, so **nothing was ever stripped and no `options` attribute was ever emitted**:

| `--preload=` | was | Perl (and now) |
|---|---|---|
| `article.cls` | `class="article.cls"` | `class="article"` |
| `[twocolumn,11pt]article.cls` | `class="[twocolumn,11pt]article.cls"` | `class="article" options="twocolumn,11pt"` |
| `[dvipsnames]color.sty` | `package="[dvipsnames]color.sty"` | `package="color" options="dvipsnames"` |

**Not cosmetic:** `latexml_post`'s `find_documentclass_and_packages` parses these PIs and
`latex_images.rs:pre_preamble` emits the result as a literal `\documentclass[…]{…}` /
`\usepackage[…]{…}`, so `--mathimages`/`--graphicimages` on any preload-driven conversion
was writing `\documentclass{article.cls}` and losing every preload option.

Fixed by stripping the bracket and the suffix off a `&str` cursor, Perl-faithfully:
only the two literal `.cls`/`.sty` suffixes (so `mystyle.tex` keeps its extension, as in
Perl), and an empty bracket contributes no attribute (Perl's `($options ? … : ())` is
falsy on `""`). Deliberately **not** routed through `parse_preload_spec`, which splits on
the last `.` and would eat any extension. Guard: `109_preload_pi_attributes.rs`, a
six-shape table whose every expectation was ground-truthed byte-for-byte against Perl
LaTeXML 0.8.8 on the same input.

### R4 — biblatex `.bbl` TokenLimit loop, 2605.17646 — ✅ FIXED 2026-07-25

Root cause was **not** `\missing{Cowen2021}` (the `.bbl`'s last line, and the
entry's standing suspicion): deleting it leaves the Fatal untouched. It is a
self-referential `\let` in the engine's pseudo-bibitem machinery —
`setupPseudoBibitem` re-arming captures `\save@bibitem` ← `\restoring@bibitem`,
whose body ends in `\bibitem`, so it expands forever. The re-arm happens because
biblatex's apa style asks biber for **two sorting schemes**, so the `.bbl`
carries two `\datalist` blocks (2 × 29 entries here) and each `\enddatalist`
expands to a whole *bare-CS* `\thebibliography…\endthebibliography` — no group,
so the first arming was still live when the second opened.

Fixed in two symmetric halves: `setup_pseudo_bibitem` captures the originals
once per arming (`\ifx\bibitem\restoring@bibitem` guard), and
`\endthebibliography` now disarms — upstream has no teardown, relying on
`\begin`/`\end` popping the group, which the bare-CS pair never opens. The
missing teardown was separately costing a stray empty bibitem outside the
biblist (`Error:malformed:ltx:bibitem`) from the blank line after
`\printbibliography`.

Witness now converts in ~1 s with **1 error** (`\missing`, undefined in both
engines) and 58 bibitems / 2 bibliographies / 2 biblists — byte-for-byte the
structure same-host Perl produces, which takes 33.7 s and reports **59 errors**.
**The defect is shared with Perl** (`\thebibliography \endthebibliography
\thebibliography \bibitem{b}` hangs Perl 0.8.8 >400 s); it stays latent upstream
only because Perl's biblatex binding never defines `\printbibliography`, so Perl
never reads a real `.bbl` this way. Mechanism, minimal trigger and the
upstream-candidate note: `KNOWN_PERL_ERRORS.md` #57. Guard
`06_cluster_bibliography::cluster_biblatex_two_datalists`.

**Follow-up the same day — the witness is now error-free.** Two more gaps it
surfaced, both landed:
* **`\missing{key}` was undefined** (`Error:undefined:\missing`). It is biber's
  marker for a cite-key absent from every `.bib` (TL `biblatex.sty` L8503
  `\blx@bbl@missing`): upstream records the key and emits a **warning**,
  typesetting nothing. Ported faithfully to `biblatex_sty.rs` — a no-op that
  names the key (`Warning:missing_entry:biblatex`), which is the author's bug,
  not ours (issue #92). Perl's binding leaves it commented out
  (ar5iv-bindings L613), so every biber `.bbl` carrying one errors there.
* **A leading relop + comma had NO parse.** `list_apply`'s fragment guard
  rejected any item with an `absent` relop operand while
  `formula relop formula_list` is deliberately gone (`KNOWN_PERL_ERRORS` #37),
  so `$>50,000$` was `ltx_math_unparsed` though `$>x$`, `$a,b$` and
  `$a>50,000$` all parsed. The guard now rejects a **comma** pair only when
  BOTH items are fragments (mirroring the relaxation `formulae_apply` already
  carried) and stays strict for `\quad`, where a fragment run is one broken-up
  equation — the `\quad` half is load-bearing: relaxing it too made
  `tests/math/sampler`'s `\displaystyle=f(x)+\phantom{g(x)}+h(x)` parse
  *wrongly* rather than not at all. Guard
  `06_cluster_math::cluster_leading_relop_comma_list`.

**Then the residual math gaps too (user-directed, same day) — the witness is now
0 errors AND 0 unparsed formulas.** Two grammar additions, both measured against
same-host Perl:
* **A bare operator used as an OPERAND** — `f(\cdot)`, `\langle\cdot,\cdot\rangle`,
  and operators NAMED rather than applied (`(+)`, `(=)`, `(\times)`). The grammar
  admitted fenced singleton bigops/OPERATORs but not the ADDOP/MULOP/BINOP/RELOP
  roles, so **Perl parsed 7 of 8 such shapes and we parsed 0**. New
  `placeholder` / `placeholder_list` (`grammar/builder.rs`) admit them only where
  FENCED — the same containment the bigop lines use — so a stray `a + \times b`
  still fails. `$\|\cdot\|$` stays unparsed as parity (Perl fails it too).
* **A comma list mixing ONE relation with a plain term.** `formula_list` carried
  only the all-`modified_term` variants, the mixed ones deferred *"until a
  witness shows them needed"* — so `f(a\geq 0, b\leq 1)` parsed while
  `f(a\geq 0, b)` did not. arXiv 2605.17646's
  `m_S(t \mid T_i \geq t_{\text{crit}}, \mathbf{Z})` is that witness; Phase 2
  adds both orders.

Not just this paper: on the #37 ambiguity stress witness **1510.03361** the two
additions took `ltx_math_unparsed` **170 → 136** *and* wall time **16.8 s →
11.7 s** — formulas that used to exhaust the parser now succeed early. The
`parse_tree_count_limits` canary stays green.

Witness end state: **0 errors, 1 warning** (the actionable missing-entry), 1.0 s,
**312 formulas / 0 unparsed** (Perl: 0 unparsed, 59 errors, 33.7 s), and
structurally identical to Perl — same counts for all 25 element classes sampled
(312 `Math`, 58 `bibitem`, 336 `td`, 78 `ref`, …; sole delta 87 vs 88 `para`).
Guards `cluster_fenced_bare_operator`, `cluster_leading_relop_comma_list`.

**Thousands separator — ✅ FIXED 2026-07-25 (US default; EU already worked).**
`50,000` is ONE number; both engines read the comma as a list separator. Owner
policy: **default US, EU a supported secondary.** The `en` half was the broken
one — Perl's thousands arm demands `$r ne 'PUNCT'` and a math comma is always
PUNCT, so it is dead code for English, while the EU decimal comma already works
through the language maps. **The ligature is the wrong seam and that is a
measured dead end** (built, reverted): ligatures run per-token during building,
so there is NO right context, and a merge-at-three-digits rule corrupts plausible
pairs — `$(1, 2024)$` → `12024`. Landed instead as a `DefRewrite` in the
post-build `Rewriting` phase, where the ligature has already collapsed each digit
run into one token, so the group length is testable with its right context and
those cases are safe by construction. Guards
`cluster_thousands_separator_us_default` / `_eu`; mechanism, the two
implementation traps and the full result table in
[`CONTENT_MATHML_GAPS.md`](math/CONTENT_MATHML_GAPS.md).


### R9-BST — `.bst` support: raw interpretation vs `_bst.rs` bindings — FUTURE, not started

**How this row came to be, and a correction (2026-07-27).** It was opened on the
claim that `\Dbar` in witness **2605.11579** proved `.bst` files *vendor macro
definitions*. **That claim was wrong and is retracted.** Checked afterwards:

* the witness uses `\bibliographystyle{alpha}`, and `alpha.bst` contains **zero**
  occurrences of `Dbar`;
* `\Dbar` is defined by **`mathscinet.sty` v1.05** (ships with **amsrefs**;
  `\ProvideTextCommand{\Dbar}{T1}{\DJ}`), which also defines `\cprime`,
  `\cdprime` and `\bud`;
* **the witness does not load `mathscinet.sty`**, so real pdflatex on that source
  raises the same undefined control sequence. `undefined:\Dbar` there is
  **PARITY**, not a defect — see R9-MSC below, which is now DONE. (The witness
  stopped *emitting* that error on 2026-07-27 for an unrelated reason: divergence
  **#80** digests only cited entries, and its `\Dbar` entry is uncited. The
  parity reasoning is unchanged; the witness is just no longer a demonstration
  of it.)

**What survives the correction.** The row still stands, on its original and
independently-true footing: we read `.bib` directly, never execute the `.bst`,
and therefore cannot reproduce the label style, sort order or **field selection**
a document's own `.bst` would produce. Field selection is the part currently
approximated by hand — OXIDIZED_DESIGN **#73** neutralizes `abstract`/`keywords`/
`contents` precisely because no standard `.bst` declares them in its `ENTRY`
list, which is a `.bst` fact we hard-coded rather than computed. A `.bst` we can
execute is what would replace that guess with an answer.

**One half of that shrank 2026-07-27.** *Entry* selection no longer needs a
`.bst` at all: divergence **#80** takes the cited set from the document's own
`BIBLABEL` records — the same information `bibtex(1)` reads out of the `.aux` —
so which entries reach the `.bbl` is now computed, not guessed. What remains
`.bst`-shaped is *field* selection (#73's hard-coded three), label style and sort
order.

**The lesson worth keeping** is about method, not `.bst`: the vendoring claim was
recalled from memory and written into a merged doc without checking the witness's
own `alpha.bst`. Two minutes of `grep` refuted it. Verify the file, not the
recollection.

**Two candidate resolutions.** They are not exclusive; (b) is a strict subset of
what (a) delivers.

**(a) Port upstream `brucemiller/LaTeXML#1955`, "[WiP] BibTeX emulation".** A
real BibTeX virtual machine: `LaTeXML/BibTeX/BibStyle/` parses the `.bst`
(`StyCommand`, `StyString`, `Precompiled`), `LaTeXML/BibTeX/Runtime/` executes it
(`Builtins`, `Names`, `Strings`, `Buffer`, `Entry`). **34 files, +4224/-801**,
open since **2022-10-05**, itself a refactor of tkw1536's closed #1231. Buys the
whole thing at once — correct labels, correct sort order, correct field
selection (which is what OXIDIZED_DESIGN #73/#74 currently approximate by hand),
*and* the vendored preamble. Costs: it is upstream **WiP and unmerged**, so
porting it means tracking a moving, unreviewed target; its author's own summary
says "I've undoubtedly broken a few features that had been working".

**(b) Per-style `_bst.rs` bindings**, mirroring the `_cls.rs` / `_sty.rs`
convention already in the tree. A binding per common style (`plain`, `unsrt`,
`alpha`, `abbrv`, `amsplain`, `amsalpha`, `splncs04`, `IEEEtran`) supplying its
preamble definitions and its label/sort rules. Cheaper per increment, matches how
the rest of the port already handles vendor files, and can land style-by-style
against measured witnesses. Costs: the long tail of `.bst` files is unbounded,
and hand-written rules will drift from what the real `.bst` computes.

**Prerequisite either way — the `.bbl` question.** arXiv's convention means most
papers ship a `.bbl`, which we already prefer; `.bst` support only matters for
sources with `.bib` + `.bst` and **no** `.bbl`. Before committing to (a) or (b),
**measure how large that population actually is** in a full-corpus run. The
existing deferral names witness **2605.16562** (LNCS, `splncs04.bst`); one
witness is not a business case. That measurement is cheap and is the honest first
step.

**Do NOT start** either resolution without that measurement and an explicit
decision — this row exists to stop the next `\Dbar` from being patched in
isolation without the context above.

### R9-MSC — `mathscinet.sty` binding — ✅ DONE 2026-07-27 (PRs #415 + #419)

Closed. `latexml_package/src/package/mathscinet_sty.rs` binds the AMS v1.05
package (amsrefs bundle), and the `\cprime`/`\Cprime`/`\cdprime`/`\Cdprime`
family that used to sit always-on in `latex_constructs.rs` — the LaTeX **kernel**
file, which must track `latex_constructs.pool.ltxml` — was moved out and then
**deleted**: it is package vocabulary, a paper gets it by loading the package (or
`amsrefs`, `amsrefs.sty` L217) or via its own `.bib` `@preamble`. The three
witnesses that opened this row all load the package by name (2508.13753 L7,
2508.20226 L3, 2509.07628 L13), refuting the `cyracc.def` justification the old
comment carried. Full record: divergence **#78**; the corpus tables and the
retracted "the stub stays" verdict are in
[`BIBLIOGRAPHY_WORKLIST.md`](parity/BIBLIOGRAPHY_WORKLIST.md).

**The one thing to carry forward:** this row said "witness 2605.11579's
`undefined:\Dbar` is correct and stays — do not use that witness to measure this
work." The *reasoning* is still right (that paper loads no package and
`alpha.bst` has zero `Dbar`, so pdflatex fails too) but the witness no longer
**shows** the error: since divergence **#80** only cited entries are digested,
and its `KacNilpotentorbits` entry (`biblo.bib` L2059) is uncited. Still do not
measure with it — now because it is silent, not because it is parity.

### R3b — `m:menclose` is not in MathML Core — OPEN, deferred by user 2026-07-30

We emit `m:menclose` for `\cancel` / `\boxed`
(`latexml_post/src/mathml/presentation.rs` ~L520 and ~L918; Perl `MathML.pm`
L339-341 and L1507-1513 do the same). **MathML Core removed the element**, and
CLAUDE.md's standing rule is that our output targets Core. So this is a genuine
open item, not a divergence to document away.

It is deferred because unlike `<none/>` → empty `<mrow/>` there is **no
mechanical replacement**, and the two notations we emit need different answers:
- `notation="box"` (`\boxed`) → an `m:mrow` carrying a CSS border, which means
  the border has to survive the XSLT and the stylesheet, not just the MathML;
- `notation="updiagonalstrike"` (`\cancel`) → **no Core equivalent at all**.
  Options are a drawn overlay or accepting a visual regression; neither is a
  rename.

So this is a rendering change, a golden change (`tests/post/mathgolden-post.xml`
pins both today), and a deliberate divergence from Perl — it needs its own branch
and its own decision on the strike case. **Do not fix it incidentally** while
touching neighbouring pMML code.

### R3 — Presentation-MathML: F17 ✅ CLOSED 2026-07-29; F5 Linebreaker open

Both from the archived MathML-post line audit
(`archive/MATHML_POST_LINE_AUDIT_2026-07-05.md`); read the F17 bullet there for
the per-item Perl line references. **F17 was a list, not a family** — each item
individually scoped — and every one is now settled: **4 fixed, 3 do-not-port/N-A,
1 blocked, 1 unreachable in both engines.** The detail below is kept because most
of the value is in the *negative* results: three items would have introduced a
divergence or dead code if ported on the audit's word. **What remains on this row
is F5**, which is a family and needs a port-or-drop decision before any code.

The method that produced those negatives is the durable lesson: **run both engines
on the item before porting it.** Reading the audit alone would have yielded a
worse tree.

**F17 — `pmml_text_aux` styling ✅ FIXED 2026-07-29.** `pmml_text_aux` took no
`%attr` at all (Perl `MathML.pm` L1029, L1041-1045 threads font / fontsize /
color / backgroundcolor / opacity down from each enclosing `ltx:*` element), so
**every `<m:mtext>` came out unstyled**: `\textcolor{red}{\text{…}}` in math lost
its color, `\text{\textbf{…}}` lost its `ltx_mathvariant_bold` class,
`\text{\small …}` lost its `mathsize`. Two more defects in the same function:
- a **leading whitespace run was dropped instead of becoming an NBSP** — the arm
  called `trim_start()` and only then tested the *already-trimmed* string with
  `starts_with(char::is_whitespace)`, which can never be true, so `$a \text{ and
  } b$` closed up on the left (Perl L1035 is `s/^\s+/NBSP/` — replace, not trim);
- an `ltx:Math` inside an `XMText` whose `XMath` was already converted on an
  earlier pass returned `vec![]`, **silently dropping the formula**; Perl
  (L1051-1052) hands back the existing `m:math`'s children. Note the fallback
  must find that `m:math` by **namespace URI, not the `m:` prefix** — this
  processor is what introduces MathML, so `m:` is not yet in the document's XPath
  context and an XPath lookup would no-op.
Also added the `framed`/`framecolor` guard on the `ltx:text` arm plus its
`pmml_maybe_resize` (L1057-1059), and the `unexpected:nested-math` warning
(L1070-1072). Perl's `delete $mmlattr{stretchy}` (L1069) has **nothing to port**:
`%props` is filled only for `m:mo` and `$stretchy` is cleared for every other tag
(L764-767), so it is belt-and-braces over an already-absent attribute.
**The dead second copy of `stylizeContent` is gone.** `mathml/mod.rs` carried a
~245-line tag-generic `stylize_content` that **nothing called** — the live token
half had grown separately inside `presentation::pmml_token_inner` — so its `m:mo`
arm had drifted out of parity unnoticed (always-emitted `_lspace`/`_rspace`, no
`stretchyhack`, size compared against a hardcoded `"100%"`). It is now the live,
`%attr`-threading `m:mtext` half, `stylize_text_content`, and its doc comment
states the split so neither half grows the other's branches.
Guard `90_latexmlpost::mtextstyle_post_test`, **0 diff lines** — the golden is
same-host Perl 0.8.8 `latexmlpost --keepXMath --pmml` on the identical core XML,
and the test was verified RED pre-fix at **18 diff lines**. The fixture exercises
every arm of the function.

**F17 — three items CLOSED as do-not-port 2026-07-29**, each settled by running
both engines rather than by reading the audit. Do not re-open without a witness:

- **`pmml_infix` ADDOP flatten via `pmml_unrow` (L639-644) — DEAD IN PERL; porting
  it would CREATE a divergence.** `pmml_unrow` only unwraps an `m:mrow` whose
  attribute hash is empty (`!scalar(keys %{ $$mml[1] })`, L586-592), but
  `Post.pm:524-525 associateNode` stamps `_sourced => 1` on **every** `pmml()`
  result unconditionally — so the guard can never pass and the flatten never
  fires. Measured: `a+b-c+d-e+f` left-nests in both engines and Perl emits **5
  nested `m:mrow`s**, byte-identical to Rust. (`{a+b}+c` and `a+{b+c}` look flat
  in both, but that is the *parser* producing a flat n-ary `XMApp(+,a,b,c)`, not
  the flatten.) The archive's "`pmml_unrow` DEAD" note was right about Rust and
  understated: it is dead in Perl too.
- **`Apply:?:formulae` pMML arm — N-A.** Output already byte-identical
  (`a=b, c=d` / `x=1; y=2` / `p=q.`, 2 `meaning="formulae"` nodes). Perl wraps
  `formulae` in an **XMDual** (`MathParser.pm:1446`) whose *presentation* branch is
  an `XMWrap` of the original tokens, so the phantom op never reaches pMML on
  either side. `multirelation` likewise already identical.
- **`pmml_parenthesize`'s `usemfenced` branch — N-A, confirmed.** `usemfenced` is
  never set anywhere in Perl LaTeXML: no CLI option, no constructor argument — the
  only three mentions are the comment (L58), the read (L602) and the POD (L2075).
  `m:mfenced` is also gone from MathML Core.

**F17 — `outerWrapper` altimg + RDFa ✅ FIXED 2026-07-29, and it needed a second
fix in CrossRef.** `outer_wrapper` (`mathml/mod.rs`) emitted only
`display`/`alttext`/`class`, dropping two whole attribute families that Perl
copies onto `<m:math>` (L81-90):

- **the image fallback** — `altimg`, `altimg-width`, `altimg-height`,
  `altimg-valign`, from the Math's `imagesrc`/`imagewidth`/`imageheight`/
  `imagedepth`. This is the entire point of `--mathimages`: a renderer without
  MathML support had nothing to fall back to. Perl NEGATES the depth ("Note the
  sign!"), so `imagedepth="5"` → `altimg-valign="-5px"`, and omits that attribute
  for a falsy depth rather than emitting a bare `-px`.
- **the RDFa set** — `about resource property rel rev typeof datatype content`,
  from the Math element or else the XMath. A document annotating a formula with
  `lxRDFa` lost the annotation at the MathML boundary.

**The RDFa half was only half-fixed by that**, which end-to-end diffing caught:
`property`/`typeof` appeared but `about` did not, because `lxRDFa` records an
intra-document subject as **`aboutidref`** (the URL is not knowable until the
document is split — `LaTeXML-common.rnc` L301) and Rust had **no port of
`CrossRef.pm::fill_in_RDFa_refs`** (L372-398) to resolve it. Now added, in Perl's
pass position (after `fill_in_refs`, before `fill_in_bibrefs`), covering both
`about`/`resource` and both the `…idref` and `…labelref` spellings. A DB-known id
becomes a real URL via `generate_url`; an unknown one still becomes a bare `#id`,
because per Perl's own comment "RDF 'id' need not be real, valid, ids!!!".
Perl's trailing `set_RDFa_prefixes` re-run is deliberately NOT ported: this pass
only ever writes absolute URLs or `#id` fragments, never prefixed CURIEs, so there
is no new prefix to declare, and prefix management already happens core-side
(`latexml_core::document::set_rdfa_prefixes`, as in Perl `Core/Document.pm:366`).
One Perl quirk mirrored rather than fixed: with `imagesrc` present but
`imagewidth` absent, Perl emits `altimg-width="px"` — unreachable via
`--mathimages`, which always sets both, so diverging would cost byte-parity for
nothing.
Guards: `90_latexmlpost::mathouter_post_test` (Perl golden, **0 diff lines**, RED
pre-fix at 4 — the fixture also pins the two negative cases: a formula with
neither family gains nothing, and an image with no depth omits `altimg-valign`)
and `06_cluster_regressions::cluster_rdfa_math_subject` (RED pre-fix). End-to-end
`\lxRDFa[//ltx:Math]{about=#thm1,property=…,typeof=…}` is now byte-identical to
same-host Perl 0.8.8.

**F17 — `pmml_scriptsize_padded` embellishment padding ✅ FIXED 2026-07-29.**
Perl L925-934, "This is to handle primed sums, etc.", plus the `emb_right`
detection in `pmml_script_decipher` (L1015-1017) that feeds it. In
`\mathop{X'}\limits_{p}^{q}` the prime is an embellishment of the **base**, not a
script of the outer construct: Perl stops its downward walk on a post script found
*below* a mid (under/over) script, keeps the embellished `Apply(post-sup, X, ')`
as the base, and widens each limit with an invisible copy of the `'` so the limits
centre over the `X` rather than over the whole `X'` box. Rust treated the prime as
an outer postscript, which **inverted the nesting** — `msup` outside `munderover`
instead of inside — and emitted no phantom at all.
**Perl's `$emb_left` is dead code and is deliberately NOT represented**: 
`pmml_script_decipher` declares it (L968) and returns it (L1022) but never assigns
it, so the left-phantom arm of `pmml_scriptsize_padded` is unreachable upstream.
Rust therefore threads a single `emb_right`.
Guard `90_latexmlpost::mathprimed_post_test`, **0 diff lines**, verified RED
pre-fix at **20** (the diff is exactly the inverted `msup`/`munderover` nesting).
The fixture's two other formulas are negative cases that must gain no phantom.

**F17 — `combineParallel` annotation-xml wrap: BLOCKED, not portable today.**
Perl's two missing branches (L123-127) fire only for a **non-MathML** secondary —
other XML, which needs that processor's own `outerWrapper`, or an image referred to
by `src`. Rust registers exactly one parallel secondary, Content-MathML, whose
mimetype takes the *first* branch. `open_math.rs` and `math_images.rs` exist as
modules but are **wired into no pipeline** (`lib.rs:140` says so outright: "as
MathImages when they are wired up to process_chain"), and Rust's CLI has **no
`--openmath` / `--mathimages` / `--mathsvg`** at all where Perl's `latexmlpost` has
all three. So porting the branches now would be untestable dead code; the
prerequisite is that larger math-format feature. Recorded rather than written.

**F17 — `preprocess` plane1 config ✅ FIXED 2026-07-29 — F17 IS NOW CLOSED.**
`MathML::plane1` existed as a struct field, was set `true` by both constructors,
and was **never read**: the token path remapped to Plane-1 unconditionally. So
`--noplane1` could not have worked even if the flag had existed, and `hackplane1`
was absent altogether. Perl `stylizeContent` L734-736 picks the variant to remap
*with*:

```perl
my $u_variant = $variant
  && ($plane1hack ? $plane1hackable{$variant}
  : ($plane1 ? $variant : undef));
```

Now ported, with `%plane1hackable` (L659-664) and Perl's `hackplane1 ⇒ plane1`
implication (L71). New CLI: **`--plane1` / `--noplane1` / `--hackplane1`**, which
Rust previously lacked entirely where `latexmlpost` has `plane1!` and
`hackplane1!`. Measured against same-host Perl 0.8.8 on
`\mathcal{A}+\mathfrak{B}+\mathbb{C}+\mathbf{D}+\mathbf{\mathcal{E}}` — **all three
modes byte-identical**:

| mode | `\mathcal{A}` | `\mathbf{D}` | `\mathbf{\mathcal{E}}` |
|---|---|---|---|
| default | `𝒜` | `𝐃` | `ℰ` |
| `--noplane1` | `A` + `mathvariant="script"` | `D` + `mathvariant="bold"` | `E` + variant |
| `--hackplane1` | `𝒜` | `D` + `mathvariant="bold"` | `ℰ` (plain script) |

`--hackplane1` leaves `\mathbf{D}` alone because `bold` is absent from
`%plane1hackable` — the table exists precisely so the doubly-styled blocks
(bold-script, bold-fraktur) degrade to the plain codepoint no font is missing.
Guard `90_latexmlpost::plane1_modes_match_perl` drives the real processor (so the
`set_plane1` handoff in `convert_node` is exercised, not just the builder) and
asserts the negative side too — `mathvariant="bold"` must appear *exactly* when
`\mathbf` did not remap, so a build emitting both codepoint and attribute fails.
Verified RED by neutering `plane1_target_variant` to the old unconditional
`Some(variant)`; note the guard cannot be red-checked by stashing the whole fix,
because the `with_plane1` builder it calls arrives with it.

**`nestmath` deliberately NOT ported**: it has no CLI in Perl either (only
`preprocess`'s `$$self{nestmath} = 0` default and the `ltx:XMText` branch at
L497-500), so nothing can turn it on in either engine. Porting it would add an
unreachable second `XMText` arm.

**Found, not fixed — a new witness for the FUNCTION APPLICATION over-insertion
family.** `\[ \mathop{X'}\limits_{p}^{q} c \]`: Rust inserts `<m:mo>⁡</m:mo>`
before the trailing factor where Perl juxtaposes. Same family as
`opdecoration_post_test`'s `op_base_is_mo` rule, but a base shape that rule does
not cover — here the base's presentation is a `munderover`, not a bare `mo`. The
`mathprimed` fixture deliberately omits trailing operands so this does not mask the
padding assertions.

**Found, NOT F17 and NOT the post layer — a math-parser script-position bug.**
`{}^{n}a_{i}`: Rust classifies the *trailing* `_{i}` as a **prescript**.

| engine | parse |
|---|---|
| Perl | `Apply(SUPERSCRIPTOP pre1, Apply(SUBSCRIPTOP post1, a, i), n)` — `n` pre-sup, `i` post-sub ✓ |
| Rust | `Apply(SUBSCRIPTOP pre2, Apply(SUPERSCRIPTOP pre1, a, n), i)` — both `pre` ✗ |

The post stage is faithful: `pmml_script_decipher` and `apply_multi_scripts`
reproduce Perl's algorithm, and fed Perl's core XML they agree. Because the
*parser* mislabels `scriptpos`, the pMML comes out
`<m:mi>a</m:mi><m:mprescripts/><m:mi>i</m:mi><m:mi>n</m:mi>` — `a_i` rendered as
`{}_i^n a`, a **relocated subscript**, not merely different padding. Reachable
from ordinary input: found via `\sum'_{i=1}^{n} a_i`, whose `{}^{n}` empty-base
superscript produces the same shape. A fully-populated tensor
(`{}^{1}_{2}X^{3}_{4}`) is byte-identical, so the fault is specific to a
partially-filled pre/post mix. **Belongs to the math-parser family (R8), deferred
by user directive 2026-06-20 — do not fix in isolation**; recorded here with the
repro so the witness is not lost. Minimal repro: `\( {}^{n}a_{i} \)`.

Third from this pass, **fixed 2026-07-30**: a braced script chain would not fold
past depth two, nor across two scripts of the SAME kind. `scripted_factor_r2`
(`latexml_math_parser/src/grammar/builder.rs`) hand-unrolled Perl's `addScripts`
(`MathGrammar` L419-423) to exactly two, alternating —
`r12 postsuperarg | r11 postsubarg` — where Perl recurses with no depth bound and
no alternation requirement. Four shapes therefore rendered `ltx_math_unparsed`:
`{x^a}^b`, `{x_a}_b`, `{{x_a}^b}_c`, `{{{x_a}^b}_c}^d`. Braces are what make
these reachable, which is why the cap survived: bare `x^a^b` is rejected by TeX
as "Double superscript" and never reaches the parser. Now left-recursive and
unbounded; the derivation stays unique, so no new ambiguity to prune. `XMath` and
pMML are byte-identical to Perl for all of them, and a controlled A/B on
`equality_big.tex` and `ams/mathtools.tex` shows no measurable parse-time change.
Guard: `06_cluster_math::cluster_script_chain_depth`.

Also from this pass, and **fixed 2026-07-30**: Rust filled an absent
`mmultiscripts` slot with `<m:none/>` where Perl uses an empty `<m:mrow/>`. This
was briefly written up as an intentional divergence (OXIDIZED_DESIGN #86, since
removed) on the mistaken grounds that `m:none` is the element MathML designates
for an empty slot. **MathML Core removed `<none>`**; an empty `m:mrow` is the
accepted placeholder for an omitted subtree, so Perl's spelling is both the
faithful and the standards-current one. `apply_multi_scripts` now emits it.
Guard: `90_latexmlpost::scriptlevels_post_test`.

### R6 — `ltx_env_<name>` env-markup class — PHASE 2, do NOT start yet
**Deferred by user directive 2026-07-29: this waits until the parity and
first-arXiv-release milestones are done.** It is a beyond-Perl styling
enhancement, not a parity gap, and its golden churn (an additive class on every
env element in nearly every test XML) would sit on top of release-critical work.
Pick it up only after those milestones land — then on its own branch.

Design notes, kept so the analysis is not re-derived:
- `Document::open_element` (`latexml_core/src/document.rs`) is the single funnel
  for element creation, so one armed-in-`before_construct` /
  consumed-by-first-`open_element` slot on `Document` tags exactly the env's
  wrapper element. That survives the schema's auto-open/auto-close (which defeats
  a parent-anchor + child-count mark for `figure`/`table`) and needs **no** node
  gid — so the "needs a globally-unique monotonic node gid" prerequisite below
  applies only to the raw `\newenvironment` half.
- Name sanitizing is already available: `clean_class_name` (Perl
  `Package.pm:527 CleanClassName`), giving `figure*` → `ltx_env_figure`.
- 302 of 305 `DefEnvironment!` sites are template-based and 3 are closures; both
  paths funnel through `open_element`, so one hook covers all of them.
- `add_class` merges with a template's existing `class` and is schema-filtered,
  so `minipage` would become `class="ltx_env_minipage ltx_minipage"`.
- Golden churn is mechanical via `tools/maketests.sh` (`LATEXML_BLESS=1`), with a
  filtered diff to prove only `class=` changed.

**User-requested generic enhancement** (2026-06-27): tag environment wrapper markup
with `class="ltx_env_<name>"` so custom/minipage-like envs (e.g. `SideBySideExample`)
become responsively styleable in CSS instead of fixed-width minipages. **MUST be on a
dedicated branch** — it changes nearly every test XML (additive class on every env
element), so the golden-suite update is large and must be done in isolation.
Two implementations, same markup outcome:
- **Binding side (`DefEnvironment!`):** the constructor guarantees exactly one element,
  so unconditionally add `ltx_env_<name>` (via an `@ADDCLASS`/`add_class` after the
  begin constructor opens). Applies to ALL DefEnvironments (`figure`, `table`,
  `theorem`, `minipage`, …) — user chose full scope.
- **Raw side (`\newenvironment`/`\renewenvironment`):** arm at env start; at `\begin`
  construction record `{name, anchor = globally-unique gid of current node, mark}`; at
  `\end` afterConstruct, if EXACTLY ONE element was deposited under the anchor since
  the mark → tag it; zero (font/text-only) or >1 (siblings, e.g. SideBySideExample's
  parboxes) → nothing. **Needs a globally-unique monotonic node gid** (verify/ add;
  `record_node_ids` exists but is xml:id-oriented).
- **SideBySideExample:** keep the working `fancyvrb-ex` raw-load (correct source+result)
  + drive responsive layout from the resulting `ltx_minipage`/`ltx_env_*` hooks in
  `ar5iv.css`; do NOT re-implement the verbatim+render dual capture.

### (not ranked) `\gls`/`\acrshort` in MATH mode, 1705.10306 — PARITY, blocked on unrunnable Perl — do not chase
293 errors `ltx:XMTok isn't allowed in <ltx:glossaryref>`: a glossary command in
math mode digests the link display text (#3, the literal acronym term) as math →
bare per-letter `<XMTok>`, which the `glossaryref` content model rejects.
**Source-confirmed 2026-06-27 that this is most likely PARITY (NOT a Rust-only
gap — the cortex "Perl 1" is stale/unreliable, per `use-cortex-for-parity-work`):**
- Perl `Stomach.pm::enterHorizontal` (L422-434) is a **no-op in math** (`$mode
  =~ /math$/ => {}`) — Rust's `enter_horizontal` matches faithfully. So the
  `enterHorizontal => 1` on the shared `\lx@glossaries@gls@link` constructor does
  NOT switch #3 to text in math in EITHER engine.
- BOTH engines raw-load the SAME `glossaries.sty` (`InputDefinitions(noltxml=>1)`)
  with the SAME override constructor → both digest #3 in the ambient math mode →
  both produce `glossaryref > XMTok` → both hit the same schema rejection.
- `\ref`/`\cite` in math do NOT error (verified) — their content is STRUCTURED
  (bibref / ref-number), not a literal term; only `\gls`/`\acrshort` emit raw
  letter-XMToks. So glossaryref is specific, but the mechanism is shared with Perl.
- **The earlier "Perl raw-loads glossaries.sty and typesets as TEXT" hypothesis is
  weakened:** Rust raw-loads the identical `.sty`, so if it typeset the term as
  text, Rust would too. It doesn't (output: italic letter-XMToks) → so the `.sty`
  display chain does NOT force text in math.
**Perl confirmed UNRUNNABLE here (2026-06-27):** `latexml glx.tex` → `Fatal:terminate`
in `expl3-code.tex` (l3kernel) at 150 s — glossaries pulls in expl3 which is
pathologically slow in Perl 0.8.8 on this host; cannot capture ground truth.
**Fixing is therefore deferred as a likely non-bug.** If pursued, it parallels the
figure_mixed_content surpass-Perl pattern (a monotonic schema expansion to accept
the math content the builder already produces) — BUT the correct structure is
genuinely uncertain without Perl (XMTok directly? XMText-wrapped? operator-token
for the `\DeclareMathOperator` case? text PCDATA?), and there is **no precedent**
for `XMTok` in any inline element's model, so a speculative change risks an
unfaithful divergence. Repro + full notes:
`docs/reproducers/glossaryref_math_xmtok.tex`.

### (not ranked) Font-selection chain audit, 2026-07-30 — residuals

**STATUS 2026-07-30, end of day: 6 of the audit's findings are FIXED and merged
(PRs #450, #452, #453, #454, #455, #456). What is left is recorded below.**

Every merged fix was re-verified against same-host Perl 0.8.8 on the fully merged
binary: 8 of 10 scenarios byte-identical, the two exceptions being the two gaps
listed under "Still open" — i.e. nothing diverges that was not already known.
Witness 2503.04421 sits at 99.922% token similarity with dingbats matching exactly
(15 ✗ / 13 ✓ both sides).

**The recurring shape, and the reason this section exists.** Four of the six fixes
were the SAME defect class: a faithfully translated helper with **zero callers** —
`ding_fontmap.rs`, `font::decode_str`'s FontDecode variant, `font::lookup_tex_font`,
`font::rationalize_font_size`. The tables and functions were correct; nothing
invoked them, and the fallback output stayed plausible, so no test could see it.
**The compiler already catches this — if visibility is honest.** A `pub fn` in a
library crate is public API, so `dead_code` stays silent no matter how unreachable
it is. Mark an internal helper `pub(crate)` and rustc says
`function \`x\` is never used` — which CI's `cargo clippy --workspace
--all-targets -- -D warnings` turns into a hard failure. Verified on
`font_decode_string`. So the rule is not "run an audit", it is **do not write
`pub` on a helper that is not cross-crate API**; the four dead helpers were all
over-exposed, which is exactly what silenced the lint.

Two caveats, because a clean lint is not proof of wiring:
- It cannot see a dead *binding module*. `ding_fontmap.rs` was reachable only if
  something assigned `encoding = "ding"`, which `\selectfont` derives dynamically
  from the family. For declared tables ask instead: *what assigns the key this
  table is filed under?*
- It cannot see a call graph wired to the WRONG live node. `\char` called
  `decode_str` when it needed the `FontDecode` sibling — the two differ by a
  single `|| 'OT1'`. Nothing is uncalled; only reading both Perl definitions
  catches it.

#### Two known-bad values PINNED by tests — do not "fix" by loosening the assert

These assert today's WRONG output on purpose, so that repairing the underlying bug
turns the test red and forces an update. Each says so in its own doc comment.

- `120_multichar_slot_paths.rs::declare_math_symbol_still_drops_the_overlay_known_gap`
  — `\DeclareMathSymbol` routes through `mathchar.rs`, whose `props.glyph` is
  `Option<char>`, so a two-character slot loses its combining mark: T2B 128 gives
  `Ӷ` where Perl gives `Ӷ̶` (U+04F6 U+0336). Fixing it means making that field a
  string, a real type change through the math-char pipeline.
- `118_delimiter_size_nominal_font.rs` pins **4 `Error:expected:<variable>`** from
  `a0poster` — Rust-only, Perl 0.8.8 emits **zero** on the same input. Unrelated to
  delimiter sizing; nobody has diagnosed it.

#### Still open, with verification status

- **`\cal ABC` collapses to one `<mi>`** and drops `class="ltx_font_mathcaligraphic"`
  — VERIFIED by me. Perl emits three `<mi>` elements, Rust one containing `𝒜ℬ𝒞`.
  This is the `<enc>_<category>_mathstyle` gap: Perl's `FontDecode`
  (`Package.pm` L2884-2889) keeps the alphanumeric as ASCII and records the semantic
  font; Rust decodes to the styled codepoint. NOT a searchability issue — both
  engines emit styled Unicode in the final HTML, which is the MathML-Core-correct
  answer; the defect is the token grouping and the missing class.
- **No base font map for encoding `U`** — VERIFIED by me, and **SHARED with Perl**,
  not a port gap. `U` is the catch-all for every symbol font (`manfnt`,
  `latexsym`/`lasy`, `mathdesign`, XY-pic, `mathx`, `MnSymbol`), so they decode
  through the OT1 fallback and print that slot's TEXT character. Reproduce with
  `--includestyles` (the ar5iv profile) — it does NOT appear in the default profile:
  `\usepackage{manfnt}` + `\dbend` gives `¨` (U+00A8) in **both** engines.
  Structural: because the family lookup is nested inside
  `if let Some(map) = load_font_map(encoding)` — byte-faithful to Perl — a
  `U_lasy_fontmap` can never be consulted until `U` itself has a base map. Corpus
  base rate: 2.0% of 547 sampled papers emit any font diagnostic at all, and the
  log census is a LOWER bound because a family-map miss on a mapped encoding is
  completely silent.
- **`\mathversion{bold}` merges the text font instead of `mathfont`** — ✅ **FIXED
  2026-08-05** (see sweep #5 below).
- **`\DeclareTextCommand`/`\ProvideTextCommand` don't install the encoding-dispatch
  chain** — VERIFIED still-broken 2026-08-05 (`latex_constructs.rs:6525`/`6544`).
  Kernel accents are masked by the dump, so only package-declared text commands
  (tipa T3, T2A extras, TS1 additions) would show it. Rust's `\DeclareTextSymbol`
  *does* install the chain. (Same item as sweep #6 below.)
- **Slot-by-slot table parity**: 4048 slots compared across all 24 maps, reported as
  0 wrong characters / 0 off-by-N / 0 extra. I verified the AMSb subset myself; the
  remaining totals are a single-agent claim (its Perl parser was cross-validated by
  running real `perl`, and `ding[0x21]`=U+2713 was hand-checked).
- One `fontsize="40%"` element differs in count under `a0poster` (Perl 4, Rust 3),
  before and after the delimiter fix. Unexplained.

#### The original sweep (kept for its file:line pointers)


Sweep of every `selectfont` occurrence in `LaTeXML/lib/LaTeXML/` (22 hits) plus the
sibling primitives, done while fixing the family-as-encoding gap above. **The
`\selectfont` question itself is CLOSED**: Perl has exactly one
`DefPrimitive('\selectfont')` (`latex_constructs.pool.ltxml:5202`) and **no package
or class anywhere redefines or `Let`s it**, so there is no second definition site to
port. The 9 family/series/shape switches, `\normalfont`, `\verbatim@font`,
`\usefont`, `subfigure`'s 6 font hooks, `\fontencoding`/`\f@encoding`,
`\DeclareFontShape`/`Family`/`FixedFont`/`Encoding`/`Subset`, `\symbol`,
`\fontsize`, `\Declare{Text,Old}FontCommand`, `\DeclareSymbolFontAlphabet` and
`\try@load@fontshape` all verified faithful.

What the sweep turned up is in *sibling* primitives — same subsystem, same class of
symptom (a font attribute or glyph silently wrong, no diagnostic). Ranked, with
verification status stated explicitly; **the unverified rows are single-agent claims
with file:line, not established facts — re-verify before acting**:

1. **`\char`/`\symbol` yield the EMPTY STRING in math mode** — ✅ **FIXED**
   (re-verified 2026-08-05; guard
   `117_char_font_decode::char_decodes_through_ot1_in_math_and_does_not_wrap_out_of_range`,
   ground-truthed vs Perl 0.8.8). `font::decode` defaults the encoding to `OT1`
   when the current font carries none — matching Perl `FontDecode`
   `$font->getEncoding || 'OT1'` (`common/font.rs:2103-2126`, `Package.pm:2877`);
   `decode_str`/`FontDecodeString` keep the deliberate empty fallback, so the
   asymmetry is intended — do NOT "align" them. `\char` uses `u8::try_from`, not
   `as u8`, so out-of-range `\char300` yields nothing like Perl instead of `,`
   (`tex_character.rs:61-84`).
2. **`\DeclareSymbolFont`'s encoding arg is `ExpandedPartially`** — ✅ **FIXED**
   (guard `117_char_font_decode::symbol_font_encoding_argument_is_expanded_before_storage`).
   `latex_constructs.rs:6907` declares `\DeclareSymbolFont{} ExpandedPartially
   {}{}{}` (Perl `latex_constructs.pool.ltxml:2664`), so the `\encodingdefault`
   that `fontmath.ltx` writes expands before storage and dependent
   `\DeclareMathSymbol`/`\DeclareMathAccent` lookups hit the right fontmap.
3. **`DeclareFontMap`'s `(uppercase|lowercase|digit)_mathstyle` options are
   unported** — VERIFIED write-only by me: `tex_fonts.rs` writes
   `OMS_uppercase_mathstyle`, `amsb_fontmap.rs:2` records a dropped blackboard
   `uppercase_mathstyle` in a comment, and nothing reads either key. Perl's
   `FontDecode` (`Package.pm:2884-2889`) uses them to keep an alphanumeric as ASCII
   while recording the semantic font change. Claimed-but-unmeasured consequence:
   `$\cal A$` double-styles (U+1D49C *and* `font=caligraphic`) where Perl gives `A`
   + caligraphic, and hands a non-ASCII letter to the grammar.
4. **`\DeclareMathAlphabet` skips `lookupTeXFont`** — ✅ **FIXED** (re-verified
   2026-08-05): `latex_constructs.rs:7006` calls
   `font::lookup_tex_font(&family, &series, &shape)` (the abstract
   `sansserif`/`medium`/`upright` mapping, not raw NFSS codes) and `:7004` emits
   `Info!("ignore", …)` on the already-defined branch — matching Perl
   `latex_constructs.pool.ltxml:2677`. The def is at `latex_constructs.rs:6998`
   (`:6957` is now `\new@internalmathalphabet`).
5. **`\mathversion{bold}` merges the text font, not `mathfont`** — ✅ **FIXED
   2026-08-05** (`latex_constructs.rs:10653`). It was `MergeFont!(forcebold =>
   true)` (merges the current TEXT `font`), so math never went bold; now it does
   `AssignValue(mathfont => LookupValue('mathfont')->merge(forcebold => N),
   'local')` exactly like `\boldmath`/`\unboldmath` (Perl
   `latex_constructs.pool.ltxml` L5290-5297), and an unknown version raises
   `Error('unexpected', …)` instead of the silent `_ => {}`. Guard
   `06_cluster_regressions::mathversion_switches_the_mathfont_like_boldmath`.
6. **`\DeclareTextCommand`/`\ProvideTextCommand` don't install the encoding-dispatch
   chain** — VERIFIED still-broken 2026-08-05 (`:2584`/`:2598` vs
   `latex_constructs.rs:6525`/`6544`, which bind the bare `\cs` to the raw
   first-encoding expansion): the first encoding to declare a CS wins permanently.
   Kernel accents are masked by the dump, so only *package*-declared text commands
   (tipa T3, T2A extras, TS1 additions) show it. Rust's `\DeclareTextSymbol`
   (`:6624`) *does* install the chain, so the two are inconsistent.
7. Lower: `\DeclareTextSymbol` decodes eagerly at declaration instead of installing a
   deferred `CharDef` (loses the glyph permanently if the fontmap is not yet
   loaded); `LoadFontMap` never emits Perl's `Info('fontmap', …)`;
   `\DeclareErrorFont` is a bare no-op where Perl defines its arg as `\relax`.
   (The `\textit@math` `\f@shape` `i`-vs-`it` sub-item is ✅ FIXED —
   `latex_constructs.rs:10568` now assigns `it`.)

`docs/parity/OXIDIZED_DESIGN.md` has no font section, so none of these is a
documented divergence. Method and the two detection traps: [`WISDOM.md`](parity/WISDOM.md) §80.

## Parked families — pointers, not content

Each outgrew this file and now lives on its own. Read the doc before starting;
several carry explicit "do NOT start" directives.

| family | rank | doc |
|---|---|---|
| Bibliography targets + MakeBibliography re-port | R5 | [`parity/BIBLIOGRAPHY_WORKLIST.md`](parity/BIBLIOGRAPHY_WORKLIST.md) |
| Beyond-Perl performance levers BP-1…BP-6 | R7 | [`performance/BEYOND_PERL_LEVERS.md`](performance/BEYOND_PERL_LEVERS.md) |
| Content-MathML / math-parser gaps | R8 | [`math/CONTENT_MATHML_GAPS.md`](math/CONTENT_MATHML_GAPS.md) |
| Deep deferred families (`.bst`, xy-pic, mode-frame, …) | R9 | [`parity/DEFERRED_FAMILIES.md`](parity/DEFERRED_FAMILIES.md) |
| Two-pass streaming split | deferred | [`performance/STREAMING_POST_DESIGN_2026-07-06.md`](performance/STREAMING_POST_DESIGN_2026-07-06.md) |

## Reference (stable — not active work)

### Engine file open gaps (MINOR, demand-driven)
- `tex_box.rs` box-dimension edges; `tex_fonts.rs` `\fontdimen` array + per-font
  `\hyphenchar`; `tex_tables.rs` padding CSS (XSLT concern).
- **Document-builder block/paragraph auto-wrap of inline content** (core,
  broad/risky family — two witnesses):
  - **`\fcolorbox` inline paragraph-grouping**: an inline `\fcolorbox`
    mid-paragraph — Perl breaks the `<p>` (its `internal_vertical` block ends
    it), Rust keeps it inline. SAME flags on both; Rust's inline reading
    arguably matches real LaTeX's `\mbox`-based `\fcolorbox`. (`\colorbox`
    matches.)
  - **bare `\includegraphics` run in a figure** (witness 1108.0198, found
    2026-06-21 via skeleton diff — a clean, error-free reproducer): a
    `\begin{figure*}` with several consecutive `\includegraphics` (no blank
    line) — Perl wraps the inline run in a `<ltx:block>` (`figure > tags >
    block > graphics×N`), Rust emits the graphics bare (`figure > graphics×N`).
    Rust is error-clean and schema-valid. **Re-witnessed + root-confirmed
    2026-06-27** (0704.0001, 0704.0017 via the corrected structural diff): NOT
    merely cosmetic — the panel `<graphics>` WIDTHS also diverge (Rust 303.5pt vs
    Perl 241.5pt, ~1.257×), so figure sizing is visibly affected.
    **The break/block-arrangement half LANDED (PR #273, 2026-07-05):**
    `arrange_panels` (`latex_constructs.rs` ~1967, doc-comment "Faithful port of
    Perl's `arrange_panels_and_breaks`") now inserts `<ltx:break class="ltx_break"/>`
    (`insert_break_before` ~1944) and does the block/merge arrangement, wired in at
    three float call sites — so the old "Simplified: mark panel children with the
    class" state is gone. What remains OPEN is only the box-WIDTH divergence (tied
    to the deep box session / the `\resizebox` panel-width item below); re-witness
    1108.0198 / 0704.0001 / 0704.0017 to confirm whether widths still diverge now
    that the arrangement is ported.
- **`\resizebox` panel scale-VALUE divergence**: in `complex/figure_mixed_content`
  two panels get a different computed natural width (xscale 1.13 vs 0.88). The
  construct in ISOLATION matches exactly (both xscale=1.9685); the divergence
  only appears inside the paper's `\footnotesize` + `table*` + `\subfloat` panel
  context → a font-size/box-context interaction. Scale *formatting* (%.15g) is
  already Perl-faithful (`551c5286ba`); missing-image candidates too
  (`64dd30b284`). Deep box-metric; for the focused box session.
- **~72-CS Perl-only long tail** (from the archived LoadFormat audit): misc
  atomics (`\@charlb`, point-size CSes, `\batchmode`, …) Perl defines, Rust does
  not. Investigate a CS only when a real paper witnesses it; refresh the CS-name
  diff before quoting counts (predates the BibTeX port).

### Primitive layer — AUDITED FAITHFUL (2026-06-20)
Probe-based Rust-vs-Perl audit found the core primitive layer byte-identical
(arithmetic, dimensions, glue, conditionals, string/token, case tables). Don't
re-audit without a witnessing paper. Shared-with-Perl quirks (NOT Rust bugs):
`\numexpr` divideround round-half-toward-+∞ (KNOWN_PERL_ERRORS #33); `\the\skip`
drops stretch/shrink to bare pt.

### Permanent ignores
- **Out-of-scope**: ns1–ns5 (`52_namespace`, no DTD support); 2402.03300,
  2410.10068, 2511.03798 (Perl also fails).
- **Rust supersedes Perl**: `1207.6068`, `0909.3444`, + 40 more in
  `memory/project_rust_supersedes_perl.md`.
- **2026-07-20 ar5iv sprint (PR #323) residuals — do not re-mine.** Its three
  ar5iv leftovers all resolve parity-or-Rust-better, none Rust-only
  (`AR5IV_DIAGNOSTICS.md`); its TL2026 dump-gate scrap closed 2026-07-23 and
  was re-confirmed on `main` 2026-07-25 (0 errors on both inits inside
  `ghcr.io/tkw1536/texlive-docker:2026`; 2026 is in the release window). Both
  in `archive/SYNC_SESSIONS_2026-07.md`.
- **BibTeX**: `BibTeX.pool.ltxml` ported (Phases 1–8; remaining B1–B6 polish in
  `BIBTEX_PORT_PLAN.md`). `--nobibtex` is opt-out, not default.

### Tikz known diffs vs Perl
`foreignObject` transform; arrow-tip path data; SVG viewBox/width; matrix
`<svg:g class="ltx_tikzmatrix">` vs inline-blocks; **bare `svg:g` in `<ltx:block>`**
(tikz-cd) trips a core-XML validity error but post-processing recovers (witness
2006.12702) — Rust-only, low priority (output recovered).

### Graphics renderer chain (subprocess-only; LANDED)
PDF→PNG `mutool draw`→`pdftocairo`→`convert+gs`; PDF→SVG `mutool convert`→
`pdftocairo`→(raster PNG fallback). EPS/PS→`gs` direct→`convert+gs`. Subprocess
`exec` (no GPL linking). Apt: `poppler-utils` (req), `mupdf-tools` (rec),
`imagemagick+ghostscript`. A heavyweight inkscape third resort for PDF→SVG was
removed 2026-06-29 (GTK stack, 20–40× slower, timeout-prone, no coverage over the
raster fallback).

### Other tracks (separate docs)
- Performance: `PERFORMANCE.md` (P1 math/large-doc open; P2 allocation partial).
- Release gates: `RELEASE_CRITERIA.md`. Releasing: `RELEASING.md`.
- **BibTeX (plan archived 2026-07-02 →
  [`archive/BIBTEX_PORT_PLAN_2026-06-20.md`](archive/BIBTEX_PORT_PLAN_2026-06-20.md)):**
  Phases 1–8 shipped; live residuals = the Phase 4–5 field-handler/MR-Zbl
  long tail, divergences B1–B6 noted in `bibtex.rs`, and the deferred
  **native `.bst` interpretation** (witness 2605.16562, `f65cf7d6dc`) —
  demand-driven, pick up on corpus evidence.
- Completed missions (archived): strict-LoadFormat dump parity, Marpa ASF
  migration, distribution-readiness, the 500K/1M warning-corpus mission, the
  diagnostic-message faithfulness pass (2026-06-20), and the upstream-sync
  PR translation U1–U11 (2026-06-26) — see `docs/archive/` and `git log`.
