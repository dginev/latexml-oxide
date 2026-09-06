# Gemini helper — perfect-kernel delegation brief (round 4)

Hand-off channel between the orchestrating Claude session (owner of branch
`perfect_kernel`, the kernel/binding edits in flight, `LEDGER.md`,
`KERNEL_CAPABILITIES.md`, and the `perfect_kernel_batch56` guard module) and the
Gemini helper. The orchestrator writes **Tasks**; Gemini appends dated entries to
**Status** (never edits task text). Rounds 1–3 (T1–T5, G1–G5, H1–H5: lineno, caption
hooks, babel-italian, proof-at-the-end, K8 attribution; K8 spill-gated sweep,
native `\ctable`, beamer frame `#`-halving, mdframed block content, gauss
`gmatrix`; the K3 self-terminating audit, endnotes, tikz trees, the `read_match`
delimiter fix, xy curve) are fully merged into `perfect_kernel` — see the LEDGER rows "Gemini
helper merge" (rounds 1–3) and `OXIDIZED_DESIGN_DIVERGENCES.md` #197/#198/#200 for
what changed at merge. Read `GEMINI.md` (root) and `CLAUDE.md` first: Perl is
ground truth, pdflatex (lualatex for lualatex-oracle manuals) is the surpass
oracle, Fatal stays Fatal, no stubs where a faithful port is possible.

## Working rules (unchanged, plus round-2 lessons)

- **Branch:** `gemini/pk-helpers-4`, branched from the current `perfect_kernel`
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
- **Lesson 5 (round 3) — run the goldens outside your module:** a binding change
  to an environment reader must be run against the fixture binaries that cover it
  (`cargo test -p latexml --test 00_tokenize`, `--test 10_expansion`, the
  `tests/structure` goldens, `06_cluster_bibliography`) — the comment.sty change
  broke `tests/tokenize/comment.xml` and was trimmed at merge. `git grep -l
  '<env>' latexml_oxide/tests` lists them.
- **Lesson 6 (round 3) — a kernel change is a surpass unless Perl agrees:** run the
  witness through same-host Perl (`~/perl5/bin/latexml --preload='[rawstyles,rawclasses]latexml.sty'`)
  before calling a gullet/stomach change a "fix"; if Perl fails the same way it
  is a divergence and needs the entry (the `read_match` change was one).
- **Lesson 7 (round 3) — get the numbers from the sweep log, not memory:** the
  expkv-bundle count was reported as 18/2; the s44 and s45 logs say 2 errors + 1
  fatal. Quote the `grep -cE '^(Error|Fatal):'` result.
- **Scope rule (round 4):** bindings only — no edits under `latexml_core/` or
  `latexml_engine/` this round; if a root needs one, stop at the design + red
  repro and report it in Status for the orchestrator.
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

### I1 — cjk-ko: bind CJK's UTF8 active-byte MEANINGS (cjk-ko-doc, 101 errors + fatal)

`latexml_contrib/src/cjk_sty.rs:35` makes `\CJK@envStart` a no-op, so cjkutf8-ko.sty:150-154's
"protect utf8 octets" loop (`\protected\edef~{\unexpanded\expandafter{~}}` over
0x80–0xF4) expands inputenc's `\@inpenc@undefined` 117 times (`Keyboard character
used is undefined in inputencoding utf8`; SHARED with Perl, pdflatex clean). Port
`\CJK@envStart` faithfully for the UTF8 encoding: define `\CJK@@@`,
`\CJK@namedef`/`\CJK@nameppdef`/`\CJK@namepppdef`/`\CJK@nameppppdef` (+
`\CJK@X…`, CJK.sty:925-1010) and run the `UTF8.bdg` binding over 0x80..0xF4
(namedef 80–BF, nameppdef C0–DF, namepppdef E0–EF, nameppppdef F0–F4). Bind the
active-byte MEANINGS only — never `\CJK@makeActive`'s catcode change (bytes
0x80–0xFE are Unicode code points here; `café résumé — 한글` must stay 0 errors
and render). Repro `~/data/pk_agents/w17/kotex/repros/cjk_envstart_protect_utf8_octets.tex`
(RED: 102). Guard: repro → 0 errors + `소개` in a `<ltx:p>`; control: the
accented-Latin line. Witness cjk-ko-doc (report its next error class); check the
~18 ctex/CJK manuals in s45 that load cjk_sty.rs for regressions.

### I2 — montex: fontenc RELOAD must input the encodings not seen on the first load (14 symbols)

`ctib.sty:61` loads fontenc with `LCT,T1`; `mls.sty:413` reloads it with
`LGR,LMS,LMO,LMA,LMC,T1`. Our reload path reports the option clash and never
inputs `lmcenc.def`/`lmsenc.def`…, so `\MyTogrog` (lmcenc.def:35) and 13 more are
undefined (montex 14 errors; pdflatex clean — real fontenc.sty processes every
load's options). Repros `tools/perfect_kernel/repros/singletons/repro_montex_lmc_{reload,control}.tex`
(control = mls first, GREEN today). Fix in `latexml_package/src/package/fontenc_sty.rs`
(and the package-reload seam only if the binding cannot see the second option
list — then STOP and report the seam). Guard: reload repro → `\MyTogrog` renders,
0 errors; control unchanged.

### I3 — oup-authoring-template: the second layer (22 → 21 today)

After `\ORCID` (batch 56u) the class binding `latexml_contrib/src/oup_authoring_template_cls.rs`
still lacks booktabs (`\toprule`/`\midrule`/`\botrule`), the `algorithm`/
`algorithmic` environments (`\State`/`\Require`/`\While`), `{unlist}`, and the
`\caption` outside a float. Root-cause each against the real cls (kpsewhich
oup-authoring-template.cls), add the `RequirePackage!`s and definitions, witness
oup-authoring-template.tex (report before/after).

### I4 — heria: `<ltx:para>` inside `<ltx:block>` (heria-proposal, 4 errors)

Remaining after batch 56u: `malformed:ltx:para <ltx:para> isn't allowed in
<ltx:block>` ×2 and caption/toccaption in a block. Find the construct in
heria.cls (a framed/box environment holding paragraphs and a captioned float);
the kernel now floats un-containable content out of `insert_block`
(`latexml_engine/src/base_utilities.rs`, batch 56u) — if the right fix is there,
report the design, do not edit the engine.

### I5 — srdp-mathematik: `srdp-tables.sty` is a vendored tabu (8 errors + fatal)

`srdp-tables.sty` is a verbatim copy of tabu.sty; it is raw-loaded because our
`tabu_sty.rs` binding is keyed on the name `tabu`, and raw tabu needs array's
`\NC@list`/`\NC@do` token machinery our `array_sty.rs` omits. Detect the vendored
copy (compare `\ProvidesPackage` line or a `\tabu@` sentinel) and route it to the
tabu binding; guard with a two-line `\usepackage{srdp-tables}` + `\begin{tabu}`
repro; witness srdp-mathematik.

## Status (Gemini → orchestrator; append-only, newest last)
