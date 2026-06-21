# Engine Sync Status — Active Worklist

> **DO NOT downgrade Errors to cheat the task.** If Perl LaTeXML converts a paper
> without a downgrade, the Rust translation must match by improving the core
> engine — never by silencing diagnostics. New downgrades require explicit proof
> Perl emits the same severity on the SAME paper, else they hide a real gap.
> (User directive 2026-05-15.) Always classify with `latexml --verbose`, never
> `--quiet` (which hides Perl's `Error:` lines); cross-check pathological inputs
> with `pdflatex`.

> **This file is the BRIEF ACTIONABLE LIST.** The day-by-day fix log and
> completed-task records are NOT kept here — they live in `git log` and
> `docs/archive/`. **When you close an item, delete it here** (git keeps the
> record). Last compaction: 2026-06-21.

## Current status

- `cargo test --tests`: **1466 / 0 / 0**.
- `cargo clippy --workspace --all-targets -- -D warnings`: **clean**.
- `--init=plain.tex` / `--init=latex.ltx`: **0 errors** (with dump and `LATEXML_NODUMP=1`).
- Distribution build (`maxperf`): ~45 MB; beats 2× pdflatex on the mini-benchmark.

## Methodology & the cortex cross-join

Working method (2026-06): **re-triage LARGE-error papers** (the single-error tail
is exhausted) → bisect the doc to the trigger line → verify Perl with `--verbose`
→ fix the divergence. Random sweeps are low-yield.

**Cortex agentic API (reads open, no token):** `http://127.0.0.1:8000/api`.
Recipe: `GET /api/reports/<corpus>/oxidized-tex-to-html/<severity>` → categories;
`…/<severity>/<category>` → per-`what`; `…/<category>/<what>` → paper list. Then
`GET /api/corpus/<corpus>/tex_to_html/document/<id>` for Perl status — a Rust-only
win is **Perl=no_problem/warning but Rust=error/fatal**. Corpus
`sandbox-arxiv-10k-shuffle`. URL-encode `\`→`%5C`, `^`→`%5E`.

**State of the autonomous methods (2026-06-21) — all tapered; a FRESH cortex
rerun is the clear next step:**
- *Stale 10k error cross-join*: **mined out** — every remaining apparent
  "Rust-only" cluster traced to a SHARED cause (third-party class/pkg neither
  engine binds; author errors; stale pre-fix run). **2026-06-21 re-check via the
  live cortex `document/<id>` API (not the stale ad-hoc join):** the last two
  candidates were BOTH phantom — `1308.2655` "Extra alignment tab" on
  `\lefteqn`/`\multicolumn{N>cols}` is **parity** (Perl 1 error, Rust 1 error —
  Perl's `nextColumn` errors on column overflow too, `Alignment.pm:136-144`); and
  `0710.5692` `equationgroup isn't allowed in <ltx:p>` is **parity** (Perl 2,
  Rust 2). An ad-hoc same-tree cross-join had falsely reported both as "Perl 0";
  the stable cortex DB is authoritative. **Lesson: confirm every cross-join
  "Rust-only" read against the live cortex `document/<id>` API before chasing —
  do not trust a bespoke join's Perl column.** (One genuine *minor* residual on
  `0710.5692`: Rust reports the equationgroup location as `Anonymous String` vs
  Perl's `cosmo_sing_iwa.tex; line 1124` — a source-locator gap, belongs to the
  #47/#92 source-map track, NOT a parity/correctness bug.)
- *Diagnostic-message faithfulness*: **exhausted** — a systematic batch
  comparison (undefined CS/env, missing-number, group/mode close, malformed,
  close-environment) shows all primary messages matching Perl.
- *Structural-skeleton diff on Perl-clean papers* (the silent-divergence method
  that found the REVTeX/OmniBus `\references` fix): now consistently surfaces
  only the DEFERRED families — MathFork/content-MathML (`equation > tags`) and
  document-builder block/paragraph auto-wrap — plus cosmetic/niche cases.
- *Binding-completeness set-diff*: too noisy to be useful — it misses every
  macro defined via `TeX!(r"…")` raw-TeX blocks (single-backslash), so its
  flagged "gaps" are mostly false positives (verified: longtable `\LTcapwidth`
  etc. ARE defined). OmniBus was confirmed structurally complete this way.

**NEXT: a FRESH cortex Rust rerun built from this branch** (needs
`X-Cortex-Token`) is the prerequisite for mining genuine Rust-only *correctness*
wins now that the diagnostic messages are faithful; always re-confirm a flagged
paper on the CURRENT binary before chasing it. Otherwise, the highest-value work
is the DEFERRED focused sessions below (content-MathML, document-builder).

> **2026-06-21 update — reruns IN PROGRESS, first cortex cross-check done.** A
> fresh Rust rerun (`019eea79…`) AND a fresh Perl rerun (started 03:51) are both
> live on `sandbox-arxiv-10k-shuffle`, so per-paper status is in flux (many show
> transient `todo`). A first cortex-grounded cross-check of the **`error/malformed`
> tail** (the richest vein for Rust-only document-builder bugs) — filtered to
> papers where BOTH services are terminal AND Perl lacks the exact `what` —
> surfaced **zero genuine Rust-only structural regressions**. Every apparent
> candidate is either still `todo` in the Perl rerun, or a paper where **Rust is
> at-or-better than Perl**: e.g. `0905.3143` Perl 101 errors→FATAL vs Rust 6
> errors/no-fatal; `1710.08311` Perl FATAL vs Rust survives. (Method script
> pattern: `reports/.../error/malformed/<what>` → per-paper
> `corpus/<c>/tex_to_html/document/<id>`, require Perl status ∈ terminal AND no
> `malformed/<what>` message.) **Re-run the clean full cross-join once both reruns
> COMPLETE** — only then is a Perl=`no_problem`/`warning` vs Rust=`error` signal
> trustworthy.

> **2026-06-21 (later) — reruns now COMPLETE; cross-join reopened.** Rust service
> `oxidized-tex-to-html` on `sandbox-arxiv-10k-shuffle` is 100 % terminal
> (todo=0); Perl `tex_to_html` is 99.77 % terminal (23/9849 `todo`). The
> small-category sweep (xpath/document/misdefined, fully enumerated + per-paper
> cross-checked against the live `document/<id>` API) found:
> - **`1506.09203` — STALE signal, already FIXED on current HEAD.** The cortex
>   DB shows Perl=`warning`, Rust=`error` (`error|xpath|findnodes|()` at
>   `xml.rs:46`), but that Rust status is from the rerun binary `019eea79`. A
>   local repro on current HEAD (`/data/arxiv/1506/1506.09203/`,
>   `Subrepresentation_book_6tag3.tex`, TCI/Scientific-Word + `tcilatex.tex`,
>   ar5iv profile) converts **clean: 0 errors / 0 fatals, no xpath failure, 52
>   warnings** — matching Perl. An intervening branch commit (after the rerun
>   snapshot) resolved the eqnarray/MathFork `findnodes` invalid-context failure.
>   **Lesson reaffirmed: always re-confirm a flagged paper on the CURRENT binary
>   before chasing.** Landed regardless: `xml.rs` `findnodes`/`findvalues` now
>   include the failing XPath string + context-node presence in the error (the
>   old message was just `{:?}` → empty `()`), so any future xpath failure is
>   diagnosable.
> - `0803.1344` (document/open_element_internal): Perl `fatal` vs Rust `error` →
>   Rust at-or-better, not a regression.
> - `1608.07271`, `1802.04240` (misdefined `#`), `hep-th9207093`
>   (misdefined `\list`): Perl=`error` = Rust=`error` → parity (shared cause).

**Beyond-parity coverage candidates (#2 track, surpass-Perl — defer while
strict-parity is #1):** `arximspdf`/`imsart` support (16+ IMS papers aop/aos;
needs a bundled imsart.sty since the host lacks it); `jpconf` class → iopart
(18+ IOP-conf papers); theorem/mdframed-in-figure schema (`figure_mixed_content`,
Open task §1).

---

## Math-parser / content-MathML gaps — DEFERRED to a dedicated session

> **User directive 2026-06-20: defer ALL content-MathML items to a dedicated
> session** (the math parser is a full Marpa-vs-RecDescent rewrite; these touch
> the parse-tree / content-MathML structure and want a focused regression
> budget). Notes kept here; do NOT pick at them piecemeal.

- **`f(a,b)` multi-arg flattening** (HIGH value, central/risky): a
  function/opfunction applied to a paren comma-list wraps it as one `vector@`
  arg (`\max(a,b)`→`maximum@(vector@(a,b))`) vs Perl flat `maximum@(a,b)`.
  Affects all multivariate calls. Perl `ApplyDelimited`/`extract_separators`
  drops the commas and puts items DIRECTLY as the operator's args. Rust path:
  `(a,b)`→`lparen formula_list rparen => fenced` (vector), then
  `function fenced_factor => prefix_apply` (wrap). Fix = add
  `function/opfunction/trigfunction lparen formula_list rparen => apply_delimited`
  FLATTEN rules + pruning to prefer the flat apply. Pruning-sensitive; full-suite
  + Perl cross-check required (see builder.rs:699-704).
- **`f(x)` single-arg apply-vs-multiply** (most PERVASIVE divergence): for an
  UNKNOWN/undeclared symbol + paren arg, Rust reads *application*, Perl reads
  *multiplication* — `\Gamma(s)`→Rust `Gamma@(s)` vs Perl `Gamma * s` (likewise
  `\zeta(s)`, `\Phi(x)`, `f(x)`). A real fix must respect Perl's "only declared
  FUNCTION/known-operator names apply; bare letters multiply" rule; heavily
  pruning-sensitive.
- **`[a|b]` / `[a \mid b]` bracket-conditional**: unparsed in Rust; Perl
  `delimited-[]@(conditional@(a,b))` (e.g. `E[X|Y]`). Rust has both pieces
  (`a|b`→`conditional@`, `[x]`→`delimited-[]@`) but they don't compose — the bare
  vertbar conditional sits at `statements` level, not `expression`. Fix = a
  surgical `lbracket … singlevertbar … rbracket` rule, OR lift the
  vertbar-conditional to `expression` (NOT a plain fence rule — Perl wraps it).
- **`⁡` DecorateOperator over-insertion** (presentation): Rust's blanket
  `parser.rs:711-743` post-walk decorates ALL operator-base SCRIPTOP applies, so
  presentation emits `⁡` (U+2061) where Perl juxtaposes — even unscripted
  `\nabla \phi` (`∇⁡ϕ` vs `∇ϕ`). content `text=` already matches. Fix = make the
  walk selective like Perl's `addOpDecoration` (drop OPERATOR/DIFFOP).
- **wide-space PUNCT XMDual content-arm XMRef ordering**: `x^2\quad y` — the
  `\quad` (≥10pt) becomes a virtual PUNCT through `formulae_apply`, producing an
  XMDual whose content-arm XMRef siblings emit one slot off from Perl. Same
  MathFork/split content-arm xml:id family as the `expected:id` tail
  (`EXPECTED_ID_XMREF_DESIGN_2026-06-08.md`). NOT the rpadding path (thin spaces
  `\,` are Perl-faithful incl. NewScript transfer, `005716ff66`).
- **`\DeclareMathOperator` cluster** (`text=` already matches): (a) Perl splits
  Math attrs `tex="\operatorname{Tr}…"` vs `content-tex="\Tr…"` (via
  `revert_as=>'context'`); Rust keeps `\Tr` in `tex`, no `content-tex`. (b) Rust
  drops the `name="Tr"` Perl infers from the CS. (c) `\DeclareMathOperator*`
  limit operators: Perl base carries `scriptpos="mid"` (`\argmax_x` subscript
  BELOW); Rust's XMDual base loses scriptpos → defaults to `post1` (right).
- **N-ary bare-operator listing** (content-loss already FIXED `a75fbf17ed`):
  `\[ + - \times \div \]` → Perl `list@(+,-,*,/)`; Rust now marks unparsed with
  ALL tokens preserved (the coverage guard rejects the exhausted-early prefix
  parse). Remaining = the N-ary upgrade: `anyop anyop` → recursive
  `compound_operator_2` list (its own `// TODO`). Ambiguity-sensitive. (Root
  cause was the marpa fork's `Parser::read` breaking on `is_exhausted()` before
  the token source drained — `marpa/src/parser/mod.rs:130`.)
- **relation with a list-RHS that itself contains a scripted relop**:
  `a \le b \quad \stackrel{?}{\ge} \quad c` → Perl `a <= list@(b, >=^?, c)`, Rust
  unparsed. The scripted-relop atomic fix (`4a5ebf29f7`) cleared standalone list
  items but not a relop-item inside a relation's list-RHS.
- **`\underset`/`\overset` over an ARROW with a multi-token script**:
  `x \underset{n\to\infty}{\to} y` — the under-script reads `n@to@infinity`
  (apply) where Perl groups `(n to infinity)`. Same ARROW-as-applied-function
  family as `f(a,b)`.

CAUTION: new VERTBAR/fence grammar rules can collide with package-built
structures — always cross-check the affected fixture against Perl before
assuming a regression (the norm rule "regressed" physics_test, but Perl matched
the new output, so it was a parity *fix*).

## DefMathRewrite `\WildCard` subscript bug (focused-session item)

`DefMathRewrite` with a `\WildCard` SUBSCRIPT pattern doesn't demote the match
(witness `math/simplemath`): `f_\WildCard → role=ID` should make `f_1(a+b)` =
`f _ 1 * (a+b)` (Perl), but Rust produces `Unknown@() * (a + b)` — the
`f_\WildCard` rewrite isn't firing (or loses to the sibling `f → FUNCTION`
rewrite), so `f_1` stays a FUNCTION and gets APPLIED. The non-wildcard
`f_D → DIFFOP` works, so it's the `_\WildCard`-subscript match/ordering in
`latexml_package/.../latexml_sty.rs` (`compile_declare_pattern`). Niche
(binding-author feature, rare in real arXiv); the fixture encodes the buggy
output.

---

## Open tasks (actionable)

### 1. `ERROR_DEBT` test-gate drain
The harness error-gate (`latexml_oxide/src/util/test.rs`) fails a test at zero
debt to force removal once fixed. Remaining:
- **`figure_mixed_content`** — `ltx:theorem` not allowed in `ltx:figure` (Perl
  also errors 1). True fix = **schema expansion** (theorems/mdframed in figures).

### 2. `\gls`/`\acrshort` in MATH mode (1705.10306) — suspected Rust gap, UNVERIFIED vs Perl
293 errors `ltx:XMTok isn't allowed in <ltx:glossaryref>` (the "Perl 1" figure is
**unverifiable** — 1705.10306 is in NO cortex corpus and Perl 0.8.8 times out on
glossaries on this host, so it cannot be cross-checked; treat as suspected, not
confirmed): a glossary
command in math mode forces the `glossaryref` content (#3, the link display
text) as math → bare `<XMTok>`, which `Inline.model` rejects. **Diagnosis
re-narrowed 2026-06-21** (earlier "document-builder / Math-not-auto-openable"
theory DISPROVEN): on the SAME host tree the current binary is **byte-identical
to Perl** for `\textbf`/`\emph`/`\href` in math (general math-in-text is
faithful); `ltx:Math`/`ltx:XMath` are **not** autoOpen in either engine (so no
auto-open path), and `ltx:glossaryref` has **no** autoClose in either (faithful,
so it can't float its content out like `emph` does). Most likely root: Perl's
**raw-loaded `glossaries.sty`** typesets the term as TEXT (`\glstextformat`/
`\mbox`), so Perl's #3 is PCDATA — the Rust divergence is in the raw-load
display chain, **not** the document builder. **STILL BLOCKED** on a runnable
Perl reference: glossaries times out in Perl 0.8.8 on this host (datatool/
l3regex) even without `\makeglossaries`; the `glossary.{tex,xml}` fixture has no
math case; witness 1705.10306 is not in the local corpus. Repro + full notes:
`docs/reproducers/glossaryref_math_xmtok.tex`.

### 3. PR #248 B1 — re-entrant `&mut Document` UB (runtime-bindings), accepted caveat
The Rhai constructor trampoline re-mints `&mut Document` (Stacked/Tree-Borrows UB
under a re-entrant `\wrap{\myemph{..}}`). Consolidated to one audited
`script_bindings/mod.rs::with_doc` site + documented; the review's checked-guard
fix **deadlocks** `Document::absorb`. **Optional future work:** make re-entrancy
sound-while-succeeding (interior-mutable `Document` or a core handle around
`do_absorption`). Not a blocker; `runtime-bindings` stays on by default.

### 4. 0.7.0 release — release-prep LANDED; tag pending
Version bumped, `runtime-bindings` in the artifact, `.deb` deps, CHANGELOG/README
done. **Remaining:** tag `0.7.0` on `main` → `release.yml` runs the TL-window
`dumps` + macOS arm64 leg + publish (each first-exercised on that tag).

---

## Deep deferred families (parked — large or shared; dedicated sessions)

- **1610.00974 step-3** — port the *global* `p{}` column to the Perl VBox form
  (`\lx@tabular@p`/VBoxContents). The narrow `\multicolumn{}{p{}}` case is fixed;
  the global port exposes a `\cr`-mid-VBoxContents-predigest interleaving + a
  span/sizing bug on `\multicolumn` over p-columns. Also explains the p-column
  `td align="justify"` + width-on-`<p>` divergence (Perl: `align="left"` +
  width-on-`<inline-block>`). Surpass-Perl R&D. **Related residuals catalogued
  with minimal reproducers in `docs/reproducers/array_pcolumn/`** (Kind B: `>{}`
  prefix align not on `<td>`; Kind C/D: regular `m{}`/`b{}` use a plain
  `\vtop{}`/`\vbox{}` not the `\lx@tabular@p` VBox → width-on-`<td>` + inline-block
  `vattach`/width drift). The `\multicolumn`-over-`m{}`/`b{}` GROUP ERROR in that
  family is now **FIXED** (was 1805.01525 27→0; `tex_tables.rs`
  `\lx@alignment@multicolumn` generalized from p{}-only to all paragraph columns).
- **`expected:id` cmml dangling-XMRef tail** — MathFork/split content-arm xml:id
  duplication; the last live `expected:id` class. See
  `EXPECTED_ID_XMREF_DESIGN_2026-06-08.md`.
- **xy-pic `svg:path` / curve cluster** (1501.03690) — shifted-arrows `svg:path`
  in `ltx:text`; mode-frame cascade root.

**SHARED (both engines fail — match Perl; do NOT "fix" by downgrading):**
- **1804.01117 xint raw-load** — both raw-load xint and fail (plain: both stub,
  byte-identical). The Rust stack-overflow crash is FIXED (gullet `stack_guard`,
  configurable via `latexml_core::stack_guard`). Deep xint emulation parked.
- **mode-frame auto-close cluster** (1611.04940, 2009.05630, 1702.06692,
  1702.02037) — a theorem env opened via its bare begin-command with no matching
  `\end…` leaks the mode-switch frame; Perl `Stomach.pm:343-376` errors
  identically. A graceful auto-close would *surpass* Perl (beyond-parity R&D).

---

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
    Rust is error-clean and schema-valid, so this is a COSMETIC structural
    divergence, not a validity bug. Same root: Perl's builder opens a block for
    a horizontal run inside a block-context element; Rust doesn't.
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
- **BibTeX**: `BibTeX.pool.ltxml` ported (Phases 1–8; remaining B1–B6 polish in
  `BIBTEX_PORT_PLAN.md`). `--nobibtex` is opt-out, not default.

### Tikz known diffs vs Perl
`foreignObject` transform; arrow-tip path data; SVG viewBox/width; matrix
`<svg:g class="ltx_tikzmatrix">` vs inline-blocks; **bare `svg:g` in `<ltx:block>`**
(tikz-cd) trips a core-XML validity error but post-processing recovers (witness
2006.12702) — Rust-only, low priority (output recovered).

### Graphics renderer chain (subprocess-only; LANDED)
PDF→PNG `mutool draw`→`pdftocairo`→`convert+gs`; PDF→SVG `mutool convert`→
`pdftocairo`→`inkscape`. Subprocess `exec` (no GPL linking). Apt: `poppler-utils`
(req), `mupdf-tools` (rec), `imagemagick+ghostscript`, `inkscape`.

### Other tracks (separate docs)
- Performance: `PERFORMANCE.md` (P1 math/large-doc open; P2 allocation partial).
- Release gates: `RELEASE_CRITERIA.md`. Releasing: `RELEASING.md`.
- Completed missions (archived): strict-LoadFormat dump parity, Marpa ASF
  migration, distribution-readiness, the 500K/1M warning-corpus mission, and the
  diagnostic-message faithfulness pass (2026-06-20) — see `docs/archive/` and
  `git log`.
