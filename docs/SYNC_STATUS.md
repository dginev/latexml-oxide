# Engine Sync Status — Active Worklist

> **DO NOT downgrade Errors to cheat the task.** If Perl LaTeXML converts a paper
> without a downgrade, the Rust translation must match by improving the core
> engine — never by silencing diagnostics. New downgrades require explicit proof
> Perl emits the same severity on the SAME paper, else they hide a real gap.
> (User directive 2026-05-15.) Always classify with `latexml --verbose`, never
> `--quiet` (which hides Perl's `Error:` lines); cross-check pathological inputs
> with `pdflatex`.

> **History note (compacted 2026-06-20):** the day-by-day fix log, Round-37 /
> R-stage sweep entries, and completed-task records were removed from this file —
> they live in `git log` and `docs/archive/`. This file is now the *brief
> actionable list*. When you close an item, delete it here (git keeps the record).

## Current status

- `cargo test --tests`: **1459 / 0 / 0**.
- `cargo clippy --workspace --all-targets -- -D warnings`: **clean**.
- `--init=plain.tex` / `--init=latex.ltx`: **0 errors** (with dump and `LATEXML_NODUMP=1`).
- Distribution build (`maxperf`): ~45 MB; beats 2× pdflatex on the mini-benchmark.

Methodology that's working (2026-06): **re-triage LARGE-error papers** (the
single-error tail is exhausted) → bisect the doc to the trigger line → verify
Perl with `--verbose` → fix the Perl divergence. Random sweeps are low-yield;
prefer the cortex cross-join (svc Rust `oxidized-tex-to-html` vs Perl
`tex_to_html`) for a precise Rust-only worklist.

**Cortex agentic API (preferred over psql, reads are open — no token):**
`http://127.0.0.1:8000/api` lists 49 endpoints. Worklist recipe:
`GET /api/reports/<corpus>/oxidized-tex-to-html/<severity>` → categories;
`…/<severity>/<category>` → the per-`what` breakdown; `…/<category>/<what>` →
the paper list. Then `GET /api/corpus/<corpus>/tex_to_html/document/<id>` for the
Perl status — a Rust-only win is one where **Perl=no_problem/warning but
Rust=error/fatal**. Corpus `sandbox-arxiv-10k-shuffle`. URL-encode `\`→`%5C`,
`^`→`%5E`. **Empirical 2026-06-20 cross-join of the 10k Rust errors: failures
are overwhelmingly SHARED** — the `\bauthor`/imsart bib cluster (16 papers),
the Timeout/fatal papers (14/15 also Perl-fatal), and the `ltx:XMApp` malformed
cluster are all shared; the structural-malformed scan found ZERO Perl-clean
cases. Rust-only wins are rare one-offs (e.g. `0805.1040` `\notetoeditor`,
fixed). Conclusion: the aggregate error tail is mostly shared upstream gaps.
**STALENESS CAVEAT:** the 10k Rust run predates recent fixes — always re-confirm
a flagged paper on the CURRENT binary before chasing it. E.g. the whole
`document/convert` "Can not mutably reference a shared Node" cluster (16 papers,
high Rc strong-count on `text`/`creator`) reads as Rust-only in the API but is
**already fixed** — all sampled papers show 0 such errors locally now. Trigger a
fresh Rust rerun (needs `X-Cortex-Token`) for accurate cross-join counts.

**10k easy-parity seam MINED OUT (verified 2026-06-20).** A full local-verified
sweep of the 10k Rust error categories (`undefined`, `unexpected` `_`/`^`/`$`/`\fi`,
`malformed`, `misdefined`, `document`, `invalid`) found every remaining apparent
"Rust-only" cluster traces to a SHARED cause — confirm before chasing:
- **Third-party class/pkg neither engine binds** → identical errors: imsart via
  `\documentclass{arximspdf}` (ships `arximspdf.cls` but `\RequirePackage{imsart}`
  and **imsart.sty is host-missing**) → `\bauthor`/`\b*` undefined in BOTH;
  `jpconf` (`jpconf.cls` not in texmf) → `\ack` undefined in BOTH; `feynmf`/`fmf*`
  (Rust already beats Perl); `changes`/`\setremarkmarkup` (`#` leaks in both).
- **Author errors**: `\DeclareCaptionFormat` used without `\usepackage{caption}`
  (1608.07271); minimal repro WITH caption is clean in both.
- **Stale cortex run** (predates fixes): `document/convert` shared-Node cluster.
The genuine Rust-only wins were the harvested one-offs (`\ifodd`→glossary,
`\notetoeditor`, `\endpage`). Further 10k cross-join needs a rerun built FROM
this branch; otherwise pivot to the beyond-parity long-tail below.

**Beyond-parity long-tail coverage candidates (#2 track, surpass-Perl —
defer while strict-parity is #1):** add `arximspdf`/`imsart` support (16+ IMS
papers: aop/aos; needs a bundled/stubbed imsart.sty equivalent since the host
lacks it); `jpconf` class → map to iopart_support (18+ IOP-conf papers);
theorem/mdframed-in-figure schema (`figure_mixed_content`, task §1).

---

## Math-parser / content-MathML gaps — DEFERRED to a dedicated session

> **User directive 2026-06-20: defer ALL content-MathML-related items to a
> dedicated session; keep notes on each (below).** The math parser is a full
> rewrite (Marpa vs RecDescent) and these touch its parse-tree / content-MathML
> structure — best tackled together with focused regression budget, not piecemeal
> in the general worklist. **LANDED already** (clean parse-gap fixes, keep):
> `\mid`-in-fences (`439630485a`), `\|x\|`/`\Vert` norm (`6aa90dd13d`), `\nabla^2`
> scripted-operator (`35525e6f38`). **The open items below are PARKED** until the
> dedicated session:
> - **`f(a,b)` multi-arg flattening** (HIGH value, central/risky): every
>   function/opfunction applied to a paren comma-list wraps it as `vector@`
>   (`\max(a,b)`→`maximum@(vector@(a,b))`) vs Perl flat `maximum@(a,b)`. Fix =
>   flatten the comma-list into the apply's content-branch args + pruning
>   preference; touches core function application. Full diagnosis below.
> - **`[a|b]` bracket-conditional** (additive-safe, niche): unparsed in Rust;
>   Perl `delimited-[]@(conditional@(a,b))` (e.g. `E[X|Y]`). Rust HAS both pieces
>   (`a|b`→`conditional@`, `[x]`→`delimited-[]@`) — they don't compose inside
>   `[...]` because the bare-vertbar conditional sits at `statements` level, not
>   `expression` (which `lbracket expression rbracket => fenced` needs). Fix =
>   a surgical `lbracket … singlevertbar … rbracket` rule producing the two-level
>   structure, OR lift the vertbar-conditional to `expression`.
> - **`⁡` DecorateOperator over-insertion** (presentation MathML): Rust's blanket
>   `parser.rs:711-743` post-walk decorates ALL operator-base SCRIPTOP applies
>   with role, so presentation emits `⁡` (U+2061) where Perl juxtaposes — even
>   unscripted `\nabla \phi` (`∇⁡ϕ` vs `∇ϕ`). Fix = make the walk selective like
>   Perl's `addOpDecoration` (drop OPERATOR/DIFFOP from the blanket list).

## Math-parser Rust-only gaps (parked — found by Rust-vs-Perl `text=` comparison)

**Method (high-yield, repeatable):** math-parse failures are SILENT
(`ltx_math_unparsed` fallback, no `Error:`) so the cortex error cross-join never
surfaces them — instead convert a diverse formula batch with both engines and
diff the core-XML `text=` (`/usr/local/bin/latexml --quiet`). The math parser is
a full rewrite (Marpa vs RecDescent), so it's the richest seam for Rust-only
divergences. Landed via this method: `\mid`-in-fences (`439630485a`), `\|x\|`/
`\Vert` norm (`6aa90dd13d`), `\nabla^2 \phi` scripted-operator (`35525e6f38`).
**Still open (reproduces as `ltx_math_unparsed` in Rust, parses in Perl):**
- **`[a \mid b]` / `[a|b]`** (bracket-conditional) → Perl
  `delimited-[]@(conditional@(a,b))`. Paren `(a|b)` and brace `{a|b}` conditional
  rules exist (builder.rs ~549/557) but bracket does not; bare `a|b` parses
  inside `[...]` differently. Needs the `[...]`-delimiter + inner-conditional
  path (NOT a simple new fence rule — Perl wraps it in `delimited-[]`).

**HIGH-VALUE STRUCTURAL divergence — multi-arg function application
`f(a,b)` (found 2026-06-20, NOT yet fixed; central/risky — wants a focused
session or a steer):** every function/opfunction applied to a parenthesized
comma-list wraps the list as a single `vector@` arg, where Perl flattens to
multi-arg. Affects `f(a,b)`, `\max(a,b)`, `\min(x,y,z)`, `\gcd(a,b)`,
`\deg(f,g)`, … (ubiquitous; impacts content-MathML for ALL multivariate calls):
- `\max(a,b)` → Rust `maximum@(vector@(a, b))` vs Perl `maximum@(a, b)`.
- `f(a,b)` (declared FUNCTION) → Rust `f@(vector@(a, b))` vs Perl `f@(a, b)`.
Perl's `ApplyDelimited`/`extract_separators` (MathParser.pm) drops the commas
and puts the items DIRECTLY as the operator's args (flat). Rust path:
`(a,b)` → `lparen formula_list rparen => fenced` (vector), then
`function fenced_factor => prefix_apply` (wrap). Note the comment at
builder.rs:699-704 — intervals were pulled out of `fenced_factor` SO `f(x,y)`
uses the "list interpretation", but that interpretation WRAPS (vector) rather
than FLATTENS like Perl. Fix = add `function/opfunction/trigfunction lparen
formula_list rparen => apply_delimited`-FLATTEN rules (apply_delimited extracts
the comma-list items as the content-branch args, presentation keeps `(a,b)`),
with pruning to prefer the flattened apply over the vector-wrap prefix_apply.
Central + pruning-sensitive → must be done carefully with full-suite + Perl
cross-check (NOT a quick cron fix). Single-arg `f(x)` and the
undeclared-letter `f(x)`/`P(...)` apply-vs-multiply ambiguity are SEPARATE and
out of scope here.

**SEPARATE pre-existing divergence (NOT a parse gap — surfaced while fixing
`\nabla^2`):** Rust inserts `⁡` (U+2061 FUNCTION APPLICATION) in *presentation*
MathML for OPERATOR applications where Perl uses bare juxtaposition — e.g. even
unscripted `\nabla \phi` is `∇⁡ϕ` (Rust) vs `∇ϕ` (Perl). Traced to
`parser.rs:711-743` `DecorateOperator`: Perl calls it SELECTIVELY from the
grammar's `addOpDecoration` (MathGrammar:697, additive-op-chain context), but
Rust applies it as a BLANKET post-parse DOM walk over every SCRIPTOP `XMApp`
whose base role ∈ {MULOP,ADDOP,…,OPERATOR,DIFFOP}, so it over-decorates
operator applications; the `role="OPERATOR"` then makes the presentation
post-processor emit `⁡`. Broad (all `\nabla`/operator applications), invisible
glyph, content `text=` already matches — so low-priority, but it's the next
operator-presentation parity item. Fix = make the walk match Perl's selective
call sites (likely drop OPERATOR/DIFFOP from the blanket list, verify nothing
that NEEDS infix decoration regresses).

CAUTION (from the norm fix): new VERTBAR/fence grammar rules can collide with
package-built structures (the norm rule initially regressed `physics_test` —
turned out physics.xml was a STALE divergence and Perl matched the new output,
so it was a parity *fix*; always cross-check the affected fixture against Perl
before assuming a regression).

## Open tasks (actionable)

### 1. `ERROR_DEBT` test-gate drain (the remaining regression test still erroring)
The harness error-gate (`latexml_oxide/src/util/test.rs`) fails a test at zero
debt to force removal once fixed. Drive each to clean via a real core fix:
- **`figure_mixed_content`** — `ltx:theorem` not allowed in `ltx:figure` (Perl
  also errors 1). True fix = **schema expansion** (theorems/mdframed in figures).

  (**`glossary` — ✅ CLOSED 2026-06-20.** Root cause was NOT a datatool/expl3
  gap but a core `\ifodd` bug: Rust's truncated `%` made `\ifodd` report every
  *negative odd* integer as even (`-23 % 2 == -1`, gated on `== 1`). expl3
  l3regex stores its cs-mode-in-class compile state as the negative odd int
  `-23`, and `\__regex_if_in_class:` is `\if_int_odd:w \l__regex_mode_int`, so
  `\c{[...]}` patterns (datatool-base.sty word/initials parsing) lost their
  char-class → an unclosed `\if_false:{\fi:}` brace-trick → `readBalanced` ran
  off the end → ~50 undefined `\__datatool_*` cascade. Fix: `\ifodd` tests
  `% 2 != 0` (`tex_logic.rs`), matching Perl `valueOf % 2`-as-boolean and TeX's
  sign-independent oddness. datatool-base raw-loads at 0 errors; entry removed
  from `ERROR_DEBT`.)

### 2. PGO of the release build — tooling LANDED, measurement pending
`tools/make_release_pgo.sh` (instrument → train → merge) + `make_release.sh`
`PGO_PROFILE` hook are in; operator recipe in `RELEASING.md` §3b. **Remaining:**
the maxperf perf measurement on the full-corpus hardware (the dev box is
freeze-prone/unrepresentative). Deliberately NOT a CI job (no arXiv corpus in
GitHub Actions). Design: `PERFORMANCE.md` → build-pipeline roadmap. BOLT +
`target-cpu=v2/v3` stack on top, also deferred to that hardware.

### 3. Confirmed open Rust-only gap: `\gls`/`\acrshort` in MATH mode (1705.10306)
293 errors `ltx:XMTok isn't allowed in <ltx:glossaryref>` (Perl 1). A glossary
command in math mode makes the `glossaryref` content digest as math → bare
`<XMTok>`, which the content model rejects. **Blocked** on a clean Perl target:
the minimal repro is confounded by the glossaries-package's own datatool/l3regex
errors (both engines) and Perl **times out** on the full paper. Fix needs the
core document-builder math-in-text handling. Repro:
`docs/reproducers/glossaryref_math_xmtok.tex`.

### 4. PR #248 B1 — re-entrant `&mut Document` UB (runtime-bindings), accepted caveat
The Rhai constructor trampoline re-mints `&mut Document` (Stacked/Tree-Borrows UB
under a re-entrant `\wrap{\myemph{..}}`). Consolidated to one audited
`script_bindings/mod.rs::with_doc` site + documented; the review's checked-guard
fix **deadlocks** `Document::absorb` (which needs the nested construction to
succeed). **Optional future work:** make re-entrancy *sound while succeeding* —
interior-mutable `Document` or a core handle around `do_absorption`. Not a
blocker; `runtime-bindings` stays on by default.

### 5. 0.7.0 release — release-prep LANDED; tag pending
Version bumped, `runtime-bindings` in the artifact, `.deb` deps, CHANGELOG/README
done (see git). **Remaining:** tag `0.7.0` on master → `release.yml` runs the
TL-window `dumps` + macOS arm64 leg + publish (each first-exercised on that tag).

---

## Deep deferred families (parked — large or shared; tackle in dedicated sessions)

- **#A l3regex — ✅ RESOLVED 2026-06-20: the real expl3 VM works natively.** The
  feasibility probe (per user direction, consulting `expl3-code.tex:26422+`)
  showed `\regex_match` (inline + compiled-var), `\regex_count`,
  `\regex_replace_all`, `\regex_extract_once` and the `\seq_*` results all run
  correctly via the real VM — intervening gullet fixes cleared the old
  `\if_int_compare:w` timing stall. So the Rust-`regex`-crate **shim in
  `expl3_sty.rs` was REMOVED** (faithful + complete). Verified: original cascade
  witness 2406.14142 (21 errors → 0), full suite 1459/0, new
  `expl3/regex_native` test. **datatool — ✅ now raw-loads cleanly** after the
  `\ifodd` negative-odd fix (2026-06-20) that unblocked `\c{[...]}` regex
  compilation (see closed `glossary` item under Open tasks §1); its
  word/initials name-parsing defines without errors.
- **1610.00974 step-3** — port the *global* `p{}` column to the Perl VBox form
  (`\lx@tabular@p`/VBoxContents). The narrow `\multicolumn{}{p{}}` case is already
  fixed; the global port exposes a `\cr`-mid-VBoxContents-predigest interleaving +
  a span/sizing bug on `\multicolumn` over p-columns (graphrot). Surpass-Perl R&D.
- **`expected:id` cmml dangling-XMRef tail** — MathFork/split content-arm xml:id
  duplication; the last live `expected:id` class. See
  `docs/EXPECTED_ID_XMREF_DESIGN_2026-06-08.md`.
- **xy-pic `svg:path` / curve cluster** (1501.03690) — shifted-arrows `svg:path`
  in `ltx:text`; mode-frame cascade root.

**SHARED (both engines fail — match Perl, not Rust-only gaps; do NOT "fix" by
downgrading):**
- **1804.01117 xint raw-load** — in `includestyles`/ar5iv both raw-load xint and
  fail; in plain both stub it (byte-identical). The only Rust-worse bit was a
  stack-overflow crash, now FIXED by the gullet `stack_guard` (configurable via
  `latexml_core::stack_guard`). Neither engine converts it. Deep xint emulation
  parked (not needed for parity).
- **mode-frame auto-close cluster** (1611.04940, 2009.05630, 1702.06692,
  1702.02037) — a theorem env opened via its bare begin-command (`\step`,
  `\case`) with no matching `\end…` leaks the mode-switch frame to the enclosing
  `\endgroup`; Perl `Core/Stomach.pm:343-376` errors identically. A graceful
  auto-close would *surpass* Perl (beyond-parity R&D), not a parity fix.

---

## Reference (stable — not active work)

### Engine file open gaps (MINOR, demand-driven)
- `tex_box.rs` box-dimension edge cases; `tex_fonts.rs` `\fontdimen` array +
  per-font `\hyphenchar`; `tex_tables.rs` padding CSS (XSLT concern).
- **~72-CS Perl-only long tail** (from the archived LoadFormat audit): misc
  atomics (`\@charlb`, point-size CSes, `\batchmode`, …) Perl defines and Rust
  does not. Investigate a CS only when a real paper witnesses it. Refresh the
  CS-name diff before quoting counts (it predates the BibTeX port).

### Permanent ignores
- **Out-of-scope**: ns1–ns5 (`52_namespace`, no DTD support); 2402.03300,
  2410.10068, 2511.03798 (Perl also fails).
- **Rust supersedes Perl** (Rust passes where Perl errors): `1207.6068`,
  `0909.3444`, + 40 more in `memory/project_rust_supersedes_perl.md`.
- **BibTeX**: `BibTeX.pool.ltxml` is ported (Phases 1–8; remaining B1–B6 polish
  in `BIBTEX_PORT_PLAN.md`). `--nobibtex` is an opt-out, not the default.

### Tikz known diffs vs Perl
`foreignObject` transform; arrow-tip path data; SVG viewBox/width; matrix
`<svg:g class="ltx_tikzmatrix">` vs inline-blocks; **bare `svg:g` in `<ltx:block>`**
(tikz-cd) trips a core-XML validity error but post-processing recovers
(witness 2006.12702) — Rust-only, low priority (output recovered).

### Graphics renderer chain (subprocess-only; LANDED)
PDF→PNG `mutool draw`→`pdftocairo`→`convert+gs`; PDF→SVG `mutool convert`→
`pdftocairo`→`inkscape`. Subprocess `exec` (no GPL linking). Apt:
`poppler-utils` (req), `mupdf-tools` (rec), `imagemagick+ghostscript`, `inkscape`.

### Other tracks (separate docs)
- Performance: `docs/PERFORMANCE.md` (P1 math/large-doc open; P2 allocation partial).
- Release gates: `docs/RELEASE_CRITERIA.md`. Releasing: `docs/RELEASING.md`.
- Completed missions (archived): strict-LoadFormat dump parity, Marpa ASF
  migration, distribution-readiness, the 500K/1M warning-corpus mission — see
  `docs/archive/` and git history.
