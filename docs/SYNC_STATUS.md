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
- *fatal/TooManyErrors mining (2026-06-22)*: **mined out — ZERO genuine
  Rust-only bugs.** Of 35 `MaxLimit(100)` papers: 24 Perl-fatal (parity), **9 a
  `cp1251`/Cyrillic env artifact** (all `[cp1251]{inputenc}`+`[T2A]{fontenc}`+
  russian babel → ~100 `unexpected:<char>` each; the `cyrillic`/`t2` TeX package
  is missing on this host so `cp1251.def`/`t2aenc.def` are absent — **local Perl
  fails identically**, the cortex Perl=clean came from a host WITH the package),
  2 stale/marginal. Same env-artifact family as the isolatin phantom. **Cyrillic
  coverage fix is host-side (`tlmgr install cyrillic cm-super`), not a code bug;
  an optional surpass-Perl charset-decode fallback for missing inputenc `.def`s
  would convert them without the host package (needs authorization).**
- *fatal/Timeout mining (2026-06-22)*: 18 papers → 16 Perl-fatal (parity), 2
  candidates. `1506.09195` = missing custom `my_paper.sty` + deep expl3/datatool/
  l3fp (local Perl also fatals; Rust runs the conditional runaway to the IfLimit
  guard). **`1707.02464` = the ONE genuine Rust-only bug from all 53 fatal papers:
  Perl completes in 11.76s, Rust hangs to the 60s watchdog** — a custom
  `\narrow` macro's `\hsize`-shrink loop never terminates because Rust's vbox
  `\ht` is `\hsize`-invariant (Perl models paragraph height ∝ `\hsize`). Recorded
  as `STABILITY_WITNESSES.md` Cluster G (open; box-model fix, regression-risky,
  warrants a focused session).
- *error-severity sweep (2026-06-22)*: full cross-join of the cortex `error`
  severity (1189 tasks) on the **same local host** (env-artifact discipline).
  **Parity/env-artifact dominated; ONE genuine Rust-only correctness bug.**
  - `malformed` (162): all parity except **`ltx:itemize` in a `p{}` cell** — the
    p{}-block-content bug (1510.07685), root = **1610.00974 step-3**, now
    **✅ FIXED 2026-06-22** (`f65b80c1c2`, the p{}→VBox port, unblocked by the
    Cluster G box-model fix `7545e07fd6`). `_CaptureBlock_`/listing errors are
    Perl-identical (parity).
  - `latex` (31): all parity. Every package `\PackageError` (`\GenericError`,
    `(ifthen)`, `(newunicodechar)` 189, `(etoolbox)` 187, `(glossaries)` 224,
    `(pgfkeys)`) is shared. The `(babel)` `Unknown option 'russian'`/`'ukrainian'`
    cluster (11 papers, cortex Perl=warning) is a **babel-VERSION env artifact**:
    local babel.sty ≥3.9 (locale-based) errors on the `russian` *option*
    (`russianb.ldf` absent), and **local Perl emits the IDENTICAL single error**
    (0709.3796: Rust==Perl==1). The cortex Perl=warning host had pre-3.9 babel.
    Same class as the isolatin/cp1251 phantoms; not a code bug (a `babel_lang_stubs`
    russian/ukrainian stub would surpass local-Perl + overlap the Cyrillic
    host-side decision → left as-is).
  - `missing_file` (31), `misdefined` (3), `document` (2), `xpath` (2): all parity.
  - `undefined` (890): top-20 whats all parity — the `imsart` bib cluster
    (`\bauthor`/`\bfnm`/`\btitle`/… + `{barticle}`, 16 papers) and `{diagram}`
    (17/19) are **Perl-also-undefined** (Perl LaTeXML ships no imsart/diagram
    binding either). Confirms "undefined = shared third-party CS".
  - `unexpected` (268): the big "Script `_`/`^` can only appear in math mode" +
    "Misplaced alignment tab `&`" clusters are **100% parity** under a FULLY
    PAGINATED cross-join (`_` 109/109, `^` 45/45, `&` 51/51 papers — no math-mode
    detection divergence; these are genuinely-malformed unescaped inputs both
    engines flag). The only "candidates" were the `<char>` inputenc Cyrillic/latin
    env-artifact cluster (0802.1123 isolatin, 1008.0492/1011.5076 babel-russian,
    1009.2998 `[cp866]`+`[T2A]` — host missing the `.def`; same class as Clusters
    A/C/E) and `\end{table}`/1805.00875 (**already FIXED** — see next).
  - **META (2026-06-22): the cortex Rust service data is STALE** (predates recent
    branch fixes). 1805.00875 (dcolumn) shows `unexpected/\end{table}` in the
    cortex report but converts **0 errors on the current binary** (the 2026-06-21
    dcolumn fix is in). So a flagged "Rust-only candidate" may already be fixed —
    **always re-confirm on the current binary** (the genuine finds 1510.07685 /
    1707.02464 were). A **fresh cortex Rust rerun built from this branch** is the
    real prerequisite for surfacing NEW genuine Rust-only correctness bugs; the
    stale data is still authoritative for *parity* and *env-artifact* classes
    (those don't change). **Conclusion: the entire `error` severity is mined out —
    parity + env-artifacts; the one genuine find (p{} block content, 1510.07685) is
    now ✅ FIXED (1610.00974 step-3 port + Cluster G box-model fix, 2026-06-22).**

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

> **2026-06-21 (later still) — the existing rerun (`019eea79`) is now STALE; a
> NEW rerun is required before further mining.** The Rust `oxidized-tex-to-html`
> error data predates this session's fixes (m{}/b{} `\multicolumn`, dcolumn
> empty-todelim, the over-parse/grammar work, etc.), so per-`what` mining keeps
> surfacing already-fixed leads. This iteration checked the highest-cascade
> `error/latex` clusters and ALL were stale/parity/Perl-worse on the CURRENT
> binary: `(newunicodechar)` 1704.05587 (cortex "ASCII character requested" ×63 →
> now PARITY: `\newunicodechar` simply undefined in both, 22=22 identical);
> `(etoolbox)` 1604.02419 (cortex Rust=error but Perl=**fatal** → Rust at-or-
> better); `(babel)` `Unknown option 'russian'` ×11 (witness 0709.3796 now
> Rust=0=Perl=0; minimal `[russian]{babel}` is Rust=1 / Perl=3, the option error
> emitted by BOTH → parity-or-better). **Do not mine `019eea79` further — request
> a fresh Rust rerun on current HEAD first** (needs `X-Cortex-Token`); only then
> is a Perl=clean / Rust=error signal trustworthy. Reliable interim method: a
> direct LOCAL both-engines diff on a small paper sample (ground truth, not the
> stale DB).
>
> **`1506.03557` (`ESSS_2015.tex`) — Rust 49 / Perl 2, PARTIALLY addressed
> (math session, 2026-06-21).** Two distinct roots:
> - **WIDE_PUNCT threshold — FIXED.** A fenced comma-list with an interword
>   control space `\ ` before a signed term (`(3,\ -5)`, `(300,\ -50,\ +50)`,
>   `\textit{Held\_For}\;(300,\ -50,\ +50)`) fell to `ltx_math_unparsed`: the `\ `
>   put 5.0pt `rpadding` on the comma, and `punct_followed_by_wide_space`'s ≥5pt
>   threshold mis-tagged it `WIDE_PUNCT` (a `\quad`-class formula-separator routed
>   through `formulae_apply`, which fails inside a fence). Raised the threshold to
>   ≥10pt (only `\quad`+; matches `filter_hints`). Now parses, matches Perl
>   `vector@(300,-50,+50)`. Regression test in `parse/sequences_and_lists`.
> - **The 42× `XMWrap isn't allowed in <ltx:p>` residual is a WRAPPING leak
>   triggered by the `program` package — ROOT LOCALIZED 2026-06-21, still OPEN
>   (niche, deferred).** Bisection: the 42 leaks come from 3 sections
>   (preliminaries=18, trip_sealin=12, pushbutton=12), and preamble bisection pins
>   the enabling factor to **`\usepackage{program}`** (commenting it → 0 leaks).
>   `program.sty` makes `_`/`;`/`` ` `` ACTIVE in math (`\catcode\_=\active
>   \def_{\ifmmode\sb\else\p@sb\fi}`, lines 535/67-75) and redefines `\(`; the
>   preliminaries math is subscript-heavy (`t_n`, `t_{now}`, …), so under the
>   active-`_` Rust produces unparsed inline math whose bare `<XMWrap>` leaks into
>   `<ltx:p>` while Perl (which has NO program.sty.ltxml — it raw-loads) keeps it
>   `<Math>`-wrapped. Rust loads `program` via the **contrib binding**
>   (`latexml_contrib/src/program_sty.rs`), so the divergence is contrib-binding
>   vs Perl-raw-load. NOT reproducible from `program` + the snippet alone — needs
>   the full preliminaries context (accumulated state). Both the unparsed Z-math
>   AND the leak are recovered in the final output; these are build-time errors.
>   Niche (`program` is rare on arXiv); for a future contrib-binding session —
>   fix in `program_sty.rs` (match Perl's raw-load active-`_` behavior) and/or the
>   document-builder unparsed-math wrapping. The WIDE_PUNCT fix above was the
>   general, landable win from this witness. (Same scan: `1705.04022`
> 16 err `_`/`^`-in-text — re-verify vs Perl before chasing.)
>
> **`1704.05644` (`Paperling_revu.tex`) — CONFIRMED Rust-only (Rust 17 / Perl 0)
> but DEEP/tangled; deferred.** Root: `shadethm.sty` (raw-loaded, no binding in
> either engine) fails to define `\newshadetheorem` in Rust in this paper's
> context → cascade of undefined `{theorem}`/`{hyp}`/`{propgrise}` envs +
> `\shadebox*`/`\shadedtextwidth` `expected:<variable>`. KEY: the *minimal*
> `\usepackage{shadethm}\newshadetheorem{thm}{Theorem}` is **parity-broken** (BOTH
> engines: `\newshadetheorem` undefined) — so shadethm's raw-load is incompletely
> emulated in both, and only the full paper's preamble context makes Perl's
> shadethm work while Rust's still fails. Not cheaply isolatable (bisection of the
> preamble/`\input{macropulko}` did not localize a single culprit; the apparent
> "`\input` breaks it" lead was a red herring — minimal no-`\input` is equally
> broken). The `\Vertex`/gastex errors in this paper are SHARED (gastex depends on
> pstricks/pst-pdf; both engines fail identically in isolation). A proper
> `shadethm` binding (which neither engine has) would be the real fix — surpass-
> Perl R&D, not strict parity. Do not chase piecemeal.

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

- **`f(a,b)` multi-arg flattening — FIXED 2026-06-22.** A KNOWN function applied
  to a paren comma-list now flattens: `\max(a,b)`→`maximum@(a,b)` (was
  `maximum@(vector@(a,b))`), matching Perl `ApplyDelimited`/`extract_separators`.
  Implementation was simpler than the planned grammar-rule approach: a post-parse
  spread in the `prefix_apply` ACTION (`semantics.rs`, helper `vector_tuple_items`)
  — when a function-role op (FUNCTION/OPFUNCTION/TRIGFUNCTION) applies to a
  `Dual` whose content is `Apply(vector, [refs])`, spread the items as direct
  operands instead of wrapping. No grammar/pruning change → NOT pruning-sensitive,
  zero fixture regressions. Scoped to known function roles, so unknown-`f` apply
  (`f(a,b)`→`f@(vector@(a,b))`) is untouched — the intentional divergence #18.
  Verified Perl-identical: `\max(a,b)`/`\gcd(a,b)`/`\min(x,y,z)`/`g(a,b,c)` +
  nesting/`\frac`/trailing-ops; suite 1466/0; regression test in
  `parse/functions`. (Known pre-existing aside: juxtaposed `\max(a,b)\min(c,d)`
  greedily reads `\max` over the product — a separate function-juxtaposition
  pruning issue, not this flatten.)
- **`f(x)` single-arg apply-vs-multiply** (most PERVASIVE divergence): for an
  UNKNOWN/undeclared symbol + paren arg, Rust reads *application*, Perl reads
  *multiplication* — `\Gamma(s)`→Rust `Gamma@(s)` vs Perl `Gamma * s` (likewise
  `\zeta(s)`, `\Phi(x)`, `f(x)`). A real fix must respect Perl's "only declared
  FUNCTION/known-operator names apply; bare letters multiply" rule; heavily
  pruning-sensitive.
  > **SURVEY 2026-06-22 (current-state + blast radius — groundwork, NOT yet
  > changed):** confirmed the split cleanly — KNOWN functions ALREADY match Perl
  > (`\sin(x)`/`\log(x)` → `sine@(x)`/`logarithm@(x)` in both); only UNKNOWN
  > symbols diverge (`f(x)`/`g(x)`/`P(x)`/`\Gamma(s)`/`\zeta(s)`/`\phi(x)` →
  > Rust `X@(x)` vs Perl `X * x`; `f(x+1)` → Rust `f@(x+1)` vs Perl `f * (x+1)`).
  > LEXER ROLE: unknown `f` = `role="UNKNOWN"`, `\max` = `role="OPFUNCTION"` — so
  > the apply-of-UNKNOWN (A) is separable from the known-fn flatten (B). BLAST
  > RADIUS of A is corpus-wide: 25 test fixtures, ~150 single-letter applies
  > (`f@(`×57, `d@(`×51, `g@(`×13, …) would flip to multiply — a sweeping change
  > that reshapes all math output. Because A is corpus-wide (even though
  > toward-Perl), it needs explicit scope sign-off before undertaking; B (below)
  > is the contained first step (~5 fixtures).
- **`[a|b]` / `[a \mid b]` bracket-conditional — FIXED 2026-06-22.** Was unparsed
  in Rust; now `delimited-[]@(conditional@(a,b))` matching Perl (`E[X|Y]` etc.).
  Root: the bare `a|b` conditional reduces only at statement level (not as an
  `expression`), so `[a|b]` had no fence rule — though `[(a|b)]` already worked.
  Fix: a surgical grammar rule `lbracket formula singlevertbar formula rbracket =>
  bracket_conditional` (`singlevertbar` also covers `\mid`) + a `bracket_conditional`
  action (semantics.rs) that builds the inner `conditional@(a,b)` (delimiter-less
  presentation) and wraps it in `delimited-[]` via the same `fenced` path
  `[(a|b)]` uses (ctxt reborrow for the two ref levels). Suite 1466/0, clippy
  clean, zero other-fixture changes; regression test in `parse/vertbars`. (The
  `E` in `E[X|Y]` stays `E@(…)` apply vs Perl `E * …` — divergence #18, preserved.)
- **`⁡` DecorateOperator over-insertion — FIXED 2026-06-22.** Presentation MathML
  emitted `⁡` (U+2061 FUNCTION APPLICATION) after operators that render as
  `<m:mo>` — `\nabla \phi`→`∇⁡ϕ`, `\partial f`→`∂⁡f`, and (pre-existing) `\sum_i
  a_i`→`∑⁡a_i`, `\int f`→`∫⁡f` — where Perl juxtaposes (∇ϕ/∂f/∑a/∫f). Perl's rule
  (MathML.pm `Apply:?:?`): insert `⁡` only when the op base is NOT an `<m:mo>` (a
  function identifier `f`/`\sin`/`\max` IS `<m:mi>` → keeps `⁡`). FIX
  (`latexml_post/.../presentation.rs`): new `op_base_is_mo` helper (descends
  msub/msup/munder/mover to the base); applied at the generic-apply site AND in
  `pmml_summation`; and removed `DIFFOP` from the big-op→`pmml_summation` route
  (Perl MathML.pm:702 `# Not DIFFOP`). Suite 1466/0, clippy clean; verified
  Perl-identical for ∇/∂/∑/∫/∏/⋃/lim + `\sin`/`\max`/scripted forms; only residual
  diff is the `f(x)` apply-vs-multiply (`f⁡(` vs `f⁢(`) — divergence #18,
  preserved. Regression test in `tests/post/opdecoration`.
- **wide-space PUNCT XMDual content-arm XMRef ordering**: `x^2\quad y` — the
  `\quad` (≥10pt) becomes a virtual PUNCT through `formulae_apply`, producing an
  XMDual whose content-arm XMRef siblings emit one slot off from Perl. Same
  MathFork/split content-arm xml:id family as the `expected:id` tail
  (`EXPECTED_ID_XMREF_DESIGN_2026-06-08.md`). NOT the rpadding path (thin spaces
  `\,` are Perl-faithful incl. NewScript transfer, `005716ff66`).
- **`\DeclareMathOperator` cluster — INVESTIGATED 2026-06-22, LOW-VALUE metadata,
  deprioritized** (`text=` and cMML already match): (a) Perl splits Math attrs
  `tex="\operatorname{Tr}…"` vs `content-tex="\Tr…"` (Perl defines `\Tr` *via*
  `Invocation(\operatorname,…)` + `revert_as=>'context'`); Rust defines it
  directly so `tex` keeps the user macro `\Tr` (arguably MORE source-faithful) and
  emits no `content-tex`. Matching Perl needs the deep `revert_as=>context`
  content-tex mechanism — high effort, metadata-only value. (b) The `name="Tr"`
  "gap" is NOT a bug: `def_math` (dialect.rs:1567) DOES infer `name` from the CS
  but DROPS it when `name == presentation` (line ~33) — a deliberate
  redundant-attr cleanup. `\Tr` (name "Tr" == content "Tr") drops it; `\argmax`
  (name ≠ "arg max") keeps it. Perl always emits it. Changing this touches the
  GENERAL def_math path (every math token) for cosmetic value → not worth it.
  (c) `\DeclareMathOperator*` `scriptpos` in display mode — the remaining
  candidate if revisited, but mode-dependent and niche. Whole cluster parked.
- **N-ary bare-operator listing** (content-loss already FIXED `a75fbf17ed`):
  `\[ + - \times \div \]` → Perl `list@(+,-,*,/)`; Rust now marks unparsed with
  ALL tokens preserved (the coverage guard rejects the exhausted-early prefix
  parse). Remaining = the N-ary upgrade: `anyop anyop` → recursive
  `compound_operator_2` list (its own `// TODO`). Ambiguity-sensitive. (Root
  cause was the marpa fork's `Parser::read` breaking on `is_exhausted()` before
  the token source drained — `marpa/src/parser/mod.rs:130`.)
- **comma-list LEFT of a relation `a,b \in A` — FIXED 2026-06-22 (2-item path).**
  Was the wrong `formulae@(a, b∈A)` (∈ binding only `b`). Now the user-specified
  surpass-Perl **XMDual**: content **DISTRIBUTES** — `formulae@(∈(a,A), ∈(b,A))`,
  sharing XMRefs to the relop and RHS — presentation wraps the list as the
  relation's LHS — `Apply(∈, XMWrap(a,',',b), A)`. Implemented as a scoped
  transform at the end of `formulae_apply` (semantics.rs): when `left` is a bare
  (non-relational, non-Dual) item and `right` is a binary RELOP relation
  `Apply(R,[lhs,rhs])` under a comma, `distribute_list_relation` builds the dual.
  `x,y \le z`→`formulae@(x≤z, y≤z)`. The list-RIGHT `0<x,y`→`list@(0<x,y)`,
  all-relational `a=b,c=d`→`formulae@`, and bare `a,b`→`list@` all stay. Full suite
  1466/0, clippy clean, zero other-fixture changes; regression test in
  `parse/relations`. **Remaining (follow-up):** the 3+-item `a,b,c \in S` goes
  through `list_apply` (not `formulae_apply`) → still `list@(a,b,c∈S)`; the same
  distribution needs porting to that path.
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

### 5. Post-processing logging parity for cortex workflows — LANDED 2026-06-22
`cortex.log` now carries core **+** post (Scan→…→Graphics→MathML→XSLT). Shared
wrapper `latexml::post::run_post_processing_logged() -> PostOutcome {html, log,
status_code}` re-binds `LOG_BUFFER` around `run_post_processing` then flushes
(Perl's single post-`convert_post` flush); post failures use `post_error()` →
buffer + Error counter (no `Fatal!`/`return`). `latexml_oxide` `--log`/archive-zip
wired via `writer::write_output_segments` (no third concat alloc). Commits
`512dbc1ba2`, `9524d2e179`; suite 1467/0. **Residual (cortex-side owner):** wire
`cortex_worker.rs::convert_archive` to the same wrapper + fold
`max(core, post.status_code)` into `Status:conversion` (Perl `LaTeXML.pm` L631-634).

### 6. Graphics: never ship a raw `.eps`/`.pdf` to the web — LANDED 2026-06-22
THREE `latexml_post/src/graphics.rs` guards so a non-web-native source can never
reach the web: (1) conversion-failure → `imageprocessing` Error + NO imagesrc
(Perl L324-329); (2) `Plan::NotFound` → Warn only, no imagesrc (Perl L216-219);
(3) plan-routing default → web-native target (svg/png/gif/jpg/jpeg kept, else
`png`) so non-web-native always routes through `Plan::Convert` (surpass-Perl: Perl
would `Plan::Copy` the raw source). A `<graphics>` without `@imagesrc` renders
`ltx_missing_image`, never a broken `.eps` src. Verified: hep-th0101114 6/6,
astro-ph0004105 15/15 EPS→PNG, zero raw srcs. Commits `80b4438385`, `604951c232`.
Post-orchestration matches Perl `convert_post`/`Config.pm` at defaults
(Split→Scan→Index→Bib→CrossRef→Graphics→pmml/cmml→XSLT). Known deltas (broader
parity, not blocking): `PictureImages` absent (Rust = regex inline-SVG); `SVG` is a
regex extractor (intentional divergence); no `prescan`; Graphics unconditional vs
Perl `dographics`.

---

## Deep deferred families (parked — large or shared; dedicated sessions)

- **1610.00974 step-3 — global p{} → Perl `\lx@tabular@p` VBox form — ✅ LANDED
  2026-06-22 (`f65b80c1c2`).** The global `p{Dimension}` column now uses Perl's
  `\lx@tabular@p t {width} { … }` (cell = `<ltx:inline-block>`, VBoxContents /
  internal_vertical) instead of the old `\vtop{\hbox to <w>..}`;
  `\lx@alignment@multicolumn` splices directly for the already-VBox-shaped column
  (array.sty m{}/b{} keep the `\vtop`/`\vbox` transform). Unblocked by the box-model
  fix (`7545e07fd6`, Cluster G). **Fixes the genuine Rust-only correctness bug
  1510.07685** (`\begin{itemize}` in a `p{}` cell → 3→0 errors; the cell is now an
  inline-block, not an `_noautoclose` `ltx:p`). rotfloat2 sidewaystable is now
  near-Perl-exact (innerheight 69.1→98.6 vs Perl 98.5). All p{}-table fixtures moved
  TOWARD-or-equal Perl (colortbls 73→41, tabular 39→21, graphrot 125→75,
  alignment/array 18→14 diff lines vs local Perl; cells 72→72 — same cluster-B
  family) and were re-baselined; suite 1467/0, clippy clean. The narrow
  `\multicolumn{}{p{}}` and `\multicolumn`-over-`m{}`/`b{}` GROUP ERROR were already
  fixed (1805.01525 27→0). **array.sty `m{}`/`b{}` ALSO ported to `\lx@tabular@p`
  (`eb978df5a9`)** — cluster-B residual C/D (m/b `vattach`/width drift, width on
  `<td>`) CLEARED: the m-cell `<inline-block>` is Perl-exact. **Cluster-B Kind-B
  CLEARED too (`1867f17da9`):** a p/m/b `<td>` now gets `align="left"` (Perl) not
  `"justify"` — Justify mapped to `"left"` at the td cell-attr output only, keeping
  the `Cell.align == Justify` marker intact for `is_pcol` detection. **Cluster-B is
  now FULLY RESOLVED.** Every paragraph-column fixture moved toward Perl, several to
  near-exact (array_newline_math 2→0 Perl-exact, array 14→2, tabular 21→3,
  colortbls 41→9, graphrot 75→27, cells 72→64). The p/m/b table-column parity arc
  (box-model → p{} port → m/b port → td-align) is COMPLETE. **Validated
  regression-free 2026-06-22**: re-ran 12 table-structure-stressed arXiv papers
  (cortex `\@end@tabular`/`\lx@begin@alignment` clusters) on the current binary —
  **0 regression-signatures** (no `\lx@tabular@p`/itemize-in-p/inline-block errors);
  the errors that remain are all pre-existing/shared (math-mode `^`/`_`,
  `\noalign`/`&` alignment, frontmatter mode-close, undefined third-party CS like
  `collcell`'s `\collectcell`). **`collcell` checked 2026-06-22 → PARITY, NOT a
  Rust bug:** both Rust AND Perl error `\collectcell`/`\endcollectcell` undefined +
  `missing file[collcell.sty]` (both default `notex=1` / `INCLUDE_STYLES=false`, so
  neither raw-loads `collcell.sty`; Rust mirrors Perl Package.pm:2676-2677 exactly).
  The first Perl run *looked* clean ("0 errors") only because `latexml --quiet`
  SUPPRESSES the error display — `latexml` (verbose) shows the same 2 errors. So the
  1901.10277 `collcell`→alignment cascade is shared; binding collcell would
  *surpass* Perl. (Same `--quiet`-suppression artifact as babel-russian.)
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
