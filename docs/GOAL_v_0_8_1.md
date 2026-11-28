# Goal: towards v0.8.1

**The assignment (DG 2026-11-23).** Solve all planned tasks in [`SYNC_STATUS.md`](SYNC_STATUS.md). The arXiv long tail —
tasks affecting fewer than 200 articles, or synthetic-only — is out of scope: it lives in [`LONG_TAIL.md`](LONG_TAIL.md)
for v0.8.2 and later (DG 2026-11-23). Land the work
on the integration branch `int/towards_v0_8_1`. Then work through the open user-reported issues and pull requests.
Work carefully and honestly: every claim backed by a run, every miss named.

This page is the contract for that work: the two timelines, the order, what "solved" means, the bar for merging the
branch, and the standards every change meets. The standards distill DG's standing rulings, CLAUDE.md,
[`parity/FIDELITY_CHARTER.md`](parity/FIDELITY_CHARTER.md) and the Guiding Principles of
[`parity/OXIDIZED_DESIGN.md`](parity/OXIDIZED_DESIGN.md); where they disagree, those sources win and this page is fixed.

## Two timelines (DG 2026-11-23)

| | v0.8.0 final | v0.8.1 |
|---|---|---|
| **When** | released 2026-11-28 UTC (tag `b954c9e5162`) | an 8-week sprint after v0.8.0 (DG 2026-11-27) |
| **Branch** | `main` (the release line until the tag) | `int/towards_v0_8_1`, cut from `main` |
| **What** | Small reviewed fixes only: regressions, crashes and Fatals, bugs rc2 and rc3 testers report, release docs | Everything else — most of the work: the row R tasks, the reviewed post-release branches, redesigns |
| **Not allowed** | Redesigns, new CLI options or Cargo features, output changes without a reach-sized A/B | Nothing extra beyond the standards below |
| **Landing** | One PR per change; a user-reported issue gets its own PR | Until the branch reaches `main`, batches merge into it and it lands as one PR; after that (DG 2026-11-27), one PR per feature or sprint against `main`, and the branch retires |

Every merge to `main` is merged into `int/towards_v0_8_1` the same day, with the full gates on the result. A task that
starts on one timeline and turns out to need a redesign moves to v0.8.1 with a row R note.

## Order of work

1. **v0.8.0 blockers first**, while the release window is open: tester reports on rc2 and any regression found on
   `main`, each through the issue procedure below.
2. **The reviewed post-release branches** (COPYROUTE, ENCURL) land on `int/towards_v0_8_1`, then the row R items
   marked post-0.8.0.
3. **Row R by the campaign order and the pragmatic bar:** regressions and Fatals first; then high-frequency or
   high-impact gaps (reach ≳ 500 papers, or content lost at scale); then counter-examples that break an approach. The
   critical path goes first, easy items fill parallel slots in other files.
4. **User-reported issues and pull requests** still open once row R is resolved (bugs with an oracle first; requests
   for new behaviour get 2–3 designs and a DG ruling before any code).

## What "solved" means for a row R task

Each task ends in exactly one of these states, recorded in its row R line and moved, in conclusion form, to the month's
`archive/SYNC_SESSIONS_YYYY-MM.md`:

- **Fixed:** a red/green guard, the fix on `int/towards_v0_8_1` (or `main` for a v0.8.0 fix), the docs it touches
  updated, witnesses reconverted.
- **Ruled:** parity, out of scope, or declined, with DG's ruling quoted in DG's words, a `_red.tex` or a witness kept.
- **Moved:** to a parked family doc (`parity/DEFERRED_FAMILIES.md`, `math/…`) with witnesses, marked "decided by
  difficulty; DG may overrule".

"Solve all" is measured on row R itself: the count of open items, with its weight (1 easy, 3 medium, 5 hard), drops to
zero. Report progress as "X of Y remain", never as "fixed" by cluster coverage.

## The bar for merging `int/towards_v0_8_1` into `main`

- Full local gates on the exact head: `cargo nextest run --workspace` 0 failed, clippy `-D warnings`, rustdoc, fmt,
  zero-error `--init` dumps on the tree under test.
- CI green on Linux and macOS, and the Windows full suite by dispatch.
- A healthy-paper all-years A/B: 0 papers worse on the three metrics (honestly error-free, authored content kept,
  semantic markup); every changed paper byte-diffed and classified.
- A 2605/2606 sandbox rerun with no new Fatal cluster against the last baseline.
- Performance: ≤ +0.5 % instructions:u per document on the measured set, no superlinear cost.
- Docs current: `release/CHANGES_v0.8.1.md`, row R, OD entries, `README.md` indexes.
- A reviewer's SHIP on the branch diff, then DG's go.

## Method: isolate, test, then fix faithfully (DG 2026-11-23)

Every problem of TeX interpretation is worked the same way:

1. **Reproduce** on the current binary, same host, verbose; judge by the `Status:conversion` code and ANSI-stripped
   logs, never by grepping for success.
2. **Isolate with a minimal `.tex`** (the `min-repro` skill, `tools/bisect_repro.sh`): shrink the paper to the
   smallest input that still fails, and keep a control twin that differs by one construct, so the culprit token is
   pinned. The repro counts only if same-host pdflatex runs it with 0 `!` and the expected text (or the expected `!`
   for an error case). It lives under `tools/perfect_kernel/repros/<family>/` with the standard header: mechanism,
   witness, oracle, engines, expect, status.
3. **Classify the origin before any fix** (the `canvas-triage` skill): Rust-only, parity with Perl, or Perl and TeX
   disagreeing. Parity is never absolution; it decides where the record goes.
4. **Write the red test first**: the repro becomes a row in an existing table-driven guard, red at the base with the
   failing line recorded. Assert whole elements or goldens and exact Error/Warning counts, with safe twins on the
   other side of every boundary. Only distinct hard cases become guards; confirming probes are reported, not added.
5. **Fix faithfully, at the general mechanism**, on two axes at once:
   - **Perl LaTeXML's architecture**: the same pipeline (Mouth → Gullet → Stomach → Document → Post), the same data
     model and interfaces, definitions in the files and order Perl uses them, one name with one meaning across Perl,
     the Rust macros and Rhai, and a binding defining a macro where its real definer does. No new abstraction
     without a stated reason the existing primitive cannot answer the question.
   - **TeX's engine algorithms**: where Perl approximates, implement what tex.web does (cite the §), and what
     latex.ltx and the package's own source do (file:line). Compute a value when TeX computes it, mirror TeX's
     irregular readers only where TeX has them, and take numerics from tex.web.
   - A divergence from Perl is recorded in OD and offered to Perl; a Perl-origin bug goes to KNOWN_PERL_ERRORS.
     The rule the source already has beats a special case; a third patch of one shape becomes a shared mechanism,
     decided with DG after cataloguing its cases.
6. **Green, then widen**: the guard green for the right reason, safe twins byte-identical, every witness named in
   the touched comments reconverted, and a reach-complete A/B when output moves.

## Standards

### Fidelity

- Read the Perl source first; it is the translation spec for the data model and interfaces. Write idiomatic Rust
  inside functions.
- Where Perl approximates TeX, real TeX decides: tex.web, latex.ltx, the real `.sty`/`.cls` cited by file:line,
  same-host pdflatex. Record the divergence in OD and offer Perl the patch.
- Bruce Miller's principle: process TeX as faithfully to TeX's model as possible while building a document model
  that keeps the semantics.
- Report a finding's origin (parity, Rust-only, environment) apart from its impact. "Perl errs too" is never absolution.
- A synthetic probe counts only if same-host pdflatex runs it with 0 `!` and the expected text.
- A delegated, database or memory verdict is a claim: re-run it on the current binary, same host, verbose. Compare
  with Perl `--verbose`, never `--quiet`. The cortex DB is a screen, never a verdict.
- Numerics (sp, dimension and glue rounding) are judged against pdflatex and tex.web, never Perl.
- A kernel command newer than the paper calls for a legacy-TL dump, never a special case.

### Priorities

- Content > semantic markup > PDF fidelity. PDF fidelity gives way first; author content is never dropped.
- Content, semantic markup and complete breakage (a missing image, a broken `\ref`) are held to 100 %.
- Regressions and Fatals first. When a Fatal is picked, name its class and fix every sibling with one mechanism.
- Every silent loss is fixed or made to raise or count a diagnostic.
- Pathological authoring gets a RED and a parked-family line, never recovery that changes the well-formed case.

### Design

- Generalize: implement the rule the source has. A flag keyed on one file or package is a smell; a third patch of
  one shape becomes a shared mechanism, decided with DG after cataloguing its cases.
- Principled mechanisms before text heuristics. Reuse the primitives Perl and the port already have.
- Bindings precede raw loading. A binding defines a macro where its real definer does, under the same conditions;
  prefer thin bindings over the real package. No speculative bindings.
- Keep the four dump-boundary rules of CLAUDE.md (LoadFormat exclusivity, unconditional apply, same-file definitions,
  zero-error inits).
- One public name means the same thing in Perl, the Rust macros and Rhai.
- OD holds divergences from Perl's observable behaviour only, one entry per mechanism; extend before adding.

### Code

- No new `unsafe`; existing `unsafe` stays at FFI and OS boundaries and does not grow. No FFI in latexml-oxide:
  missing wrappers go to rust-libxml upstream.
- Fx maps, thread-local state, the token macros, Perl order with `// Perl L…` comments (see CLAUDE.md).
- Comments are 1–2 lines plus witnesses and Perl/tex.web references. Never delete a witness id; reconvert named
  witnesses before landing.
- Minimal code, maximal coverage: prefer deletion and sharing; every review names the DRY reductions.
- No `.py` file in the repo. The binary stays self-contained: owned resources embedded, only the host TeX read.

### Tests and verification

- Red/green: a minimal pdflatex-checked `.tex` and a whole-element guard, red at the base with the failing line
  recorded, then green.
- A gap reported but not fixed becomes a `_red.tex` whose `moves:` names its row, or an out-of-scope row R line.
- A new case is a row in an existing table-driven guard. Fixtures have 0 Error and 0 Warning unless pinned by count.
- Full suite on the exact committed tree before any A/B; read every gate line. Byte-diff A/B outputs and classify
  every changed paper.
- Per change: a reach-complete A/B (every paper using the construct, 20–200) when output moves. Engine changes also
  regenerate the dumps (zero errors, sorted diff) and measure instructions:u.
- CSS and MathML changes run the browser tests in Chrome and Firefox, judged by ink.
- Portable tests: green on every TeX Live; skip visibly where a package is absent; never pin CI to our TL.

### Performance

- Gains are algorithmic and keep the Perl-shaped interfaces. One lever per run, keep/revert bars written first,
  instructions:u on a `--profile bench` build.
- ≤ +0.5 % instructions:u per document for any change; superlinear costs are fixed regardless.
- Deep-dive surprising differences to the machine code, and say plainly when the mechanism is unknown.

### Errors and severity

- Every distinct Warning, Error and Fatal is emitted and counted; recovery never silences one.
- An Error where TeX or Perl errs, unless a ruling says otherwise; a false Error is worse than a Warning. A Fatal
  stays Fatal. A panic is a bug and becomes an Error mirroring TeX's.
- Run a census before any severity change; land it with rows on both sides of the boundary.

### HTML, MathML and CSS

- Emit MathML Core; check the Core spec in `background/mathml-core/`, never argue from MathML 3.
- `ltx_*` classes and the HTML structure are a public contract; renames are recorded.
- Every CSS change is mirrored into ar5iv-css; its releases wait for DG.
- Rendering premises are checked in both browsers before asking DG; final sizing CSS is DG's design choice.

## Process

- **Agents:** at most 4 at once (DG 2026-11-23). Narrow and bounded: analysts are read-only; only payload writers
  write, each on its own `pk/<batch>` branch and worktree, pinned to cores 64-127; a fresh writer per fix round.
  Timebox writers ~60–75 min and reviewers ~30–45 min; at the budget, commit what is green and report NOT DONE.
- **Review:** a reviewer before every integration. Blockers are a regression, a content loss or a real paper. At most
  2 fix rounds; a design question that fails twice goes to DG with the exact scenario. When NO-SHIPs pile up, an
  advisor checks the approach.
- **Git:** every change is a PR from a fresh worktree; never commit to `main`; never amend. Docs-only PRs merge after
  a local lint. Delete a merged worktree and its target directory.
- **Issues:** grep the CHANGELOG and `git log` for the issue; classify; canvas-triage → min-repro → perl-port on a
  per-ticket branch; red/green; A/B when output moves; gates, reviewer, the issue's own PR; once merged, close it
  with a reply to the reporter (anything short of a full resolution needs DG's OK).
- **Safety:** one heavy workload at a time; 8 GB per conversion; arXiv sources only from the local archive into a
  fresh scratch directory; never touch the frontmatter holdout; no network for corpus data.
- **Public hygiene:** no arXiv regression counts or lists in public text; contributors credited by their public
  identity.
- **Docs:** row R is the single worklist, edited by the main session at every decision and merge. Record conclusions,
  not play-by-play: witnesses, file:line, guards, measured figures, settled dead-ends.

## Open questions for DG

- **After the 0.8.0 tag:** do the release-phase freezes (no new CLI options or features, redesigns deferred) lift on
  `int/towards_v0_8_1`? This page assumes yes.
- **Math:** is math still parked except named exceptions during v0.8.1, with the math-parse study on Nasser's
  mathematics as one of them?
- **Frontmatter:** is a new holdout draw still planned, or are the frontmatter bars closed for v0.8.1?
- **Issue 972:** decided (DG 2026-11-26): Grow, in 0.8.0 through rc6, in the base LaTeXML.css and the bundled
  ar5iv-css 0.9.6.
