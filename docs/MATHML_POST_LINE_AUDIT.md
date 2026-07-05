# MathML post-processors — line audit (exhaustive-port verification)

> **Living worklist** (opened 2026-07-02, user-commissioned: "the Rust
> translation wants to be an exhaustive port"). Walk every Perl sub AND every
> `DefMathML` registration in the MathML post-processing stack, verdict each
> against the Rust port, and — critically — verify the **wiring** (producer →
> consumer chains), not just function existence. When complete: date + move to
> `docs/archive/`, lifting residuals into `SYNC_STATUS.md`.
>
> **Status 2026-07-02:** full sweep complete (3 parallel readers over
> MathML.pm's 60 subs, the 197 `DefMathML` registrations, and the four sibling
> files). Verdicts below are the sweep's output; each fix re-verifies its item
> against the Perl source before landing (the sweep itself already caught one
> self-error: it called `fmt_em` a match — it isn't, see F4). Fixes land as
> individual commits with a witness formula vs same-host Perl MathML.

## Motivation (the charter bugs)

The astro-ph0001001 `S9.Ex4.m1` lost-spaces bug (fixed 2026-07-02,
`3ab9ce3cb3`) required BOTH halves of a wiring gap: Perl MathML.pm L344-348
(attach `lpadding`/`rpadding` → `_lpadding`/`_rpadding`) was never ported —
the consumer existed with no producer — AND the live `adjust_pair` dropped
Perl's author-spacing term from `target` and could not materialize onto an
invisible operator. Neither is findable by name-matching functions. The same
sweep found the systemic cause: **`mod.rs` carries a parallel, fuller set of
ports (`stylize_content`, `pmml_maybe_resize`, `needs_mathstyle`, a whole
spacewalk) that the live `presentation.rs` pipeline never calls** — the
thinner inline versions are what actually run.

## Method

- Perl side: `LaTeXML/lib/LaTeXML/Post/MathML.pm` (60 subs + 197 `DefMathML`
  registrations), `MathML/{Presentation,Content,OperatorDictionary,
  Linebreaker}.pm`. (`MathProcessor.pm` does not exist as a file in our Perl
  snapshot; its role is covered by the base-class code in MathML.pm/Post.pm →
  Rust `math_processor.rs` trait.)
- Rust side: `latexml_post/src/mathml/{presentation,mod,content,
  operator_dictionary,linebreaker}.rs`. Live pipeline:
  `mod.rs::convert_node` → `presentation.rs::convert_to_pmml` /
  `content.rs::convert_to_cmml`.
- Verdicts: **PORTED** (where) / **PARTIAL** (what's missing) / **MISSING** /
  **N-A** (infrastructure Perl-ism with an idiomatic replacement) /
  **DEAD** (Rust code exists but is unreachable from the live pipeline).
- Witness protocol: same-host Perl via `latexmlc --format=html5` (segfaults at
  exit-cleanup on this host AFTER writing complete output — exit 139 with a
  complete file is not a failure) or `/usr/local/bin/latexml | latexmlpost`.
  CAVEAT: the installed Perl is 0.8.8 and LAGS the `LaTeXML/` reference tree —
  cross-check any witness delta against the reference sources before treating
  it as a Rust gap (worked example: `\fbox{$x$}` — 0.8.8 loses the frame, the
  reference tree emits `enclose='box'` exactly like Rust; parity confirmed at
  the consumer).

## Scope ruling: line-breaking is OPT-IN in Perl (default-parity)

Perl's whole linebreaking stack (`preprocess_linebreaking`, `convertNode_linebreak`,
Linebreaker.pm's 58 subs, the MathFork/MathBranch document rewrap) activates
**only** when `--linelength=n` is passed (`Presentation.pm` L21/L58 gate on
`$$self{linelength}`, default `undef` — `latexmlpost` L39). Neither ar5iv nor
cortex set it, and our CLI does not expose it. So the Rust linebreaker gap is a
**feature gap** (like DTD support), NOT a production-output divergence.
Consequences recorded here, deprioritized behind the live-path findings:
- `latexml_post/src/mathml/linebreaker.rs` is a from-scratch sketch (~10% of
  the Perl algorithm), uses the `mspace linebreak=newline` strategy Perl
  explicitly **abandoned** (live Perl builds an `mtable`), and is **dead**
  (`best_fit_to_width`/`apply_layout` have zero callers; the
  `with_linebreaking` flag is never read).
- Decision needed at release time: either port Linebreaker.pm faithfully
  behind a `--linelength` option, or delete `linebreaker.rs` and document the
  feature as out of scope. Tracked as **F5**.

## Internal-attribute wiring table (the bug class)

| attr | Perl producer | Perl consumer | Rust producer | Rust consumer | status |
|---|---|---|---|---|---|
| `_role` | stylizeContent + pmml L354-355 | adjust_pair atom types | token path + pmml wrapper (all results) | get_node_role | ✅ wired (wrapper half landed 2026-07-02) |
| `_lspace`/`_rspace` | stylizeContent (opdict) | adjust_pair defaults | presentation.rs (opdict) | get_node_attr_f64 | ✅ wired |
| `_lpadding`/`_rpadding` | pmml L344-348 (sums refr+node) | adjust_pair L1228-9 | pmml wrapper `attach_source_padding` | adjust_pair | ✅ FIXED `3ab9ce3cb3`; residual: Perl sums the **referring XMRef's** padding too (F13) |
| `_largeop` | stylizeContent L811-820 | needsMathstyle | mo styler (SUMOP/INTOP) | needs_mathstyle (live, F7) | ✅ wired 2026-07-02 |
| `_ignorable` | stylizeContent (zero-width Hints) | filter_row L577 | ⚠️ Hint path doesn't mark (F11) | no filter_row in live pmml_row (F11) | ⬜ both halves unwired |
| cleanup | (Perl arrays never serialize `_`-keys) | | `clean_internal_attrs` | | ✅ |

## Findings (fix queue — ranked by production impact)

Verified-and-landed items move to the ✅ list at the bottom.

- **F8 residuals — token styler** (bulk + F8b landed, see ✅): still missing
  vs Perl `stylizeContent`: `color='red'` on unknown empty tokens (L713),
  Perl's `name||meaning` fallback order (Rust: meaning||name),
  `force_lspace`/`force_rspace` beyond the ZWSP case, and the SIZE/
  DESIRED_SIZE context half (deliberately unported — our engine stamps
  absolute fontsize on tokens, the mathsize context gate compensates; a
  faithful port would regress script sizing). The dead, fuller
  `mod.rs::stylize_content` remains to reconcile/delete (used by mod.rs
  text paths only).
- **F19 (NEW) — math-parser mid-script mis-parse.** `x\mathrel{\mathop{=}
  \limits^{def}}y` parses in Perl as a script application (`XMApp(
  SUPERSCRIPTOP, =, def)` → `m:mover`) but in Rust as a failed parse
  (`XMWrap[BIGOP =, XMApp role=POSTSUPERSCRIPT]` → empty-base `m:msup`).
  NOT a post-processing gap (each post side renders its input faithfully) —
  belongs on the math-parser worklist.
- **F13 residual** — through an XMRef the wrapper attrs apply by recursion
  (inner=target, outer=refr): equivalent to Perl's `_getattr` refr-preference
  for `_role` (outer overwrites) and padding (sums), near-equivalent for
  class (order reversed); sole corner: refr AND target both carrying
  `enclose` nest two menclose where Perl picks one. Accepted; revisit only on
  a witness.
- **F14 residual** — `m:share` hrefs use the primary ID suffix ('');
  parallel-markup secondary-suffix wiring (Perl `$MATHPROCESSOR->IDSuffix`)
  still to connect. `cmml_not` (L1406) has no callers among the ported
  registrations — N-A unless a key needing it shows up.
- **F20 (NEW, engine) — witnesses to re-run after binding fixes.** The cfrac
  investigation showed the deltas were ENGINE-side (amsmath binding), fixed
  2026-07-02: the fused `\cfrac` constructor hardcoded mathstyle='display',
  never emitted `name='cfrac-inline'`, and compounded nested sizes to 50%.
  Lesson: witness diffs must be traced to producer vs consumer before
  patching the post side.

- **F17 — smaller pmml gaps:** `pmml_infix` ADDOP flatten via `pmml_unrow`
  (L639-644) absent; `pmml_scriptsize_padded` (L926, primed-sum limit
  centering) + `pmml_script_decipher` emb_left/emb_right absent;
  `pmml_text_aux` text nodes skip `stylizeContent` (font/color lost on mtext,
  L1034) and element attrs not propagated (L1040-1044); `Apply:?:formulae`
  has no pmml arm (renders the phantom op); `pmml_parenthesize` no
  `usemfenced` branch (obsolete `m:mfenced` — probably N-A, confirm);
  `outerWrapper` missing altimg/RDFa attrs (L82-89); `combineParallel`
  missing `annotation-xml` non-mathml wrap (L123-127); `preprocess` doesn't
  wire plane1/hackplane1/nestmath config (L69-73).

- **F4 residual** — `annotated` mspace width `0.389em` vs Perl's raw
  `0.3888888888888889em` (fmt_em itself is now byte-parity, see ✅).
- **F5 — Linebreaker feature gap** (see Scope ruling above). Decide: faithful
  port behind `--linelength`, or delete the dead sketch + document as out of
  scope. Sub-findings if ported: mtable strategy (not mspace-newline),
  multiplex/breakstepper enumeration, height/depth model, scriptlevel-scaled
  width estimation, fence resize, `lhs_pos` alignment, punctuation
  extraction, `isFence`/`isSeparator` attribute overrides.

### Landed

- **F8b ✅ (2026-07-02)** — inherited-context bindings: CTX_FONT/COLOR/
  BGCOLOR/OPACITY thread-locals bound in `convert_to_pmml` from the XMath
  ancestor chain (Perl pmml_top L278-285), rebound per node down the
  recursion (pmml L332-335), consumed in the token path (color→mathcolor,
  context-only mathbackground mirroring Perl's L683 `&&` quirk,
  opacity→style, font fallback) and the sqrt/mroot color fallbacks.
  **CORRECTION (2026-07-05 commit review):** the original commit `ff87a841e5`
  message claimed the ENCLOSE/FRACOP color fallbacks too, but its diff only
  touched sqrt/mroot + the token path — the FRACOP/ENCLOSE hunks were lost
  when the edit script aborted mid-run and the stale comments masked the gap
  (the lone witness has no fraction/enclose, so it couldn't expose it). Those
  two arms actually landed later in `cb1ad27a61` (PR_READINESS must-fix 2),
  caught by the branch's live A/B. Net state is correct; the claim was ahead
  of the code by one commit. Perl's Format-char styling suppression (L747-748)
  added so
  invisible operators stay unstyled. Witness `{\color{red}$a+b$}`: every
  token now `mathcolor="#FF0000"` byte-identical (was: colored math rendered
  black — a visible arXiv bug class). Bonus: fixed a latent FFI crash in
  `find_inherited_attribute` (walking up to the Document node read a
  misaligned ns field — the known rust-libxml class; element-type guard,
  Perl-faithful).

- **F15 ✅ (2026-07-02)** — `do_cfrac` ported in full (denominator-sum last-
  term pull-up: trailing `\cdots`, nested-cfrac recursion, invisible-times
  `\cdots`·factor; Perl L1930-1951) behind the faithful `cfrac-inline` gate
  (only that variant unrolls; display cfrac stays a plain recursive mfrac —
  the sweep overstated this). PLUS the amsmath `\cfrac` binding rewritten to
  Perl's trampoline structure (capture surrounding mathstyle once, \let-
  rebind for nested reuse, args digest under the captured style, name from
  CFRACSTYLE). Witness (nested cfrac inline/display/\cfracstyle{inline}):
  structures byte-identical; sole residual delta is the DOCUMENTED \cdots
  ELIDEOP-vs-ID divergence (OXIDIZED_DESIGN).

- **F3+F6+F11 ✅ (`856de84a10`)** — spacewalk rewritten as Perl's stream
  algorithm on child-index paths: mrow unwinding + script-base streaming
  into the pair stream, embellished dict-spacing reads from the INNER
  operator, all adjust_pair branches (negative-target mpadded rewrap via
  `compute_size` string metrics incl. the min-10pt mathscript hack, mspace
  width-merge, both-mo target/2 split with raw Perl number concat, fudge
  Info); Hint widths normalized to em, zero-width Hints `_ignorable` +
  `filter_row` in rows and array cells. Witnesses 26/28 byte-identical
  (residuals: Perl-0.8.8 version artifact + F19).
- **F14 ✅ (bulk, 2026-07-02)** — content-MathML structural parity:
  `multirelation` → pairwise applies chained under `m:and` with `m:share`
  (cmml_shared generates xml:id/fragid like Perl generateNodeID; xml:id read
  via the namespaced accessor — WISDOM trap); `less-than-or-approximately-
  equals` → `m:or` composition; `cmml_leaf` ci content now STYLIZED (font →
  mathvariant → plane1: `<ci>𝑥</ci>`, `𝐯`, `ℒ` — formerly raw ASCII on every
  identifier, the largest cmml delta); `cmml_decoratedSymbol` meaning→csymbol
  branch + pmml-subtree ci content; Perl-regex integer test for `m:cn`
  (arbitrary length, not i64). Witness a<b<c / \mathbf{v} / \mathcal{L}:
  byte-identical incl. share hrefs.
- **F16 ✅ (2026-07-02)** — OperatorDictionary `$Content_form` + fence tables
  REGENERATED verbatim from the Perl range strings (machine-parsed, sorted,
  non-overlap asserted) replacing the hand-written if-chains: closes the Cat
  A dingbat/arrow holes, Cat B negation holes, the U+2A50 B-vs-C
  misclassification, fence `U+0331`, and the boundary over-inclusions.
  Regression tests pin 2A50/27A1/0331.

- **F10+F12+F13 ✅ (`8074ef8e0a`)** — pmml-wrapper parity (menclose wrap from
  source `enclose`, class merge, `_role` recording) + dedicated Apply:ENCLOSE
  arm (`\cancel` → menclose updiagonalstrike) + FRACOP verbatim linethickness
  /mathcolor/bevelled + root mathcolor. Witness inventory byte-identical.
- **F7 ✅ (2026-07-02)** — full mathstyle→`m:mstyle` propagation: `%stylemap`/
  `%stylemap2` transition tables, corrected `needsMathstyle` (mfrac/_largeop/
  displaystyle-shield), XMApp wrap (dispatch extracted, style switched
  around it), XMArray wrap + mtable `displaystyle="true"` in display context,
  `pmml_bigop` wrap for SUMOP/INTOP/BIGOP tokens, script inner-base
  displaystyle wrap, AND the mode-sensitive entry baseline (display math →
  Display, inline → Text — Perl convertNode L20-21; Rust formerly always
  started at Display). Witness: `\tfrac`/`\dfrac`/`\displaystyle\sum`/
  smallmatrix all byte-identical to Perl.
- **F8 ✅ (bulk, 2026-07-02)** — faithful mo styler: opdict xor-emission
  (stretchy/fence/separator/largeop), `_largeop` for needsMathstyle,
  `symmetric='true'`, `movablelimits='false'` for mid-position bigops
  (`∑`,`lim`), size resolution (context gate + script rescale + %→em) with
  the minsize/maxsize stretchyhack for symmetric-wanting delimiters
  (`\bigl(` → `minsize/maxsize="1.200em"`), and mathsize for ALL token
  types (smallmatrix cells `0.700em`). Residuals above.
- **F9 ✅ (2026-07-02)** — `pmml_maybe_resize` ported (with the XMDual
  parent-attr fallback the dead copy lacked) and wired at all five Perl call
  sites: XMWrap/XMArg, XMApp (before mstyle), XMArray (after mstyle), XMText,
  and every token. Dead mod.rs copy deleted. Witness: `\raisebox` →
  `mpadded voffset="4.3pt"` byte-identical.
- **F4 ✅ (2026-07-02)** — `fmt_em` now byte-matches Perl `sprintf("%.3fem")`
  (trailing zeros kept: `1.200em`); residual: the `annotated` constant above.
- **F18 ✅ (2026-07-02)** — `nth-root` arg-order bug: XMath args are
  (degree, radicand) in BOTH engines, but all three Rust consumers had them
  swapped — presentation rendered `<mroot>` spec-backwards (degree as base,
  radicand shrunk), content emitted `<degree>` around the radicand,
  unicode_math picked the radicand as the root index. The sweep's
  "self-consistent, don't fix" verdict was WRONG (double-swap ≠ identity
  across three different consumers). Witness `\sqrt[3]{x}`: pmml AND cmml now
  byte-identical to Perl.
- **F1 ✅ (`3ab9ce3cb3`)** — `_lpadding`/`_rpadding` producer + author-spacing
  term + invisop materialization (astro-ph0001001 witness).
- **F2 ✅ (this commit)** — dead duplicate spacewalk deleted from `mod.rs`
  (`adjust_spacing`/`space_walk`/`AtomType`/`role_to_atom_type`/
  `tag_to_atom_type`/`atom_pair_spacing`/`TEX_SPACING`/`fmt_em` + helpers).
  Before deletion, all three tables were verified entry-for-entry against
  Perl `$role_atomtype`/`$atompair_spacing`/`%m_atomtype` (L1150-1218) AND
  against the live presentation.rs copies — full three-way match, including
  Perl's deliberate `mfrac→Ord` deviation from Knuth. Table-parity tests
  moved onto the live copies in presentation.rs. NOTE: `mod.rs` still holds
  live helpers (`pmml_row`, `pmml_parenthesize`, `get_xm_hint_spacing`,
  `find_inherited_attribute`, `stylize_content`…) — only the spacewalk
  cluster was dead. Remaining suspected-dead mod.rs items are audit items,
  not deleted blind: `style_step`, `style_size`, `pmml_tag_for_role`,
  `needs_mathstyle` (→F7), `pmml_maybe_resize` (→F9), `apply_handler_for_meaning`,
  `cmml_element_for_meaning`, `has_dedicated_cmml_structure`, `pmml_punctuate`
  (dead in Perl too, "never used?"), `pmml_unrow` (→F17 ADDOP flatten).

## MathML.pm named subs (60) — sweep verdicts

| Perl sub (line) | verdict | notes |
|---|---|---|
| `preprocess` (L66) | PARTIAL | plane1/hackplane1/nestmath config unwired (→F17) |
| `outerWrapper` (L77) | PARTIAL | altimg/imagesrc/valign + RDFa attrs missing (→F17) |
| `rawIDSuffix` (L109) | PORTED | mod.rs `raw_id_suffix` |
| `combineParallel` (L113) | PARTIAL | `annotation-xml` non-mathml wrap missing (→F17) |
| `getQName` (L147) | N-A | document.rs qname helpers |
| `addCrossref` (L154) | PORTED | crossref.rs |
| `realize` (L163) | PORTED | inlined idref resolution at call sites |
| `getOperatorRole` (L173) | PORTED | presentation.rs embellished-role recursion |
| `DefMathML` (L199) | N-A | registry → match-arm dispatch |
| `lookupPresenter` (L205) | N-A | presentation.rs match |
| `lookupContent` (L212) | N-A | content.rs match |
| `pmml_top` (L273) | PARTIAL | FONT/SIZE/COLOR/… context bindings not bound (→F8 interplay) |
| `find_inherited_attribute` (L291) | PORTED | mod.rs |
| `pmml_smaller` (L303) | PORTED | presentation.rs |
| `pmml_scriptsize` (L311) | PORTED | presentation.rs |
| `pmml` (L318) | PARTIAL | enclose/class/_role refr wiring missing (→F13); padding half FIXED (F1) |
| `first_element` (L359) | N-A | libxml |
| `_getattr` (L367) | MISSING | refr-preferred attr read (→F13) |
| `_getspace` (L371) | PARTIAL | refr's own padding not summed (→F13) |
| `getXMHintSpacing` (L380) | PORTED | mod.rs `get_xm_hint_spacing` (but Hint arm bypasses it →F11) |
| `pmml_internal` (L387) | PARTIAL | mstyle wraps (→F7), maybe_resize (→F9), XMArray span/border/thead, nestmath, ltx:ERROR (→F17) |
| `needsMathstyle` (L512) | DEAD+PARTIAL | mod.rs, uncalled; missing mfrac/mstyle branches (→F7) |
| `pmml_maybe_resize` (L525) | DEAD | mod.rs port unwired (→F9) |
| `filter_row` (L577) | MISSING | `_ignorable` drop (→F11) |
| `pmml_row` (L581) | PARTIAL | no filter_row (→F11) |
| `pmml_unrow` (L586) | DEAD | needed by ADDOP flatten (→F17) |
| `pmml_parenthesize` (L594) | PARTIAL | no usemfenced branch; no synthesized OPEN/CLOSE mo (→F17) |
| `pmml_punctuate` (L611) | N-A | dead in Perl too ("never used?") |
| `pmml_infix` (L626) | PARTIAL | ADDOP flatten missing (→F17) |
| `stylizeContent` (L672) | PARTIAL | live inline is thin; full dead copy in mod.rs (→F8) |
| `pmml_mi/mn/mo` (L830-845) | PARTIAL | no maybe_resize wrap (→F9) |
| `pmml_bigop` (L847) | MISSING | no mstyle wrap (→F7) |
| `pmml_script` (L876) | PARTIAL | innerbase mstyle wrap (→F7) |
| `pmml_script_mid_layout` (L893) | PARTIAL | NOMOVABLELIMITS + phantom padding (→F17) |
| `pmml_scriptsize_padded` (L926) | MISSING | primed-sum limit centering (→F17) |
| `pmml_script_multi_layout` (L936) | PARTIAL | empty slot `m:none` vs Perl empty mrow (→F17) |
| `pmml_script_decipher` (L963) | PARTIAL | emb_left/emb_right + prelevel logic (→F17) |
| `pmml_text_aux` (L1029) | PARTIAL | text-node styling + attr propagation (→F17) |
| `adjust_spacing` (L1079) | PORTED | presentation.rs |
| `space_walk` (L1096) | PARTIAL? | verify mrow-unwrap parity (→F6) |
| `compute_size` (L1135) | MISSING | needed by F3 mpadded branch |
| `adjust_pair` (L1220) | PARTIAL | mpadded/mspace/target÷2 branches (→F3) |
| `fmt_em` (L1285) | PARTIAL | trailing-zero trim divergence (→F4) |
| `cmml_top` (L1290) | PORTED | content.rs |
| `cmml` (L1301) | PORTED | + Rust-only cycle/depth guard |
| `cmml_internal` (L1311) | PARTIAL | meaning-vs-role dispatch nuance |
| `cmml_contents` (L1350) | PORTED | |
| `cmml_unparsed` (L1360) | PORTED | |
| `cmml_leaf` (L1377) | PARTIAL | mathvariant prefix on m:ci (→F14) |
| `cmml_decoratedSymbol` (L1396) | PARTIAL | meaning→csymbol + pmml content (→F14) |
| `cmml_not` (L1406) | MISSING | (→F14) |
| `cmml_synth_not` (L1410) | PARTIAL | inlined for one caller |
| `cmml_synth_complement` (L1415) | PORTED | inlined |
| `cmml_shared`/`cmml_share` (L1420-1434) | MISSING | m:share (→F14) |
| `cmml_or_compose` (L1436) | MISSING | (→F14) |
| `pmml_summation` (L1796) | PARTIAL | Rust adds ⁡ when base≠mo — verify vs Perl |
| `do_cfrac` (L1931) | MISSING | cfrac unrolling (→F15) |

## DefMathML registrations (197) — sweep verdicts

Dispatch: pmml via `pmml_apply`/`pmml_token` match arms; cmml via `cmml_impl`/
`cmml_leaf` + the `meaning_to_cmml_element` table. Bulk verdict: **the long
tail is PORTED** — all Token meaning→cmml entries (trig/hyperbolic/inverse ×28,
arithmetic ×27, relations ×9, sets ×10, calculus ×11, statistics ×6, linear
algebra ×6, constants ×15 — including Perl's preserved `hyperbolic-cotantent`
typo), interval/set/list/vector/cases/matrix constructors, accents
(OVER/UNDER incl. nesting), scripts, infix/postfix roles, `hack-definite-
integral`, `not-approximately-equals`/complement synthesis. Exceptions (all
folded into the Findings): ENCLOSE (F10), FRACOP/root mathcolor+bevelled
(F12), multirelation + lt-or-approx cmml (F14), continued-fraction (F15),
formulae pmml arm (F17), Hint width normalization (F11), `Token:?:absent`
empty `m:mi` vs `m:mrow` (documented divergence, Task #264), `m:cn` integer
detection `i64`-parse vs Perl regex (huge ints/leading `+` — micro-gap),
nth-root arg order reversed on BOTH producer+consumers (self-consistent, final
markup matches — do not "fix" one side alone).

## Sibling files — sweep verdicts

**Presentation.pm:** `convertNode_simple`/`rawIDSuffix`/`canConvert`-adjacent
flow PORTED; `associateNodeHook` (href/title) relocated to token build-time —
equivalent for same-node association; everything linebreaking-related is the
F5 feature gap; `convertNode`'s `converted_pmml_cache` + MathBranch branch are
linebreaking-only (F5).

**Content.pm:** all 3 subs PORTED (`convert_to_cmml`, `.cmml` suffix,
`can_convert` gating on `math_is_parsed`).

**OperatorDictionary.pm:** structure + all 14 category property sets PORTED
(spot-check of 10 diverse operators: lspace/rspace/properties identical);
data holes in the Cat A/B long tail + U+2A50 misclass + fence `U+0331` → F16.

**Linebreaker.pm:** F5 in full (dead sketch, wrong strategy, ~10% algorithm).
