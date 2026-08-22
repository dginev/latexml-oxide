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
  → OXIDIZED_DESIGN_DIVERGENCES #148, KNOWN_PERL_ERRORS #105.
  Guard `50_structure::algorithm2e_linenumbers_test`.
- **Ruled/boxed body frames** — `add_float_frames` skips `<ltx:tags>` so the inner
  frame lands on the real body (was silently dropped onto attribute-less `<tags>`
  in both engines); algorithm2e `\algocf@style` dispatch extended to the ruled
  family. Reach: all framed floats. **Surpass.**
  → OXIDIZED_DESIGN_DIVERGENCES #149, KNOWN_PERL_ERRORS #106.
  Guard `50_structure::algorithm2e_frames_test`.
- **`\fname@<type>` internal** — `\floatname`/`\newfloat` now define real float.sty's
  `\fname@<type>` (LaTeXML reimplemented float.sty with `\lx@name@<type>` only), so
  the `breakablealgorithm` recipe compiles instead of leaking raw `\fname@algorithm`.
  **Surpass.** → OXIDIZED_DESIGN_DIVERGENCES #150, KNOWN_PERL_ERRORS #107.
  Guard `50_structure::float_fname_internal_test`. Witness arXiv 2408.07803
  (html_feedback #1998).

All surpass entries carry an **"Upstream: to be filed at brucemiller/LaTeXML"** note —
filing is a manual step (cannot be done from the conversion tooling).

## Landed in Part 2 (PR feat-algorithm-rendering-part2)

- **`\Comment*[r]`/`[l]` side comment is now inline flush-right.** `\@endalgocfline`
  stays non-breaking in the side-comment path — keyed on `\ifx\\\algocf@endstartsidecomment`
  (the raw sty `\let\\` at L2073 uniquely marks that path), still breaking for
  `\lIf`/`\lElse`. `\algocf@scrfill`=`\hfill` flushes it right. Guard
  `50_structure::algorithm2e_linenumbers` (re-blessed).
- **Ruled caption-at-top.** For the ruled family, `after_construct` moves
  `<caption>`/`<toccaption>` before the body (`float_sty::reposition_caption_top`);
  `plain`/`boxed` keep it at the bottom. Surpass OXIDIZED_DESIGN #153, guard
  `06_cluster_regressions::cluster_algorithm2e_ruled_caption_at_top`.
- **Uniform bold line numbers.** `\algocf@printnl` now wraps the number in the real-sty
  `\NlSty` (`\textnormal{\textbf{…}}`), and `renumber_algo_lines` was made
  font-preserving (rewrites the inner `<text>`, not `set_content` on the tag). The old
  "tag bypasses `\algocf@printnl`" premise was WRONG — it flows through. Guard
  `06_cluster_regressions::cluster_algorithm2e_uniform_line_number_font` (un-ignored, green).
  Independent of the counter over-step.
- **Side-by-side minipages** (2402.19043) — CSS only: `.ltx_align_middle` no longer
  strips an authored minipage width (ar5iv.css, scoped `:not([style*="width"])`), so the
  two 0.48-width algorithms keep their width instead of collapsing into the paragraph.
- **algpseudocodex box-model — scrollbars mitigated** (CSS): the phantom VERTICAL
  scrollbar is gone (`.ltx_listing` `overflow-y:hidden`, both ar5iv.css + LaTeXML.css),
  and framed lstlistings are contained (`.ltx_lstlisting` overflow-x). The comment
  `<p>`-gaps and the `◁` `\rlap`-on-its-own-line still need the algpseudocodex binding
  below.
- **`\hbox to \hsize` leader separators fill the column** (`\dashfill`/`\hrulefill`/
  `\dotfill`) — relativized to `width:100%` instead of a frozen 345pt that overflowed the
  algorithm. Leader-content discriminator in `tex_box.rs`; text/fixed-width boxes untouched
  (fancyvrb 345pt parity preserved). Surpass OXIDIZED_DESIGN #152, witness 1510.02728
  ("Modified ellipsoid method" separators); guard `cluster_hbox_to_hsize_leader_fills_width`.
- **Repeated frontmatter no longer duplicates** — replaceable tags (title/abstract/…) keep
  one entry, creators still accumulate (`REPLACEABLE_FRONTMATTER_TAGS`, forward-port of
  upstream `%ReplaceableFrontmatterTags`). Surpass OXIDIZED_DESIGN #154, witnesses 2002.09766
  (appendix `\icmltitle`), 2511.21969 (nested `{abstract}`); guard
  `cluster_frontmatter_replaceable_dedup`.
- **wrapfigure/minted overlap** (2605.03143) — CSS only: a float-scoped min-width override
  stops the wrapfig flex cell forcing the code box over the body text (ar5iv.css).
- **`\For`/`\While`/`\If` bodies written with `\\` now indent under the `|`** — the algorithm
  line-break binding was clobbered by the float setup's tabular guard (`\\`→`\lx@newline`); we
  re-assert `\\`→`\lx@algo@par` after `before_float`. Surpass (SHARED, KNOWN_PERL_ERRORS #109),
  witness 2002.09766 Algorithm 1; guard `cluster_algorithm2e_for_body_indentation`.
- **Multiple full-line `\hbox to \hsize` separators stack** (follow-up to #152) — two width:100%
  leader boxes flanking a centered label overflowed a `nowrap` listingline (>200%); the fill-line
  box is now marked `ltx_leaderfill` and set `display:block` (engine + both stylesheets), so each
  separator owns its line. Witness 1510.02728.

## Open follow-ups

All are **SHARED** with Perl (Perl does not achieve them either) — each a surpass, not
a regression.

1. **AlgoLine counter over-steps** so a `\ref`/`\lnl` to a line reads high (e.g. a
   `\Comment*[r]` example pre-renumber = 2,3,4,5,7). `renumber_algo_lines` fixes the
   VISIBLE tags to `1..N` but not the counter. The extra `\nl` fires come from
   NON-content hmode entries — the `[H]` float placement at the start, and box/glue
   triggers (`\unhbox`, the endline-indentation `\hskip`) — each enters horizontal
   mode with `everyparnl=\nl` and steps AlgoLine without producing a kept numbered
   line. Fix needs firing `\everypar` only for GENUINE paragraph content
   (letter/other/math), not box/glue placement — a trigger-catcode gate in
   `enter_horizontal`/`fire_everypar`, done carefully so it does not break legitimate
   cases. **HIGH risk** (same `fire_everypar` seam as the numbering surpass + KwInOut
   unnumbered + `\Comment*[r]` numbered); deferred to its own pass. Until then
   `renumber` stays and `\ref`-to-line is the residual.
2. **algpseudocodex `\Comment`/`\Statex` binding** — there is no binding
   (`algpseudocode_sty.rs` is a stub for the algorithmicx variant; algpseudocodex — the
   distinct newer package — raw-loads), so `\Comment` runs the package's raw box TeX (a
   ~319pt `\parbox` minipage + a full-width `\rlap` rule), giving inter-line `<p>` gaps
   and the `◁` end-marker on its own line. The CSS above only mitigates the scrollbar;
   the cure is a new `algpseudocodex_sty.rs` emitting a semantic right-aligned inline
   comment that shares the statement line. Substantial new binding; deferred. Witness
   arXiv 2511.21969 Alg 1/2.

**FIXED this pass (were open items):**
- Raw-loaded `algpseudocodex` emitted spurious empty `<equation/>` blocks (2 per
  `\State $math$ \Comment{…}` line) that blew out algorithm vertical spacing (witness
  2511.21969). Pruned via `Tag!("ltx:equation", after_close_late …)` (drop equations with
  no Math) — KNOWN_PERL_ERRORS #108, guard
  `06_cluster_regressions::cluster_algpseudocodex_no_spurious_empty_equation`.
- `\tabto` (tabto-ltx, `\RequirePackage`d by algpseudocodex) had no binding; its raw `$$`
  line-measurement hack broke the right-justified `\Comment` onto its own line. Bound
  `\tabto`→`\hfill` (`tabto_sty.rs`, OXIDIZED_DESIGN #151) → comments now flush right inline.

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
