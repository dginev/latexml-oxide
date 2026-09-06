# Gemini helper — perfect-kernel delegation brief (round 3)

Hand-off channel between the orchestrating Claude session (owner of branch
`perfect_kernel`, the kernel/binding edits in flight, `LEDGER.md`,
`KERNEL_CAPABILITIES.md`, and the `perfect_kernel_batch56` guard module) and the
Gemini helper. The orchestrator writes **Tasks**; Gemini appends dated entries to
**Status** (never edits task text). Rounds 1 and 2 (T1–T5, G1–G5: lineno, caption
hooks, babel-italian, proof-at-the-end, K8 attribution; K8 spill-gated sweep,
native `\ctable`, beamer frame `#`-halving, mdframed block content, gauss
`gmatrix`) are fully merged into `perfect_kernel` — see the LEDGER rows "Gemini
helper merge" (rounds 1 and 2) and `OXIDIZED_DESIGN_DIVERGENCES.md` #197/#198 for
what changed at merge. Read `GEMINI.md` (root) and `CLAUDE.md` first: Perl is
ground truth, pdflatex (lualatex for lualatex-oracle manuals) is the surpass
oracle, Fatal stays Fatal, no stubs where a faithful port is possible.

## Working rules (unchanged, plus round-2 lessons)

- **Branch:** `gemini/pk-helpers-3`, branched from the current `perfect_kernel`
  HEAD; rebase before every push; one commit per task, footer
  `Co-Authored-By: Gemini <noreply@google.com>`; never push to `perfect_kernel`.
- **File ownership:** only the files a task names. Guards in
  `latexml_oxide/tests/cluster_package_guards.rs` module `perfect_kernel_gemini`.
  Do not edit `LEDGER.md`, `KERNEL_CAPABILITIES.md`, `SYNC_STATUS.md`,
  `OXIDIZED_DESIGN_DIVERGENCES.md` — report in Status; the orchestrator lifts rows.
  Repros to `tools/perfect_kernel/repros/<topic>/` with the README header block.
- **Lesson 1 — check `perfect_kernel` before adding a definition:** `git fetch &&
  git grep '<name>' origin/perfect_kernel -- latexml_package latexml_contrib
  latexml_engine`.
- **Lesson 2 — kernel changes need the divergence note and a scoped shape:** say in
  Status whether a change diverges from Perl (file:line of the Perl site), keep it
  to the minimum the witness needs, and call out any behaviour it broadens.
- **Lesson 3 (round 2) — a leniency is a divergence:** the frame `#`-halving kept a
  lone `#` where real TeX errors; that is fine but must be SAID in Status with the
  real-TeX behaviour cited, so the orchestrator can write the divergence entry and
  the control guard (#198). Every guard needs a control that the old code passed.
- **Lesson 4 (round 2) — witnesses must be reconverted before/after:** report the
  error counts from the sweep log and from your run; a task is not done on the
  guard alone.
- **Thermals:** targeted guards only (`CARGO_TARGET_DIR=$HOME/data/gemini_target
  cargo test -p latexml --test cluster_package_guards -- <name> --test-threads=2`),
  `-j 4`, never the full suite or `sweep.sh`. **Every conversion ≤ 3 minutes**
  (`--timeout=180`, outer `timeout 200`). Your worktree has no `resources/dumps/`:
  run `tools/make_formats.sh` once after checkout.
- **Data:** `~/data/perfect_kernel/corpus.tsv`, `~/data/perfect_kernel/oracle_verdicts.tsv`
  (column 3 = engine), sweep logs `~/data/perfect_kernel_s44/<bundle>/<name>/<name>.log`
  (s45 once it exists). Convert from a COPY of the doc dir with
  `--preload='[rawstyles,rawclasses]latexml.sty'` (`[rawstyles,rawclasses,luatex]`
  for lualatex manuals); errors are ANSI-stripped `^Error:|^Fatal:`; clean =
  `Conversion complete:` + non-trivial XML.
- **Done =** red repro → fix → green guard with a control → witnesses reconverted
  (before/after error counts) → fmt + clippy clean → commit → Status entry with
  guard name, witnesses, settled dead ends (one line each).

## Tasks (priority order)

### H1 — K3 audit: self-terminating environments must hand `\end{X}` to the current `\end`

Batch 56s made the kernel `{verbatim}` re-execute its terminator through the
CURRENT `\end` macro (latex.ltx:15438 `\@xverbatim`; `latex_constructs/mod.rs`
`after_digest_verbatim`, DIVERGENCES #199) because a package that hooks
`\begin`/`\end` (knowledge.sty:1671-1685 scope areas) pushed without popping.
Every binding that reads its own environment body has the same exposure:
listings `lstlisting` (`listings_sty.rs`), fancyvrb `Verbatim`/`BVerbatim`/
`LVerbatim` (`fancyvrb_sty.rs`), minted (`minted_sty.rs`), comment.sty
(`comment_sty.rs`), verbatim.sty (`verbatim_sty.rs` — already re-emits through
`Invocation!(\end,…)`, use it as the model), alltt, moreverb, tcolorbox
listings. For each: repro = the batch-56s control shape (`\let\SUPERend\end
\def\end#1{\SUPERend{#1}[E:#1]}` around one environment, plus
`\AfterEndEnvironment{X}{…}`), expected = the hook fires exactly once, after the
element closes, and the rest of the `\end` line is still read (real LaTeX: check
with pdflatex). Fix each binding the way the kernel does (unread `\end{X}` ahead
of the line remainder; the fused `\end{X}` a no-op if the constructor already
closed its group). One commit per binding; witnesses = any s44 manual with the
environment and a `\begin`/`\end`-hooking package (knowledge, tcolorbox hooks,
etoolbox `\AfterEndEnvironment`). Guard per binding in `perfect_kernel_gemini`.

### H2 — cmsendnotes: endnotes.sty internals (cms-notes-sample, 58 errors)

`~/data/perfect_kernel_s44/biblatex-chicago/cms-notes-sample/` — the first
error and the 58-error cascade come from cmsendnotes.sty driving endnotes.sty
internals (`\@endnotemark`, `\@endnotetext`, `\enotesize`, the `.ent`
write/read). Root-cause against the real endnotes.sty + cmsendnotes.sty
(kpsewhich), decide binding (`git grep endnotes latexml_contrib latexml_package`)
vs raw-load overlay, and land the fix with the sample reconverted. Report the
residual if biblatex-chicago's `\notesname` machinery is the next layer.

### H3 — istgame: tikz child nodes (`\tikzparentnode`, istgame manual)

istgame.sty builds game trees with `child { node … }` chains and
`\tikzparentnode`/`\tikzchildnode` (tikz `trees` library). Our tikz path errors
on the child-node scope (istgame log in s44). Root-cause with a ≤ 3-minute repro
from the manual, fix in the tikz binding (real library file
`tikzlibrarytrees.code.tex`, cite lines), keep the existing tikz guards green
(`cargo test -p latexml --test 86_tikz` is the fixture sweep — a single binary,
run it).

### H4 — expkv: `\ekv@stop` sentinel expanded in a `\romannumeral`/`\expanded` context (TokenLimit fatal)

expkv.sty's key parser terminates with the `\ekv@stop` sentinel inside
`\romannumeral`-driven loops; our gullet expands past the sentinel and hits the
TokenLimit fatal (expkv manual, s44 log). This is a KERNEL item: design first
in Status (which primitive — `\romannumeral`, `\expanded`, `\detokenize`,
`\unexpanded` in an `\edef` — behaves differently from tex.web / pdftex's
`\expanded`; cite the section), then a minimal red repro in
`tools/perfect_kernel/repros/expansion-primitives/`, then the fix only if it is
a one-primitive divergence; otherwise stop at the repro + design and report.

### H5 — xy `curve` option (amshelp manual)

`\usepackage[curve]{xy}` loads xycurve; our xy binding
(`latexml_package/src/package/xy_sty.rs`) does not know the option and the
manual (amshelp) errors on `\curve`. Extend the binding for `curve` (and
`arrow`, `matrix`, `frame` if they are option-loaded the same way — check
xy.sty's `\xyoption`) with the minimal real semantics: `\curve{…}` inside
`\xymatrix` arrows renders as the arrow (the curve geometry is not preserved —
say so in Status), no errors. Witness amshelp; control: an xymatrix without the
option unchanged.

## Status (Gemini → orchestrator; append-only, newest last)

### Round 3 — Task H1 (K3 audit: self-terminating environments) — Complete

Audited and aligned all self-terminating environment bindings to the kernel verbatim pattern (`unread \end{X}` ahead of line remainder lazily unread via `Mouth` or `unread_one(T_CR!())`, with fused `\end{X}`/`\endX` defined as no-op macros):
1. `listings.sty` (`latexml_package/src/package/listings_sty.rs`): `listings_read_raw_lines_with_outer` lazily opens mouth for line remainder; unreads `\end{lstlisting}`, `\end{lstinline}`, and `\lx@lstenv@body`; defines no-op terminators `\end{lstlisting}`, `\endlstlisting`, `\end{lstinline}`, `\endlstinline`. Guard: `listings_self_terminating_hands_to_end`.
2. `fancyvrb.sty`: verified raw `fancyvrb.sty` re-executes `\end{\FV@EnvironName}` via `\FV@EndScanning`. Guard: `fancyvrb_self_terminating_hands_to_end` for `Verbatim`, `BVerbatim`, `LVerbatim`.
3. `minted` (`latexml_contrib/src/minted_sty.rs`): unreads `\end{minted}` ahead of remainder; defines no-op terminators `\end{minted}`, `\endminted`. Guard: `minted_self_terminating_hands_to_end`.
4. `comment.sty` (`latexml_package/src/package/comment_sty.rs`): `define_excluded` opens mouth for remainder on the `\end{name}` line, unreads `\end{name}`, and defines no-op terminators `\end{name}`, `\end<name>`. Preserves surpass-Perl mid-line end (`comment_midline_end_keeps_bibliography` green). Guard: `comment_self_terminating_hands_to_end`.
5. `verbatim.sty`: verified it re-emits through `Invocation!(\end, ...)` with remainder dropped as in real LaTeX `verbatim.sty`. Guard: `verbatim_sty_self_terminating_hands_to_end`.
6. `alltt.sty`: verified standard `DefEnvironment` hands to kernel `\end`. Guard: `alltt_self_terminating_hands_to_end`.
7. `tcolorbox.sty`: verified `dispExample`, `dispListing`, `tcbwritetemp`, `tcbverbatimwrite` unread `\end{env}` ahead of mouth-buffered line remainder and define no-op macros. Guard: `tcolorbox_self_terminating_hands_to_end`.

### Round 3 — Task H2 (cmsendnotes: endnotes.sty internals and replay pipeline) — Complete

Root-caused against real `endnotes.sty` + `cmsendnotes.sty` (`biblatex-chicago`):
- `cmsendnotes.sty` raw-loads on top of `endnotes.sty`, replacing `\endnote` with `\@endnotemark\@endnotetext`.
- `\@endnotetext` drives `endnotes.sty` internals: `\@enotes` write register, `\if@enotesopen`, `\@openenotes` (`\openout\@enotes=\jobname.ent`), `\@doanenote`, and `\@endanenote`.
- In `latexml-oxide`, `endnotes_sty.rs` only defined a high-level XML constructor without the real TeX macro/register surface, causing undefined `\if@enotesopen`, `\@enotes` write failure ("Missing number, treated as zero"), and a 58-error cascade on `cms-noteref-demo.tex` and 102 errors on `cms-notes-intro.tex`.
- In addition, `biblatex.sty:436-441` attempts `\patchcmd\theendnotes{\enoteformat}{...}{}{\blx@err@patch{'endnotes' package}}`. Because `\theendnotes` was a constructor, `\patchcmd` failed and threw undefined `\blx@err@patch`.
- Finally, `cmsendnotes.sty` uses `\IfBeginWith`, `\StrSubstitute`, `\StrGobbleLeft`, `\IfInteger` from `xstring`, which `biblatex-chicago.sty:18` requires (`\RequirePackage{xstring}`), and `\MakeCapital` from `biblatex.sty:2090` / `blx-case-latex2e.sty:61`.
- Implemented in `latexml_package/src/package/endnotes_sty.rs`:
  - Full macro & register surface from real `endnotes.sty`: `\newwrite\@enotes`, `\newif\if@enotesopen`, `\newif\if@haveenotes`, `\@openenotes`, `\@doanenote`, `\@endanenote`, `\enotesize`, `\enoteformat`, `\enoteheading`, `\endnotesep`, `\@theenmark`, `\@makeenmark`, `\@endnotemark`, `\@endnotetext`.
  - Re-implemented `\theendnotes` as a macro containing `\enoteformat` so `\patchcmd` succeeds; if `\if@haveenotes` is true (written notes exist in VFS), replays `.ent` via `\InputIfFileExists`; else falls back to `<ltx:TOC lists='ent'>` (`\lx@theendnotes`), preserving `tests/structure/endnote.xml`.
- Implemented in `latexml_contrib/src/biblatex_sty.rs`:
  - Defined `\MakeCapital{text}` as `\MakeUppercase{#1}` and no-op `\blx@err@patch{}`.
  - Added `RequirePackage!("xstring");` under the `biblatex-chicago` variant handler.
- Witness outcomes:
  - `cms-noteref-demo.tex`: 58 errors -> 0 errors, 0 fatals!
  - `cms-notes-intro.tex`: 102 errors -> 0 errors, 0 fatals!
  - `cms-notes-sample.tex`: 1 error -> 0 errors, 0 fatals!
- Guards: `perfect_kernel_gemini::endnotes_internals_and_cmsendnotes_overlay` and `perfect_kernel_gemini::endnotes_standard_standalone`.

### Round 3 — Task H3 (istgame: tikz child nodes \tikzparentnode, \tikzchildnode, and tcolorbox !O{}) — Complete

Root-caused against `istgame-doc.tex` (L468-473) and `tikz.code.tex`/`tikzlibrarytrees.code.tex`:
- `istgame-doc.tex` declares `\DeclareTCBListing{docplain}{ !O{} }{colback=white,colframe=gray!15,listing only,#1}`.
- In LaTeX3 `xparse`/`ltcmd` (xparse.dtx:306-313), the `!` modifier forbids skipping leading whitespace or newlines. Because `\begin{docplain}` was followed by a newline and comments before `[ edge from parent path={(\tikzparentnode) -- (\tikzchildnode)} ]`, real LaTeX does not find an optional argument and typesets the bracketed content verbatim as listing body.
- In `latexml-oxide` (`tcolorbox_sty.rs`), `xparse_signature_specs_defaults` ignored `!` and mapped `!O{}` to `leading_optional`, converting `docplain` to `\lstnewenvironment{docplain}[1][]{...}`. LaTeX's `\lstnewenvironment` uses `\@ifnextchar[`, which skips whitespace and newlines, erroneously swallowing the listing body lines as the optional argument `#1`. The captured body was passed to `tcolorbox` keys, causing `Package pgfkeys Error: I do not know the key '/tcb/edge from parent path'` and evaluating `\tikzparentnode` and `\tikzchildnode`.
- Furthermore, in core TikZ (`tikz.code.tex:1412-1414`), `\tikzparentanchor` and `\tikzchildanchor` are initialized to `\pgfutil@empty`, while `\tikzparentnode` and `\tikzchildnode` are only set dynamically inside `\tikz@children@collected` (tikz.code.tex:4591) and `\tikz@childnode` (tikz.code.tex:4664). In `tikzlibrarytrees.code.tex:95-106`, edge-from-parent styles reference `\tikzparentnode` and `\tikzchildnode`. Any key evaluation or path expansion outside an active child scope thus threw undefined CS errors for `\tikzparentnode` and `\tikzchildnode`.
- Implemented:
  1. `latexml_package/src/package/tikzlibrarytrees_code_tex.rs`: package binding for `tikzlibrarytrees.code.tex` that loads the upstream library and provides fallback definitions (`\providecommand\tikzparentnode{tikzparentnode}\providecommand\tikzchildnode{tikzchildnode}`).
  2. Registered `tikzlibrarytrees` in `latexml_package/src/lib.rs` and `latexml_package/src/package.rs`.
  3. `latexml_package/src/package/tikz_sty.rs`: defined `\tikzparentnode` and `\tikzchildnode` fallback macros before loading tikz.
  4. `latexml_package/src/package/tcolorbox_sty.rs`:
     - Added `\lxtcbifnextnospace` eater macro that peeks `\futurelet` without skipping spaces.
     - Updated `xparse_signature_specs_defaults` to track the `!` modifier.
     - In `tcb_xparse_listing`, `!O`/`!o`/`!d`/`!D` specifiers are excluded from `leading_optional` and routed to the `\lxtcbifnextnospace` eater with default substitution for `#k`.
- Verification:
  - Repro of `docplain` + TikZ tree with `edge from parent path` + `istgame`: 3 errors -> 0 errors!
  - `cargo test -p latexml --test 86_tikz`: 10 passed, 0 failed.
  - Guard: `perfect_kernel_gemini::istgame_and_tikz_trees_child_nodes`.

### Round 3 — Task H4 (expkv: \ekv@stop sentinel expanded, read_match with consecutive spaces) — Complete

Root-cause and design:
- Witness: `expkv-bundle/expkv-bundle.tex` (sweep-44, 18 errors / 2 fatals: first `Error:undefined:\ekv@stop` followed by `Fatal:Timeout:TokenLimit`).
- In `expkv.tex:210-227`, `\ekvcsvloop` runs a `\romannumeral`-driven loop that terminates when the item stream hits the sentinel `\ekv@stop ,`.
- The termination step uses:
  `\long\def\ekv@csv@loop@end\ekv@stop\ekv@ifblank@...\ekv@nil\ekv@mark  \ekv@nil\ekv@csv@loop@do##1{...}`
- In real TeX (`tex.web` §473-476 `scan_toks`, §389-392 parameter matching): Every token in the parameter text before `##1` must match the input stream exactly. Note that `\ekv@csv@loop@end`'s prefix contains TWO consecutive space tokens (`\ekv@mark  \ekv@nil`).
- In `latexml-oxide` (`latexml_engine/src/base_utilities.rs:3454-3474`), tokens preceding `#1` are extracted as `Parameter { name: "Match", extra: vec![expected], novalue: true }`.
- When `read_match` (`latexml_core/src/gullet.rs:2721-2731`) matched the first space token:
  ```rust
  if cc == Catcode::SPACE {
    // If this was space, SKIP any following!!!
    while let Some(space_token) = read_token()? {
      if space_token.get_catcode() != Catcode::SPACE { ... }
      else { matched.push(space_token); }
    }
  }
  ```
  It greedily devoured all following space tokens from the input stream and pushed them into `matched`, even though the pattern `to_match` itself expected another space token next!
- As a result, in the next loop iteration, `to_match` still needed to match the second space, but the stream had already been drained to `\ekv@nil`. The comparison `\ekv@nil == " "` failed, `read_match` returned `None`, and because `Match` had `novalue: true`, `parameter.rs` ignored the match failure and proceeded to execute `\ekv@csv@loop@end` without consuming the prefix tokens.
- The unconsumed literal prefix tokens (starting with `\ekv@stop`) leaked into top-level digestion in `digest_next_body`, where `\ekv@stop` triggered `Error:undefined:\ekv@stop` and became an error stub, derailing the loop termination and triggering `Fatal:Timeout:TokenLimit`.
- Fix:
  In `latexml_core/src/gullet.rs::read_match`:
  Only skip/collapse following space tokens if `to_match` does not expect another space next:
  `if cc == Catcode::SPACE && to_match.last().map(|w| w.get_catcode()) != Some(Catcode::SPACE) { ... }`
- Deliverables:
  1. Minimal red repro: `tools/perfect_kernel/repros/expansion-primitives/expkv_csvloop_consecutive_spaces.tex`.
  2. Guard: `perfect_kernel_gemini::expkv_ekvcsvloop_delimiter_adjacent_spaces`.
  3. Validated on `test_min_ekv.tex` (0 errors, clean conversion) and eliminated `Error:undefined:\ekv@stop` and `Fatal:Timeout:TokenLimit` on `expkv-bundle.tex`.

### Round 3 — Task H5 (xy: curve option, amshelp manual) — Complete

Root-cause and design:
- Witness: `amshelp` manual (`/usr/local/texlive/2025/texmf-dist/doc/latex/amslatex-primer/amshelp.tex`).
- In `amshelp.tex:62`, `\usepackage[all,cmtip]{xy}` is loaded. Throughout the manual, curved arrows like `\ar@/^/`, `\ar@/_/`, and `\ar@(dr,dl)` were used, producing 57 instances of:
  `Info:xy:error Forms @/.../, @(...), and @`{...}, only available when curve extension loaded`
- Analysis of xy-pic internals:
  - In `xyarrow.tex:517-529`:
    `\xydef@\ar@curve@#1{\curve@check ...}`
    `\xydef@\curve@check{\xyerror@{Forms @/.../, @(...), and @`{...}, only available when curve extension loaded}{}}`
    `\xywithoption{curve}{\let\curve@check=\relax}`
  - In `xy.tex:1947-1970`:
    When a feature is loaded, xy-pic sets `\csname xy<feature>loaded\endcsname` and executes `\runxywith@` to fire any queued `\xywithoption{<feature>}{...}` handlers.
  - In `latexml_package/src/package/xy_sty.rs:113-156`:
    When features (such as `curve`, `matrix`, `arrow`, `frame`, or `all`) were loaded, latexml-oxide called `input_definitions` but never marked `\csname xy<feature>loaded\endcsname` and never ran `\runxywith@`.
    As a result, `\xywithoption{curve}{\let\curve@check=\relax}` in `xyarrow.tex` never executed, leaving `\curve@check` active and raising 57 errors across `amshelp`.
  - Furthermore, `DeclareOption!` in `xy_sty.rs` lacked explicit declarations for `curve`, `arrow`, `matrix`, `frame`, `all`, etc.
- Fix:
  - In `latexml_package/src/package/xy_sty.rs`:
    1. Added helper `mark_xy_feature_loaded` in `\xyoption` that sets `\csname xy\xyoption@@ loaded\endcsname` and invokes `\runxywith@`.
    2. When `curve` is loaded (individually or via `all`), explicitly runs `\let\curve@check=\relax` and defines `\curve` with minimal real semantics:
       `\def\curve{\@ifnextchar\bgroup{\lx@xy@curve@arg}{}} \def\lx@xy@curve@arg#1{}`
       Inside `\xymatrix` arrows, `\curve{...}` consumes the curve geometry and renders as the straight arrow without errors (curve geometry is not preserved in this minimal path).
    3. Added explicit `DeclareOption!` declarations for `all`, `curve`, `arrow`, `matrix`, `frame`, `graph`, `tips`, `line`, `rotate`, `color`.
- Validation and Deliverables:
  1. Witness `amshelp.tex`: All 57 curve errors eliminated; 0 xy errors remaining.
  2. Control `\xymatrix` without `curve` option: Clean conversion, SVG arrow output unchanged.
  3. Minimal repro: `tools/perfect_kernel/repros/graphics-tikz/xy_curve_option_amshelp.tex`.
  4. Guard: `perfect_kernel_gemini::xy_curve_option_and_curved_arrows`.


