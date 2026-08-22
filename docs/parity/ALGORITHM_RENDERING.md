# Algorithm rendering — worklist & unification plan

**Mission.** Make LaTeXML-oxide's algorithm listings match the **pdflatex golden**
(the oracle — *not* Perl LaTeXML, which shares several of the bugs). Triggered by
arXiv/html_feedback "algorithm"-labelled reports; the class is commonly reported.
Ground rules: work from the pdflatex golden ([`OXIDIZED_DESIGN.md`] guiding
principles); read each package's own docs for full construct coverage; a **surpass**
over Perl is documented in `OXIDIZED_DESIGN_DIVERGENCES.md` + `KNOWN_PERL_ERRORS.md`
and filed upstream at `brucemiller/LaTeXML`.

## Status by package

| Package | State |
|---|---|
| **algorithmicx / algpseudocode** (`ltx_float_algorithm`) | **CLOSED.** ar5iv-css #51 (`white-space:nowrap`). |
| **algorithm2e** (`ltx_algorithm`) | `\lIf`/`\lElse` merge, line numbering, indentation, and ruled/boxed body frames all fixed. Open: `\Comment*[r]` inline comment position, ruled caption-at-top, `\ref`-to-line counter, markup unification. |

## Completed (landed; see the cited entries for mechanism — not restated here)

- **`\lIf`/`\lElse` no longer merge** — the break lives inside `\@endalgocfline`
  (Perl's binding, not the raw sty); `\lx@strippar` restored in
  `\algocf@Vline`/`Vsline`/`Noline`; blank-line drop ignores `<rule>` too. Faithful
  fix (GENUINE-RUST-ONLY: the port had regressed it). `algorithm2e_sty.rs`.
- **Line numbering** — faithful engine `\everypar` on paragraph-start (tex.web
  `new_graf`), guarded body-only; per-line `leave_horizontal_internal` seam;
  indentation moved to an endline DOM-prepend. `\Comment*[r]` statements are
  numbered; KwInOut headers stay unnumbered. **Surpass.**
  → OXIDIZED_DESIGN_DIVERGENCES #147, KNOWN_PERL_ERRORS #105.
  Guard `50_structure::algorithm2e_linenumbers_test`.
- **Ruled/boxed body frames** — `add_float_frames` skips `<ltx:tags>` so the inner
  frame lands on the real body (was silently dropped onto attribute-less `<tags>`
  in both engines); algorithm2e `\algocf@style` dispatch extended to the ruled
  family. Reach: all framed floats. **Surpass.**
  → OXIDIZED_DESIGN_DIVERGENCES #148, KNOWN_PERL_ERRORS #106.
  Guard `50_structure::algorithm2e_frames_test`.
- **`\fname@<type>` internal** — `\floatname`/`\newfloat` now define real float.sty's
  `\fname@<type>` (LaTeXML reimplemented float.sty with `\lx@name@<type>` only), so
  the `breakablealgorithm` recipe compiles instead of leaking raw `\fname@algorithm`.
  **Surpass.** → OXIDIZED_DESIGN_DIVERGENCES #149, KNOWN_PERL_ERRORS #107.
  Guard `50_structure::float_fname_internal_test`. Witness arXiv 2408.07803
  (html_feedback #1998).

All surpass entries carry an **"Upstream: to be filed at brucemiller/LaTeXML"** note —
filing is a manual step (cannot be done from the conversion tooling).

## Open follow-ups

All are **SHARED** with Perl (Perl does not achieve them either) — each a surpass, not
a regression.

1. **`\Comment*[r]` inline-right comment position.** The statement is numbered (the
   hard part); the comment still falls to its own line because the side-comment path
   emits `\@endalgocfline\ ` (raw sty L2073, non-alt r/l branch) and our
   `\@endalgocfline` BREAKS (needed for `\lIf`/`\lElse`, which call it directly). To
   keep the comment inline the statement-terminator must be non-breaking ONLY in the
   side-comment path — but that path is inside `\SetKwComment`'s generated
   `\algocf@<c>@star` macro, not cleanly interceptable, and `altsidecomment` is also
   false for `\lIf`/`\lElse` so it cannot gate `\@endalgocfline`. Options: re-implement
   `\SetKwComment` in the binding to use `\algocf@endline` (raw `;`, non-breaking) +
   let `\algocf@scpar`'s `\par` do the break; OR a distinct `\if@lx@algo@sidecomment`
   flag set by overriding a side-comment-only presentation hook. Intricate.
2. **AlgoLine counter over-steps** so a `\ref`/`\lnl` to a line reads high (e.g. a
   `\Comment*[r]` example pre-renumber = 2,3,4,5,7). `renumber_algo_lines` fixes the
   VISIBLE tags to `1..N` but not the counter. The extra `\nl` fires come from
   NON-content hmode entries — the `[H]` float placement at the start, and box/glue
   triggers (`\unhbox`, the endline-indentation `\hskip`) — each enters horizontal
   mode with `everyparnl=\nl` and steps AlgoLine without producing a kept numbered
   line. Fix needs firing `\everypar` only for GENUINE paragraph content
   (letter/other/math), not box/glue placement — a trigger-catcode gate in
   `enter_horizontal`/`fire_everypar`, done carefully so it does not break legitimate
   cases. Until then `renumber` stays and `\ref`-to-line is the residual.
3. **Ruled caption-at-top.** `[ruled]`/`[boxruled]` place the caption at the TOP
   inside the top rule; we emit it at the bottom (standard float caption). A separate
   DOM/caption-position change. Check whether Perl shares this first (it does).
4. **Side-by-side algorithm minipages** (witness arXiv 2402.19043, html_feedback
   #2282). Two `\noindent\begin{minipage}{0.48\textwidth}…\hfill…\begin{minipage}`
   blocks following intro text with NO blank line become `inline-logical-block`
   minipages INSIDE the preceding `<p>`, flowing inline with the paragraph text
   instead of a block-level side-by-side row. **Perl emits byte-identical core XML** —
   this is a CSS/HTML layout-fidelity concern (inline-block minipage flow after text),
   NOT a core-XML divergence, and improving it is a non-trivial shared-layout surpass
   touching general minipage rendering.
5. **algorithm2e line-number font is not uniform** (Rust-only). The pdflatex golden
   renders every line number in one `\NlSty` = `\textnormal{\textbf{…}}` upright-bold
   style; ours makes a number bold only when its line leads with a bold algorithm2e
   keyword (`\For`/`\If`/`\While`… via `\KwSty`), plain otherwise — the number inherits
   the ambient font. Side-effect of the §4 content-start `\everypar` numbering (Perl
   emits the number at end-of-line = neutral font → uniform). Entangled with the
   counter over-step above and the real number-emission path (the tag does NOT flow
   through the binding's `\algocf@printnl` — verified by probe), so a correct fix needs
   that path mapped. RED/GREEN guard (currently `#[ignore]`d):
   `06_cluster_regressions::cluster_algorithm2e_uniform_line_number_font`. Group with the
   counter over-step for the 0.7.7 everypar-timing pass.

**FIXED this pass (were open items):**
- Raw-loaded `algpseudocodex` emitted spurious empty `<equation/>` blocks (2 per
  `\State $math$ \Comment{…}` line) that blew out algorithm vertical spacing (witness
  2511.21969). Pruned via `Tag!("ltx:equation", after_close_late …)` (drop equations with
  no Math) — KNOWN_PERL_ERRORS #108, guard
  `06_cluster_regressions::cluster_algpseudocodex_no_spurious_empty_equation`.
- `\tabto` (tabto-ltx, `\RequirePackage`d by algpseudocodex) had no binding; its raw `$$`
  line-measurement hack broke the right-justified `\Comment` onto its own line. Bound
  `\tabto`→`\hfill` (`tabto_sty.rs`, OXIDIZED_DESIGN #150) → comments now flush right inline.

**Open — algpseudocodex comment/line box-model polish (witness 2511.21969 Alg 1/2).**
Against the pdflatex golden (compact; each `▷` comment paired with its `◁` end-marker on
ONE line) three residual layout facets remain, all rooted in how algpseudocodex builds a
line (TikZ code-boxes + per-comment `\parbox`/minipage + `\rlap` markers):
1. **Vertical gaps** between comment/block lines — each comment sits in a ~319pt
   `ltx_inline-block ltx_minipage` whose `<p>` carries line-height/spacing; the golden is
   tight. (Not empty equations any more — those are pruned.)
2. **`◁` end-marker on its own line** — it is emitted as a `width:0.0pt`
   `ltx_align_left ltx_inline-block` (an `\rlap` overlap) that does not overlap onto the
   statement's line the way the golden pairs `▷…◁`.
3. **New scrollbars** — `nowrap` (the algorithm-listing fix) plus the flushed-right
   comments make lines wide enough to trip the pre-existing `.ltx_float_algorithm
   .ltx_listing { overflow-x:auto }`; the golden fits without scroll.
These need a focused `ar5iv.css` + algpseudocodex box-model pass (compact the comment
minipage/`<p>`; make the `\rlap` `◁` truly overlap; contain width without scroll). Deep;
group with the markup-unification work below.

## Markup unification across the algorithm bindings (plan — large, cross-binding — SCHEDULED 0.7.7)

**Scheduled for the 0.7.7 release** (deferred, user-directed 2026-08-22); tracked in
[`../SYNC_STATUS.md`](../SYNC_STATUS.md) → "Algorithm markup + CSS unification".

**Goal:** unify the markup emitted for all algorithm-related bindings (algorithmic,
algorithmicx, algorithm2e, and the language variants) onto one shared marker class,
**and derive generic CSS that works in BOTH `LaTeXML.css` and `ar5iv.css`** — one
algorithm-layout rule set, not per-theme per-package selectors. **Constraint:** listings
(`lstlisting`) is a separate, established XML dialect — do NOT change it; the algorithm
bindings *borrow* its `<ltx:listing>/<ltx:listingline>/<ltx:tags>` elements (inherited
from Perl, stays).

**Immediate driver (why the shared class, not a CSS discriminator).** The
algorithm-layout rule (`white-space:nowrap`, so the pretty-printer's newlines between a
line's number tag and its statement do not become breaks) is keyed on the wrapper
classes `.ltx_float_algorithm` / `.ltx_algorithm`. An algorithm authored OUTSIDE an
`algorithm` float — e.g. the `breakablealgorithm` recipe (bare `center` around
`\begin{algorithmic}`) — emits a bare `.ltx_listing` with NEITHER wrapper class, so it
falls through to code's `white-space:pre` and renders broken. A one-line CSS
discriminator can't fix it safely: a numbered CODE `lstlisting` shares
`.ltx_tag_listingline`/`.ltx_lst_numbers_left` (so `:has(.ltx_tag_listingline)` would
break code indentation, #6632); `minted` carries `.ltx_lstlisting` so
`.ltx_listing:not(.ltx_lstlisting)` is only *nearly* safe. Hence the robust fix is a
positive shared marker class on every algorithm listing → ONE generic rule for both
themes. Witness arXiv 2408.07803 (html_feedback #1998); issue class #6080/#6236/#5492/#3450.

**Assessment: possible, ~80% already there.** Every algorithm binding emits the shared
`<ltx:listing>/<ltx:listingline>/<ltx:tags>` vocabulary. The family reduces to **two
markup producers** (the language variants `algcompatible`/`algmatlab`/`algpascal`/
`algc` all layer on `algorithmicx`; `algpseudocode` too):

| | Producer A: algorithmic + algorithmicx family | Producer B: algorithm2e |
|---|---|---|
| float class | `ltx_float_algorithm` (algorithm float pkg `\newfloat`) | `ltx_algorithm` (own `DefEnvironment`; Perl-faithful) |
| listing class | *(none)* | `ltx_lst_numbers_left` (Perl-faithful) |
| line-number `<ltx:tags>` | **leading** (via `\lx@make@tags`) | Perl prepends (leading); the Rust port historically emitted them trailing + a CSS gutter hack — the main unification blocker |

**Target (all algorithm floats):** a shared marker class on the float, leading
line-number `<ltx:tags>`, so ONE `.ltx_algorithm` CSS rule + ONE rendering path serve
every algorithm package and the ar5iv `#51` (nowrap) + gutter rules collapse to one.

**Steps (own branch):**
1. **Tag position.** Make algorithm2e PREPEND the line-number `<ltx:tags>` into the
   listingline (match Perl `\algocf@printnl`'s floatToElement+prepend) so numbers are
   DOM-leading like algorithmicx; then DROP the `.ltx_algorithm .ltx_lst_numbers_left`
   gutter hack from `ar5iv.css`. Same DOM-prepend machinery as the endline-indentation
   fix; made natural by the content-start numbering.
2. **Shared class.** Give both float wrappers one common class (e.g. the algorithm
   float pkg adds `ltx_algorithm` alongside `ltx_float_algorithm`), keeping
   Perl-faithful class names where possible; decide against the golden.
3. **Consolidate CSS** in `ar5iv.css`: one `.ltx_algorithm` listing/listingline block
   (nowrap; gutter-free once tags lead), replacing the dual selectors.
4. **Guards.** Core-XML fixtures asserting BOTH producers emit the same
   listingline/tags shape (one algorithmicx case + one algorithm2e case, diffed).

**Risk:** two producers + algorithmicx language variants + shared CSS, each with its
own tests. Verify each producer against its pdflatex golden and the Perl oracle
before/after. Its own branch + review round.

## Reproduction method

Golden = `pdflatex` (+ `pdfcrop`/`pdftoppm`). Classify SHARED vs RUST-ONLY with
same-host Perl (`perl -I LaTeXML/blib/lib LaTeXML/blib/script/latexml --nocomments`).
A per-macro tracer: post-load `\def\@marker#1{ <#1> }` (the Perl binding's `\@marker`
debug, off by default) — how the break mechanism was cracked. **grep the installed sty
with `LC_ALL=C grep -a`** — it is ISO-8859 and a UTF-8 locale makes grep silently
return nothing. Construct battery: the algorithm2e manual's own canonical examples
(disjoint-decomposition, interval, generic `\Fn`) plus targeted `\Comment*[r]`,
side-comments, KwInOut, and algorithmicx cases.
