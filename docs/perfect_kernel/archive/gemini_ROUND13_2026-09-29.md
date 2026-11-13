# Gemini helper — perfect-kernel delegation brief (between rounds)

You are a helper on branch `perfect_kernel` of `~/git/latexml-oxide` (the perfect-kernel
program: `docs/PERFECT_KERNEL.md`, whose **Roadmap** section is the ranked plan). Round 13's nine
tasks landed (merge f5a98f95f6, review fixes after it; LEDGER "Gemini round 13"); **there are no open
tasks**. The orchestrator writes round 14's tasks here, re-verified RED on the tip, before the next
round starts; until then, do nothing. The Perl source under `LaTeXML/` is ground truth; pdflatex
(lualatex for lualatex-oracle manuals) is the surpass oracle.

## Working rules

- **No pull requests.** All work lands on `perfect_kernel` through your
  `gemini/pk-helpers-N` branches; the single PR to `main` is opened by the user.
  Never open a PR.
- **This file lists only OPEN tasks.** At each merge the orchestrator lifts your
  Status entries into `LEDGER.md`/`KERNEL_CAPABILITIES.md` and deletes them here
  together with the solved task text; a task that is still open is carried over
  under a new number. Append your Status for THIS round below; nothing older
  belongs here.
- **Branch:** `gemini/pk-helpers-<round>` from `origin/perfect_kernel`; rebase before every push; one
  commit per task, footer `Co-Authored-By: Gemini <noreply@google.com>`; never push to
  `perfect_kernel`.
- **Scope:** bindings (`latexml_package/`, `latexml_contrib/`) and the files a task names. The
  orchestrator is working in `latexml_math_parser/` (the math grammar and its semantics) and
  `latexml_oxide/tests/parse/`; do not edit them. Do not edit `LEDGER.md`,
  `KERNEL_CAPABILITIES.md`, `SYNC_STATUS.md`, `OXIDIZED_DESIGN_DIVERGENCES.md` — report in Status;
  the orchestrator lifts rows.
- **Guards** go in `latexml_oxide/tests/cluster_package_guards/perfect_kernel_gemini.rs` (module
  `perfect_kernel_gemini`). Every guard asserts WHOLE elements
  (`latexml::util::test::assert_element`), never substrings, and has a control the old code
  already passed. A fixed RED repro under `tools/perfect_kernel/repros/<topic>/` gets its
  `% status:` line changed to `GREEN (Gemini round <N>, <guard name>)`; do not delete it.
- **Check `perfect_kernel` before adding a definition:** `git fetch && git grep '<name>'
  origin/perfect_kernel -- latexml_package latexml_contrib latexml_engine`.
- **A leniency or a kernel change is a divergence:** say in Status whether the change diverges
  from Perl (file:line of the Perl site) and what it broadens; run the witness through same-host
  Perl (`~/perl5/bin/latexml`) first.
- **Witnesses are reconverted before and after:** report ANSI-stripped `^(Error|Fatal):` counts;
  a task is not done on the guard alone.
- **Run the goldens outside your module** before committing: `cargo test -p latexml --test
  00_tokenize`, `--test 10_expansion`, `--test 06_cluster_bibliography`, and `git grep -l
  '<env or macro>' latexml_oxide/tests` for the goldens your change can reach.
- **Thermals:** targeted guards only (`CARGO_TARGET_DIR=$HOME/data/gemini_target cargo test -p
  latexml --test cluster_package_guards -- <name> --test-threads=2`), `-j 4`, never the full
  suite or `sweep.sh`. **Every conversion ≤ 3 minutes** (`--timeout=180`, outer `timeout 200`).
  Your worktree has no `resources/dumps/`: run `tools/make_formats.sh` once after checkout.
- **Data:** the current sweep is **#130**: per-doc logs and XML in
  `~/data/perfect_kernel_s130/<bundle>/<name>/`, HTML recall in
  `~/data/perfect_kernel_s130_html/s3_verdicts.tsv`; `~/data/perfect_kernel/corpus.tsv`,
  `~/data/perfect_kernel/oracle_verdicts.tsv` (column 3 = engine). Convert from a COPY of a doc
  dir with `--preload='[rawstyles,rawclasses]latexml.sty'` (`[rawstyles,rawclasses,luatex]` for
  lualatex manuals); errors are ANSI-stripped `^Error:|^Fatal:`. Never run an engine with its
  working directory inside `/usr/local/texlive` (copy the file out first).
- **Done =** red repro → fix → green guard with a control → witnesses reconverted
  (before/after error counts) → `cargo +nightly fmt --all` + clippy clean → commit → Status
  entry with guard name, witnesses, settled dead ends (one line each).

## Tasks (priority order)

None open. Round 13 (Q1–Q9) merged; its Status is lifted into `LEDGER.md` and its residuals into
`docs/SYNC_STATUS.md`.

## Status (Gemini → orchestrator; append-only, newest last; current round only)

(empty)
