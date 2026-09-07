# Gemini helper — perfect-kernel delegation brief (round 8)

You are a helper on branch `perfect_kernel` of `~/git/latexml-oxide` (the perfect-
kernel program: `docs/PERFECT_KERNEL.md`). Work on a branch `gemini/pk-helpers-8`
cut from the current `perfect_kernel` tip (which contains batch 56ab AND your
round-7 merge, commit `66a7bf6144`, plus the orchestrator's fixup commit right after
it), push it, and append to the **Status** section at the end of this file (never
edit task text). Rounds 1–7 are merged. Round-7 review outcome, for your calibration:
the algpseudocodex entry is DIVERGENCES **#214** (numbers continue from the file's
last entry on `perfect_kernel`, never from `main`); the `DefKeyVal!` registrations
shadowed your raw `\KV@…` handlers (every explicit option was a no-op — handlers now
come after the registrations); your M2 (replay any dropped `\maketitle` body) was REVERTED —
the full suite showed resphilosophica.cls:331's body running against the amsart
binding, which lacks `\@setcopyright`/`\andify`/`\@maketitle@hook` (a generic
replay is the unseen-class backfire the orchestrator warns about; the uspatent
binding's explicit unlock is back); a hard-coded page count of 10 became l3's own fallback of 1; the xcolor guard
asserted a driver literal no code produces (M3 had already landed in batch 56aa —
always `git grep` `perfect_kernel` before re-implementing); three guards had no
structural assertion; a `\ExplSyntaxOn … \ExplSyntaxOff` block at latexml.sty top level
(`latexml_sty/mod.rs`) raw-loaded expl3 while the format was still loading and
pinned the dvips backend before the document could choose pdftex — top-level code
there names expl3 macros through `\csname`, never `\ExplSyntaxOn` (rewritten at merge). The istgame seam you reported (`\LoadClassWithOptions` dropping
the caller's options) is fixed in the kernel — an engine-seam report with a repro is
exactly the deliverable we want. The Perl source under `LaTeXML/` is ground truth,
pdflatex (lualatex for lualatex-oracle manuals) is the surpass oracle.

## Working rules (unchanged, plus round-2 lessons)

- **No pull requests.** All work lands on `perfect_kernel` through your
  `gemini/pk-helpers-N` branches; the single PR to `main` is opened by the user
  when the branch is ready. Never open a PR (PR #798 was closed for this reason).
- **This file lists only OPEN tasks.** At each merge the orchestrator lifts your
  Status entries into `LEDGER.md`/`KERNEL_CAPABILITIES.md` and deletes them here
  together with the solved task text; a task that is still open is carried over
  under a new number. Append your Status for THIS round below; nothing older
  belongs here.

- **Branch:** `gemini/pk-helpers-8`, branched from the current `perfect_kernel`
  HEAD (the round-7 merge + its fixup commit); rebase before every push; one commit per task, footer
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
- **Lesson 5 (round 3) — run the goldens outside your module** (`cargo test -p
  latexml --test 00_tokenize`, `--test 10_expansion`, the `tests/structure`
  goldens, `06_cluster_bibliography`); `git grep -l '<env>' latexml_oxide/tests`.
- **Lesson 6 (round 3) — a kernel change is a surpass unless Perl agrees:** run
  the witness through same-host Perl (`~/perl5/bin/latexml`) first.
- **Lesson 7 (round 3/4) — numbers from the sweep log, and only for docs the fix
  touches:** quote `grep -cE '^(Error|Fatal):'` on `~/data/perfect_kernel_s46/…`,
  and confirm the doc actually loads the binding you changed (`grep 'Loading
  <name>' <log>`) before claiming it as a witness (pxcjkcat, round 4).
- **Lesson 8 (round 5) — an engine seam report is a deliverable:** J3's bisect + red repro
  went straight into a kernel fix; keep doing that (file:line of the seam, the commit that
  introduced it, a ≤30-line repro, and the proposed rule).
- **Scope rule (rounds 4–6):** bindings only — no edits under `latexml_core/` or
  `latexml_engine/`; stop at the design + red repro and report it in Status.
- **Thermals:** targeted guards only (`CARGO_TARGET_DIR=$HOME/data/gemini_target
  cargo test -p latexml --test cluster_package_guards -- <name> --test-threads=2`),
  `-j 4`, never the full suite or `sweep.sh`. **Every conversion ≤ 3 minutes**
  (`--timeout=180`, outer `timeout 200`). Your worktree has no `resources/dumps/`:
  run `tools/make_formats.sh` once after checkout.
- **Data:** `~/data/perfect_kernel/corpus.tsv`, `~/data/perfect_kernel/oracle_verdicts.tsv`
  (column 3 = engine), sweep logs `~/data/perfect_kernel_s51/<bundle>/<name>/<name>.log`
  (s53 = batch 56ac, s54 = 56ad are the current sweeps; s52 = 56ab + your round 7). Convert from a COPY of the doc dir with
  `--preload='[rawstyles,rawclasses]latexml.sty'` (`[rawstyles,rawclasses,luatex]`
  for lualatex manuals); errors are ANSI-stripped `^Error:|^Fatal:`; clean =
  `Conversion complete:` + non-trivial XML.
- **Done =** red repro → fix → green guard with a control → witnesses reconverted
  (before/after error counts) → fmt + clippy clean → commit → Status entry with
  guard name, witnesses, settled dead ends (one line each).

## Tasks (priority order)

Round 9. Rounds 1–8 are lifted into `LEDGER.md` (their Status text is deleted here by rule).

### N1 — tikzpingus: bisect the shading regeneration failure (root-cause + fix)

tikzpingus/tikzpingus-doc (oracle lualatex, renders) dies with `\lxSVG@sh@defs`/
`\lxSVG@pos`/`\lxSVG@sh` undefined → `Timeout:PushbackLimit`. The trio is what
`\@pgfshading<name>!` defines when invoked (Perl pgfsys-latexml.def.ltxml:672-724,
Rust `pgfsys_latexml_def.rs:1616-1760`); it is undefined iff a use-time regeneration
(`\pgfuseshading` → `\pgfshadepath`, pgfcoreshade.code.tex:769-810) failed to reinstall
`\@pgfshading<xname>!`, so `\pgfsys@shadinginsidepgfpicture{\relax}` runs. Fingerprint:
three "Illegal unit of measure (pt inserted) at Anonymous String" right before. A
root-causer's 12 isolation probes stayed clean (`~/data/pk_agents/w22/pgf-pair/NOTES.md`,
`probe_*.tex`); the trigger needs the full manual's showcase+tcolorbox+tikzducks
context. Task: bisect tikzpingus-doc.tex (delete halves of the body until the first
error survives in ≤ 40 lines), then decide between the two candidates — (1) the global
`\@pgfshading<xname>!` lost across `\pgfmath@smuggleone\csname\pgf@shadingxname\endcsname`
(:794-802) on our opaque `DefPrimitiveI`; (2) the malformed-spec dimension math in
`\lxSVG@sh@create`/`@intervals`/`@stop` derailing the stream before `\lxSVG@sh@defstripes`
installs the shading — with file:line evidence, a red repro, the faithful fix (binding
or, if (1), a note for the orchestrator: core edits are theirs), and a guard asserting
0 errors + an `<svg:linearGradient>`/`radialGradient` with ≥ 2 `<svg:stop>`.

### N2 — pgfplots `scatter` markers leak one boxing `{` frame each (token trace)

ualberta/ualberta (oracle lualatex exit 1) — `\endgroup Attempt to close non-boxing
group` then a PushbackLimit runaway. Inline repro (no data file, ≥ 32 errors):
`~/data/pk_agents/w22/color-group/repro_b.tex` (`\addplot+[scatter,mark=*] coordinates
{(1,1)(2,4)(3,9)}`); non-scatter plots are clean. `LXML_TRACE_FRAMES=1` shows one boxing
`{` frame per marker pushed in restricted_horizontal and never popped; the marker body is
`\pgfplots@scatter@plot@mark`, an xdef'd `\noexpand\begingroup … \noexpand\endgroup`
deferred by `\aftergroup` (pgfplots.markers.code.tex:178-214). Task: a token-level trace
of ONE marker (which `}` is lost — the marker `\hbox`'s closer, or an `\aftergroup` fired
into the wrong frame; tex.web §1063-1068 for the box closer, §280 for `\aftergroup`), the
mechanism named at the general level (this is an engine `\aftergroup`-vs-box-reader
question — do NOT patch pgfplots), a ≤ 25-line red repro without pgfplots if possible, and
a fix PLAN for the orchestrator (core edits are theirs) with the guard
`pgfplots_scatter_marker_group_balance` (0 errors + one `<svg:g>` per marker). Note also
that the runaway after the first error is a robustness gap (error recovery re-unreads an
unclosable `\endgroup`) worth a separate cap — name where.

### N3 — forest: a real binding (the user's standing side goal, DIFFICULT_CASES §D10)

`latexml_contrib/src/forest_sty.rs` discards the whole `{forest}` body into an
`<ltx:ERROR>` (mirrors ar5iv's `discard_env_body`): every tree in the 12 forest manuals
(3 oracle-clean) is lost while the docs read "0 errors". A raw load needs tikz +
`pgfopts` + `elocalloc` + `environ` + `xparse` + `inlinedef` + forest's own bracket
parser; the last two are the blockers. Task, direct implementation: write the forest
bracket grammar as a Rust reader (forest.sty / forest-lib-*.sty: `[root [child][child]]`
with node options `[label, key=value, …]`, `
ode` text, nested brackets, comments) and
emit the tree as nested `ltx:para`/`ltx:enumerate`-style structure — a semantic tree
(each node a titled item with its children) — keeping node TEXT and options as attributes;
no tikz drawing (out of scope for now, document what would be needed). Guard: a 3-level
tree → 0 errors, every node label present, nesting depth reflected; the 3 oracle-clean
manuals re-converted with their trees present. Cite forest.sty line numbers for the
grammar you implement and list unsupported syntax explicitly.

### N4 — an `animate` binding: one representative frame

tikz-among-us/tikz-among-us reaches the 4.6 GB memory fuse in 13 s because
`egin{animateinline}…\multiframe{180}{rt=0+1}{<tikzpicture>}` (animate.sty:2369-2394,
a bounded `\whiledo`) materializes 180 SVG frames in the document tree (~39 MB each);
pdflatex ships each frame as a Form XObject. No Perl binding either. Task: a
`latexml_contrib/src/animate_sty.rs` that runs the `\multiframe` body ONCE (the first
frame; expose the frame count as an attribute on a wrapper block) for both `animateinline`
and `nimategraphics` (which selects one file of a sequence), keeping every other command
of the package (`
ewframe`, `\multiframe`, timeline options) argument-consuming. Repro:
`~/data/pk_agents/w22/among-us/repro.tex` (`--max-memory=1536`). Guard
`animate_multiframe_single_frame`: 0 errors, `count(//svg:svg)=1`, the frame-count
attribute present.

### N5 — chemnum: sequential compound numbers instead of blank labels

`latexml_contrib/src/chemnum_sty.rs` no-ops `\cmpd`/`\cmpdinit`/`efcmpd`… so compound
labels and references are blank (11 manuals, 1 oracle-clean; body text kept). Raw load is
blocked (expl3/l3keys/translations/chemgreek). Task: implement the numbering model of
chemnum.sty (`\cmpd{label}` assigns the next number on first use, `efcmpd` prints it,
sub-compound `label.sub` → `1a`, lists `\cmpd{a,b}` → `1, 2`, the `\cmpdinit` declare
form; cite the .sty lines), emitting numbers as text with an `ltx:text class="ltx_cmpd"`
wrapper and an `xml:id`/`idref` pair so references link. Guard: first use = 1, second
label = 2, `efcmpd` of the first = 1, sub-compound = 1a; 0 errors.

## Status (Gemini → orchestrator; append-only, newest last; round 9 only)
