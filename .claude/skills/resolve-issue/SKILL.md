---
name: resolve-issue
description: >
  Drive a public GitHub issue (bug report / feature request / docs request) to a
  landed resolution on a fresh per-ticket branch. Classifies the issue type, then
  — for the common case of a TeX source + its conversion — builds a red/green TDD
  reproducer, writes the approach to a scratch TICKET_APPROACH.md, implements to
  green, and opens the PR. Pairs with canvas-triage, min-repro, perl-port. Invoke
  for "resolve issue #N", "work on the bug report", "fix this feature request",
  "handle issue X", "/resolve-issue".
---

The repo is public; issues now arrive from real users. This skill is the outer
loop that wraps the others (`canvas-triage` → `min-repro` → `perl-port`) into one
issue-to-PR procedure. **Faithful translation still governs everything: Perl is
ground truth, never downgrade errors to "pass", diverge only when
`OXIDIZED_DESIGN.md` sanctions it.**

## Step 0 — Branch for the ticket (first, always)

One ticket = one branch = one PR. Never work on `main`, and never pile a new
ticket onto another ticket's branch/PR. Read the issue title to get the number +
a slug, then cut a branch off an up-to-date `main`:

```bash
git switch main && git pull --ff-only
git switch -c fix-<N>-<short-slug>      # e.g. fix-293-subimport-standalone
```

Use a `fix-` / `feat-` / `docs-` prefix per the issue type (Step 1). If triage
(Step 2) later proves the issue is parity / out-of-scope / not-ours, abandon the
branch (`git switch main && git branch -D fix-<N>-…`) — no PR. Cut the branch off
`main` even when another ticket's PR is open, so the two stay independent.

## Step 1 — Classify the issue

Read the issue in full (`gh issue view <N> --json title,body,labels,author`).
Pick exactly one type; it selects the path:

| Type | Signal | Path |
|---|---|---|
| **1 — Bug** | "X is wrong / crashes / differs from LaTeX" | red/green TDD → patch (the main path below) |
| **2 — Feature** | "add / support / it would be nice if" | design plan first → then TDD-implement |
| **3 — Docs** | "document / explain / unclear" | refactor the doc with more explanation; verify claims against real behavior |
| **4 — Other** | meta, build, question, out-of-scope | triage; usually reduces to 1–3, else answer/ask the user |

Most issues name **a TeX source and its conversion** → Steps 2–8 apply, PR in
Step 9. Types 2/3 add a front-half; see "Type-specific notes".

## Step 2 — Reproduce and confirm it's ours

1. Put the issue's MWE verbatim into a `.tex` file (scratchpad first). Convert
   with the **current** binary: `cargo run --bin latexml_oxide -- --dest=/tmp/r.html repro.tex`.
   Confirm you see the reported symptom, not something adjacent.
2. **Classify vs Perl before writing any fix** (`canvas-triage` golden rules):
   run same-host Perl **verbose, never `--quiet`**, ANSI-strip before grepping. A
   bug the user reports is only a straight patch when it is **GENUINE-RUST-ONLY**
   (Perl behaves correctly, Rust doesn't). If Perl misbehaves identically it is
   parity/upstream (`KNOWN_PERL_ERRORS.md`) or a `surpass-perl` decision — not a
   silent divergence. If the paper is large, shrink it with `min-repro` first.

## Step 3 — Write the RED test (before the fix)

The single most important repo-specific choice: **pick the harness that exercises
the layer where the bug lives.** A core-only fixture is green-on-buggy for a
post-processing bug and will never catch it.

| Bug surfaces in | Harness | Red-test shape |
|---|---|---|
| **Core XML** (mouth/gullet/stomach/document, i.e. the `<ltx:…>` before post) | `.tex`+`.xml` sibling pair under a wired dir (`tests/structure/`, `tests/math/`, …), discovered by `tex_tests!`/`GlobTeXTests` (`latexml_codegen/src/testable.rs`) | golden `.xml` = the **intended** core output; compare is **exact line-by-line** (`util/test.rs`) + a 0-error gate |
| **Post-processing** (Scan, CrossRef/TOC, MathML, Split, Bibliography) | hand-written `.rs`: `convert_and_post()` (see `tests/06_cluster_regressions.rs`) or the `90_latexmlpost.rs` `<name>.xml`→`<name>-post.xml` pattern | assert on the post XML (structure/substring) |
| **Whole pipeline / HTML / a CLI flag** (`--stylesheet`, `--split`, …) | drive the **binary** via `Command::new(env!("CARGO_BIN_EXE_latexml_oxide"))` (the `07/08/09_xslt_*` pattern) — the in-process `Converter` stops at Core XML — or `post::run_post_processing` for the library path | assert on the final HTML / stderr |

**Red-first, not bless-first.** `LATEXML_BLESS=1 cargo test <name>` (or
`tools/maketests.sh <name>`) writes the golden **from the current — buggy —
binary**, so blessing before the fix gives you a green-on-wrong test that pins the
bug. For a bug fix, **hand-author the golden / assertion to the *intended*
output** so the test is RED for the right reason now and only goes green when the
behavior is actually fixed. (Bless-after-fix is fine for a *new* construct whose
first-correct output is whatever the fixed binary emits.)

- New `.tex`+`.xml` pair ⇒ **`cargo clean`** once (the plugin globs at compile
  time; a new pair is invisible until the test target recompiles).
- Run just this test: `cargo test <fn_name> -- --nocapture`. Confirm it FAILS,
  and that the failure line is the issue's symptom (the "canary"), not noise.
- Strip intentional-divergence artifacts from any golden (`--nocomments`, no
  `%&#10;` — CLAUDE.md "Intentional divergences").

## Step 4 — Plan → `TICKET_APPROACH.md`

Write a scratch working doc at the **repo root** named `TICKET_APPROACH.md`
(gitignored — dev scratch, **never committed**; delete or leave in scratch when
done). It is your thinking made durable across the implement loop. Include:

1. **Issue + type**: number, one-line restatement, classification, GENUINE-RUST-ONLY verdict.
2. **Reproducer + canary**: the `.tex`, the exact failing line/assertion, which layer.
3. **Root-cause hypothesis**: the Perl source that is ground truth, cited `file:line`
   (`LaTeXML/lib/LaTeXML/…`), and where the Rust diverged (`crate/src/…:line`).
4. **The faithful change**: what to port and how it stays parity-true.
5. **Variations to guard**: adjacent cases the fix must also satisfy (Step 6).
6. **Validation checklist**: the Step 7 gates.

## Step 5 — Implement to green

Use `perl-port`: **read the Perl source first**, translate faithfully, cite the
line range in a comment, obey `WISDOM.md` / `KNOWN_PERL_ERRORS.md` /
`OXIDIZED_DESIGN.md`. Iterate until the Step-3 test is GREEN — and green because
the output is correct, not because you loosened the assertion. Re-run any witness
arXiv id named in a touched comment (CLAUDE.md: never drop a witness).

## Step 6 — Variations belong in the SAME test

When implementing surfaces a complication or a neighboring case, **prefer
extending the existing `.tex` / assertion with more cases** over spawning a new
test file. One fixture that exercises the whole behavior band is easier to read
and keeps the red/green signal in one place. Add a genuinely separate `.rs` test
only when the case needs a different harness layer (Step 3 table).

**Check connected behavior.** When the root cause sits in *shared machinery* (a
dispatcher, a filter, a post-processor like `gen_toc`), the same defect often
touches other documented behavior — including an intentional Perl divergence. Ask
"what else reads this code path?" and guard it too: your fix may silently
*complete* or *regress* it. (Witness: #291's `gen_toc`-ignores-`select` defect
also broke the LaTeXML#2316 arXiv-fork "abstract exempt from `\tableofcontents`"
half — the fix repaired both, and both now have guards.)

## Step 7 — Validate & ship

- `cargo test --tests --no-fail-fast` green (new pair ⇒ `cargo clean` first).
- `cargo +nightly clippy --workspace --all-targets -- -D warnings` and
  `cargo +nightly fmt --all` (the pre-push hook enforces both, not tests).
- Perl parity re-confirmed on the witness (verbose, same host).
- **Docs:** refresh `docs/release/ISSUE_AUDIT.md`; if the root cause was an
  upstream Perl bug, record it in `KNOWN_PERL_ERRORS.md`; a sanctioned divergence
  ⇒ add the `OXIDIZED_DESIGN.md` entry in the same change.
- Commit on the ticket branch (Step 0), message referencing the issue
  (`fix #N: …`). Delete/park `TICKET_APPROACH.md` — it does not ship.

## Step 8 — Review round (adversarial, before the PR)

Stop and review your own change against three questions — this catches the
"looked done but wasn't" failures. Answer each concretely; a gap loops you back.

1. **Full scope — no leftovers.** Re-read the issue. Enumerate *every* case it
   describes (explicit + implicit), not just the headline MWE, and check the fix
   covers each. A change that greens the one MWE but leaves a sibling case (an
   edge input, another platform, a whole document class) broken is a leftover.
   *(Witness: #292 greened Linux 1582/0 but left the macOS libxml2 path broken —
   a platform leftover CI caught.)* Gap ⇒ back to Step 3 (add the case) / Step 5.
2. **Right level of abstraction — DRY, general, no scope creep.** Is the fix at
   the most GENERAL level that resolves the deficiency (fix the shared root:
   one rule/function), not N symptom patches — and not a special-case that the
   next input re-breaks? *(Witness: keying #292 on the invariant basename beat
   patching each URL scheme; #291's `gen_toc` `select` fix beat per-level hacks.)*
   BUT the general fix must stay **within the ticket's definition** — a general
   fix ≠ refactoring adjacent code or adding a feature. Bar: the smallest change
   that fixes the whole *deficiency*, not the smallest that greens the *test*,
   and no larger than the ticket warrants.
3. **Tested — no overclaiming.** Every claim you'll write in the PR
   (Diagnostic/Approach/Validation) needs a run you actually executed behind it:
   the guard test, full-suite N/0, Perl parity on the witness, CI. Don't write
   "matches Perl" / "handles all X" / "fixes the class of…" without the evidence.
   *(Witness this session: "#292 may need the libxslt fork" and "1582/0 ⇒ done"
   were both wrong until checked.)* Unverified claim ⇒ verify it or drop/soften
   it ("verified on the MWE + 2 variants" beats "fixes all edge cases").

## Step 9 — Open the pull request

Only once Steps 7–8 pass and you are confident the issue is resolved. **Hard
gate: the FULL suite must pass first** — run `cargo test --tests --no-fail-fast`
to completion on this branch and confirm **N/0** (capture the full output, not a
`tail` — the exit of a piped `| tail` is tail's, not cargo's; tally every `test
result:` line). A green targeted test is not enough; the whole suite gates the PR.
**Then do the due diligence**: enumerate side-effects, connected/adjacent
behavior, commit scope (stage specific files, not `-a`),
repo-config side-effects, parity re-checks. Then push the ticket branch (never fast-forward
`main`) and open the PR. The description is **minimal, high-level, precise** —
three labelled parts, one to three lines each, so a reviewer grasps the whole
change in ~20 seconds. No narrative, no blow-by-blow, no restating the issue body
(the reader clicks `#N`).

```bash
gh pr create --base main --head fix-<N>-<slug> \
  --title "fix #N: <outcome, not mechanism>" --body "$(cat <<'EOF'
**Diagnostic.** <root cause in 1–2 sentences: the file/function and why it produced the symptom>

**Approach.** <the faithful fix in 1–2 sentences; cite the Perl ground truth (e.g. `standalone.sty.ltxml`)>

**Validation.** <red→green guard test(s) by fn name; full suite N/0; clippy/fmt; Perl parity on the witness>

Closes #N
EOF
)"
```

- Title states the **outcome**, references the issue. `Closes #N` (or `Fixes #N`)
  so the merge auto-closes the tracker.
- Name the guard test(s) by function name — a reviewer can run them.
- One clause for connected behavior only if it changes review; otherwise omit.
- Commits carry the `Co-Authored-By` trailer; the PR body ends with the Claude
  Code trailer. Report the PR URL back to the user — do not merge it yourself.
- **Watch CI to green — the local suite is not the last gate.** After pushing,
  `gh pr checks <PR>` / `gh run watch <id>` until Linux CI, **macOS CI**, lint,
  and miri all pass. Local dev is Linux; **macOS CI runs a different native
  libxml2/libxslt**, so anything touching libxml2/libxslt (URI resolution,
  XPath, serialization) can pass locally yet fail on macOS — make such fixes
  platform-robust (key on invariants like a basename, not on a platform's URI
  composition). Witness: #292's `urn:` XSLT resolver passed Linux 1582/0 but
  failed macOS (bare vs `urn:`-composed relative import); basename-keying fixed
  both. On a CI-only failure, pull the failing job log
  (`gh api repos/<o>/<r>/actions/jobs/<job>/logs`), fix, push a follow-up commit.

## Type-specific notes

- **Type 2 (feature)** — front-load design: use the `Plan` agent or plan mode to
  weigh approaches, then `TICKET_APPROACH.md` becomes the design record. Where the
  feature is a new conversion behavior, still drive it red→green (Steps 3–6). Flag
  scope/UX decisions to the user before building.
- **Type 3 (docs)** — the "test" is a claim check: verify every statement against
  the actual code/behavior before writing it down; don't document aspiration as
  fact. Keep BOTH `docs/README.md` and the CLAUDE.md doc index current when you
  add/rename/move a doc (CLAUDE.md "Rules for these docs"). Prefer refactoring the
  owning doc over a new orphan file.
- **Type 4 (other)** — if it's a question, answer from the code; if out-of-scope
  (host TeX package, DTD, `--feature` not built), say so plainly and cite
  `ISSUE_AUDIT.md` / CLAUDE.md scope. Escalate genuine ambiguity to the user
  rather than guessing the intent.

## Related skills

`canvas-triage` (is it really our bug?) · `min-repro` (shrink the MWE) ·
`perl-port` / `port-from-perl` (the faithful fix) · `surpass-perl` (when Perl is
wrong too) · `start-session` (ground yourself first).
