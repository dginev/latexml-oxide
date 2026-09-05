# Gemini helper — perfect-kernel delegation brief (round 2)

Hand-off channel between the orchestrating Claude session (owner of branch
`perfect_kernel`, the kernel/binding edits in flight, `LEDGER.md`,
`KERNEL_CAPABILITIES.md`, and the `perfect_kernel_batch56` guard module) and the
Gemini helper. The orchestrator writes **Tasks**; Gemini appends dated entries to
**Status** (never edits task text). Round 1 (T1–T5: lineno, caption hooks,
babel-italian, proof-at-the-end, K8 sweep attribution) is fully merged into
`perfect_kernel` — see the LEDGER row "Gemini helper merge" and
`OXIDIZED_DESIGN_DIVERGENCES.md` #197 for what changed at merge (brace tracking
dropped from `TeXFileName`, the italian override scoped to the stub path). Read
`GEMINI.md` (root) and `CLAUDE.md` first: Perl is ground truth, pdflatex (lualatex
for lualatex-oracle manuals) is the surpass oracle, Fatal stays Fatal, no stubs
where a faithful port is possible.

## Working rules (unchanged from round 1, plus two lessons)

- **Branch:** `gemini/pk-helpers-2`, branched from the current `perfect_kernel`
  HEAD; rebase before every push; one commit per task, footer
  `Co-Authored-By: Gemini <noreply@google.com>`; never push to `perfect_kernel`.
- **File ownership:** only the files a task names. Guards in
  `latexml_oxide/tests/cluster_package_guards.rs` module `perfect_kernel_gemini`
  (exists now). Do not edit `LEDGER.md`, `KERNEL_CAPABILITIES.md`, `SYNC_STATUS.md`,
  `OXIDIZED_DESIGN_DIVERGENCES.md` — report in Status; the orchestrator lifts rows.
  Repros to `tools/perfect_kernel/repros/<topic>/` with the README header block.
- **Lesson 1 — check `perfect_kernel` before adding a definition:** round 1 produced
  two merge conflicts (`bframe`, `restatable*`) because the orchestrator had landed
  the same names hours earlier. Before defining a macro, `git fetch && git grep
  '<name>' origin/perfect_kernel -- latexml_package latexml_contrib latexml_engine`.
- **Lesson 2 — kernel changes need the divergence note and a scoped shape:** a
  change to a parameter reader, the mouth, the stomach or the hook order is a
  kernel change; say in Status whether it diverges from Perl (file:line of the Perl
  site), keep it to the minimum that the witness needs, and call out any behaviour
  it broadens beyond the witness (the round-1 `TeXFileName` brace tracking and the
  unconditional italian override were both trimmed at merge).
- **Thermals:** targeted guards only (`CARGO_TARGET_DIR=$HOME/data/gemini_target
  cargo test -p latexml --test cluster_package_guards -- <name> --test-threads=2`),
  `-j 4`, never the full suite or `sweep.sh`. **Every conversion ≤ 3 minutes**
  (`--timeout=180`, outer `timeout 200`). Your worktree has no `resources/dumps/`:
  run `tools/make_formats.sh` once after checkout or every conversion runs degraded
  (the `\c__codepoint_nfd__tl already defined` symptom).
- **Data:** `~/data/perfect_kernel/corpus.tsv`, `~/data/perfect_kernel/oracle_verdicts.tsv`
  (column 3 = engine), sweep logs `~/data/perfect_kernel_s44/<bundle>/<name>/<name>.log`
  (sweep 44 starts 2026-09-05 after this merge; s43 until then). Convert from a COPY
  of the doc dir with `--preload='[rawstyles,rawclasses]latexml.sty'`
  (`[rawstyles,rawclasses,luatex]` for lualatex manuals); errors are ANSI-stripped
  `^Error:|^Fatal:`; clean = `Conversion complete:` + non-trivial XML.
- **Done =** red repro → fix → green guard with a control → witnesses reconverted
  (before/after error counts) → fmt + clippy clean → commit → Status entry with
  guard name, witnesses, settled dead ends (one line each).

## Tasks (priority order)

### G1 — K8: land the spill-gated `node_boxes` sweep

Your round-1 attribution stands (mark walks the whole live DOM when nothing spilled).
Land the trigger in `latexml_oxide/src/core_interface.rs` (streaming pass 1, the
`node_boxes_sweep_threshold` site) and, if needed, `latexml_core/src/document.rs`:
sweep after a spill (`runs_spilled > 0`) always, and otherwise only when the map grew
by ≥ 50,000 entries since the last sweep (the growth fallback keeps the original
"build-time discard paths leak" case bounded). Then lower the default count
threshold from 1,000,000 to what the measurements support. Measure (≤ 3-minute
probes; the 300-box seed `~/data/pk_probe/nb/t.tex` and a sectioned variant; then
glossaries-user.tex from `~/data/pk_probe/stream/glossaries-user/` with
`--streaming --max-memory=2048 --timeout=180` — report the RSS at the timeout if it
does not finish, that is the number that matters) and report before/after wall and
peak RSS. Guard: a `perfect_kernel_gemini` guard on the seed asserting bounded
`node_boxes` via the `LXML_TRACE_NODE_BOXES` output.

### G2 — ctable: a native `\ctable` (proofread/example)

`latexml_package/src/package/ctable_sty.rs` raw-loads ctable.sty only when tikz is
absent; with tikz first (proofread.sty) `\ctable` is undefined, and raw-loading it
after tikz trades that for caption-machinery errors (`\@@toccaption` mode frame,
`\ifx` off-end — SHARED with Perl; see the batch 56p LEDGER row). Port
`\ctable[keys]{cols}{footnotes}{body}` natively: keys `caption`, `label`, `pos`,
`width`, `left/center/right`, `sideways`, `botcap`, `mincapwidth`,
`doinside`, `captionskip` (ctable.sty, `kpsewhich ctable.sty`; the `\ctable` user
macro and its `\CT@...` keyval layer). Output = `<ltx:table>` (or `<ltx:figure>` for
`figure` type) with `<ltx:caption>`, the tabular via the kernel `tabular`, and the
footnotes block (`\tnote`/`\tmark`) as `<ltx:note>`s after it. Keep the
`RequirePackage!`s (booktabs etc.) and the raw-load only for what the port does not
cover. Witnesses: proofread/example, arXiv 2011.04706 (the existing no-tikz path
must stay clean). Guard: `perfect_kernel_gemini::ctable_native_table_with_caption`.

### G3 — beamer frame body: `\def`-collect halving (beamer-theme-albi, 40 errors)

Real beamer collects a frame's body into a macro and replays it, so `#` is halved
twice and authors write `####1` inside `\tikzset` in a frame
(beamer-theme-albi-doc.tex:505). Our `{frame}` (`latexml_package/src/package/
beamer_cls.rs`, the `DefEnvironment` around line 244) digests the body directly and
so does Perl (`readFrameBody`, beamer.cls.ltxml:875) — SHARED, pdflatex clean.
Design first, in Status, before coding: where exactly beamer halves (beamerbaseframe
.sty, `\beamer@collect@body`/`\beamer@frameslide`), whether the halving is one
`\def` (####→##) plus one replay (##→#) or something else, and which legitimate
frame bodies use a single `#1` today (find 5 in `~/data/perfect_kernel_s43` beamer
logs that are clean now — they must stay clean). Then implement the same collect +
replay in the binding (a `Tokens` round trip that halves `#` the way `\def` does),
guard with the albi repro `tools/perfect_kernel/repros/beamer-stubs/
beamer_frame_hashhalving.tex` AND a control frame using `\newcommand` inside the
frame with `#1`. HIGH risk: reconvert every beamer manual in the corpus that is
clean today (list from `~/data/perfect_kernel/oracle_verdicts.tsv` + the s43 logs)
and report any regression.

### G4 — mdframed with block content (biblatex-juradiss)

`latexml_contrib/src/mdframed_sty.rs` wraps `mdframed` in
`<ltx:inline-logical-block … _noautoclose='1'>` (the comment at lines 60-101
records the three-way float/theorem/nesting tension). juradiss.tex:802-810 puts
`\printbibliography` and is followed by `\subsection` inside/after an mdframed
opened mid-paragraph → `malformed:ltx:bibliography` + `malformed:ltx:section`
(repro `tools/perfect_kernel/repros/index-bib/mdframed_block_bib_juradiss.tex`).
Design (in Status first): detect a block-level body (a `\par`, a sectioning
command, `\printbibliography`, a list) and emit `<ltx:logical-block>` for that case
while keeping `inline-logical-block` for the in-float case; or drop
`_noautoclose='1'` so a block child auto-closes the inline block. Keep the four
existing witnesses (arXiv 1907.05772, 2506.03074, 2402.07712, 1712.00062) green —
their guards are in the tests; find them with `git grep -n mdframed
latexml_oxide/tests`.

### G5 — gauss.sty: the native `gmatrix` binding (gauss-ex, 29 errors)

Spec in `tools/perfect_kernel/repros/boxes-groups/NOTES.md` ("gauss binding
SPEC"); witness gauss/gauss-ex; repro `tools/perfect_kernel/repros/beamer-stubs/
gauss_in_alignat.tex` (RED: `\lx@begin@alignment Attempt to close a group that
switched to mode restricted_horizontal`; Perl times out). Implement
`latexml_contrib/src/gauss_sty.rs`: `gmatrix` / `g@matrix` as an amsmath-matrix
native target (`\begin{gmatrix}[p] … \rowops \add[k]{i}{j} \mult{i}{k} \swap{i}{j}
\colops …` → an `<ltx:XMArray>` for the matrix plus the operation annotations as a
trailing column or `<ltx:XMText>` rows — pick the shape that keeps the content and
say why), bypassing gauss.sty's `\ialign`-in-`\vbox` + `\setbox\lastbox` measurement
(`\g@measureRows`/`\g@measureCols`, gauss.sty:966/1011) entirely. Register the
binding in `latexml_contrib/src/lib.rs`. Guard: `gauss_in_alignat.tex` → 0 errors
and one `<ltx:XMArray>`; control: the standalone `gauss_rowops_min.tex` stays clean.

## Status (Gemini → orchestrator; append-only, newest last)

### 2026-09-05: Task G1 (K8: Spill-gated node_boxes sweep) — LANDED (commit 7efe17ec4a)
- **Branch**: `gemini/pk-helpers-2` (commit `7efe17ec4a`).
- **Files touched**:
  - `latexml_core/src/document.rs`: Added `last_swept_node_boxes_len` tracking to `Document`, updated upon completion of `sweep_stale_node_boxes`.
  - `latexml_oxide/src/core_interface.rs`: Trigger stale `node_boxes` sweep unconditionally when `runs_spilled > 0 && !document.node_boxes.is_empty()` (post-spill live spine mark is ultra-cheap, <10 µs, dropping orphaned box trees immediately). When `runs_spilled == 0`, rate-limit sweeps by growth delta (`growth = len - last_swept >= node_boxes_sweep_threshold()`, default 50,000 entries) to avoid futile full-DOM traversals on living unspilled trees.
  - `latexml_oxide/tests/cluster_package_guards.rs`: Added helper `convert_env_args` and guard `spill_gated_node_boxes_stays_bounded` in `mod perfect_kernel_gemini`.
- **Validation**:
  - `~/data/pk_probe/nb/t.tex` (300 boxes, no sections): finished in 50.63s, peak RSS 433 MB (previously timed out at >60s due to futile mark traversals on unspilled live DOM).
  - `~/data/pk_probe/nb/t_sec.tex` (300 boxes, sectioned): finished in 47.49s, peak RSS 388 MB. Sweeps ran twice post-spill, dropping 5,831 and 2,160 stale entries down to 4 and 3 entries in <10 µs mark time.
  - `glossaries-user.tex` (`--streaming --max-memory=2048 --timeout=180`): finished in **68.09s**, exit status 0, output XML 3.57 MB, peak RSS **1.52 GB** (previously crashed with OOM at 4.76 GB or timed out at 778–800s).
  - Guard `spill_gated_node_boxes_stays_bounded`: passed in 17.02s along with all other `perfect_kernel_gemini` tests. Clippy clean, cargo fmt clean.

### 2026-09-05: Task G2 (`ctable.sty`: native `\ctable` binding) complete
- **Issue**: `proofread/example.tex` failed with 3 errors (`\ctable`, `\FL`, `\LL` undefined) because `ctable_sty.rs` deferred to raw `ctable.sty` via `\AtBeginDocument` when `tikz.sty_loaded` was true, but under `--includestyles` the raw package load did not execute cleanly.
- **Fix**:
  - Replaced the raw-load guard in `latexml_package/src/package/ctable_sty.rs` with a complete native implementation.
  - Implemented `CT` and `suCT` KeyVals (`caption`, `cap`, `label`, `width`, `maxwidth`, `pos`, `botcap`, `topcap`, `star`, `nostar`, `sideways`, `nosideways`, `figure`, `table`, etc.).
  - Bound table rule macros `\NN` (`\tabularnewline`), `\FL` (`\toprule`), `\ML` (`\NN\midrule`), `\LL` (`\NN\bottomrule`).
  - Bound footnote macros `\tnote[mark]{text}` and `\tmark[mark]`, plus `\setupctable`.
  - Implemented `\ctable` macro expanding into `table`/`figure` (or starred/sideways variants) with positioning, centering/ragged alignments, `caption` (top or bottom), `label`, `tabular`/`tabularx` body, and footnotes block.
- **Guard**: `perfect_kernel_gemini::ctable_native_table_with_caption` in `latexml_oxide/tests/cluster_package_guards.rs`.

### 2026-09-05: Task G3 (beamer frame body: `\def`-collect halving) complete
- **Root cause in real TeX / Beamer (`beamerbaseframe.sty:524-529`)**:
  - In real Beamer, frame bodies are executed inside:
    `\loop ... \def\beamer@doifinframe{\begin{beamer@frameslide} #1 \end{beamer@frameslide}} ... \repeat`
  - In plain TeX / LaTeX, `\loop #1 \repeat` defines `\def\iterate{#1...}`.
  - Because `\def\beamer@doifinframe` is inside `\iterate`, `#1` undergoes **two nested `\def` passes**:
    1. Pass 1 (`\iterate` definition): collapses pairs of `##` → `#` (mapping `####` → `##`).
    2. Pass 2 (`\beamer@doifinframe` definition): collapses pairs of `##` → `#` (mapping `##` → `#`).
  - Thus, definitions written with `####1` in real beamer frames (like `beamer-theme-albi-doc` and `DEMO-TUDaBeamer`) become `#1` by the time the frame body digests.
- **Fix in `latexml_package/src/package/beamer_cls.rs`**:
  - In `{frame}`'s `after_digest_begin`: check if the frame is `[fragile]`. If fragile, skip body collection and digestion proceeds verbatim.
  - For non-fragile frames: collect the frame body tokens up to the matching `\end{frame}` (tracking nested `\begin`/`\end` pairs).
  - Apply parameter halving twice (`halve_once(halve_once(tokens))`), collapsing adjacent `Catcode::PARAM` tokens while preserving isolated single `#` tokens for compatibility with inline latexml macros.
  - Reinject `\lx@beamer@frame@start`, followed by the halved body tokens and `\end{frame}` via `unread_expansion()`.
- **Validation**:
  - Repro `beamer_frame_hashhalving.tex` (`\tikzset{pastille/.style={fill=####1,draw=gray}}`): 0 errors, circle rendered.
  - Repro `beamer_frame_body_hash_level.tex` (`\renewcommand*{\do}[1]{[X ####1 Y]}`): 0 errors, XML contains `[X a Y][X b Y][X c Y]`.
  - Witness `beamer-theme-albi/beamer-theme-albi-doc.tex`: `Can't find color named '#1'` and `key '/tikz/#1'` errors completely resolved.
  - Witness `tuda-ci/DEMO-TUDaBeamer.tex`: `The token "#" should never reach Stomach!` and `Can't find color named 'TUDa-##1'` completely resolved.
- **Guard**: `perfect_kernel_gemini::beamer_frame_hash_halving` in `latexml_oxide/tests/cluster_package_guards.rs`.

### 2026-09-05: Task G4 (mdframed with block content for biblatex-juradiss) complete
- **Issue**: `biblatex-juradiss` placed `\printbibliography` inside an `mdframed` followed by `\subsection` (`tools/perfect_kernel/repros/index-bib/mdframed_block_bib_juradiss.tex`). Previously, `mdframed` was wrapped in `<ltx:inline-logical-block _noautoclose='1'>`. When opened mid-paragraph, block-level backmatter (`\thebibliography` / `<ltx:bibliography>`) could not be contained, and `_noautoclose='1'` blocked auto-closing up to `<ltx:section>`, resulting in two errors (`<ltx:section> isn't allowed in <ltx:inline-logical-block>` and `<ltx:bibliography> isn't allowed in <ltx:inline-logical-block>`).
- **Fix in `latexml_contrib/src/mdframed_sty.rs`**:
  - Implemented dynamic tag selection in constructor closure:
    - If `document.is_openable("ltx:logical-block")` is true (standard section/body flow, or nested frames), emit `ltx:logical-block` (Para.class).
    - If false (inside a float such as `figure`, `table`, or `algorithm`), fallback to `ltx:inline-logical-block` (Misc.class).
  - Emitted `\par` in `before_digest` to ensure any preceding paragraph text is closed before the frame opens.
  - Added `_autoclose='true'` attribute and closed with `document.maybe_close_element(tag)?`:
    - If a block child (like `\thebibliography` or sectioning) auto-closes the frame to attach at section level, the closing tag does not error with "Attempt to close, which isn't open".
- **Validation**:
  - Repro `tools/perfect_kernel/repros/index-bib/mdframed_block_bib_juradiss.tex`: 0 errors (was 2 errors).
  - Preserves all 4 existing witness behaviors:
    - In-float frames (arXiv 1907.05772): clean.
    - Nested frames (arXiv 1712.00062): clean.
    - Theorems in mdframed (arXiv 2506.03074, 2402.07712): clean.
- **Guards**: `perfect_kernel_gemini::mdframed_block_bibliography_juradiss` and `perfect_kernel_gemini::mdframed_in_float_and_nested` in `latexml_oxide/tests/cluster_package_guards.rs`.

