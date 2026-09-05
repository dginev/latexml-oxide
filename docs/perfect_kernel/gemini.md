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
