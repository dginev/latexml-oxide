//! Red/green guards for perfect-kernel phase-63 batches: run 336's `Fatal:Stomach:Recursion` cluster (arxbj
//! `{longlist}`, etex register blocks, the autoref tag's `~` in math) and the VTeX IMS markup arxbj shares with
//! arximspdf.
use latexml::util::test::assert_element;

use super::perfect_kernel_batch57::{RAW, assert_elements};

/// 63a: arxbj.cls:930-958 makes `{longlist}` a `\list` counted `longlist` and labelled "(i)", "(ii)" (:1070); the
/// binding's `\let\longlist\list` took the first `\item` as the label argument, every label `\item` again: an endless
/// recursion (1203.0186, 1003.1189). Repro list-structure/arxbj_longlist_items_are_labelled.
#[test]
fn arxbj_longlist_items_are_labelled() {
  let xml = assert_elements(
    include_str!(
      "../../../tools/perfect_kernel/repros/list-structure/arxbj_longlist_items_are_labelled.tex"
    ),
    RAW,
    (0, 0),
    &[],
  );
  assert_element(
    &xml,
    "itemize",
    &[],
    r#"<itemize><item xml:id="S0.I1.i1"><tags><tag>(i)</tag><tag role="autoref">i</tag><tag role="refnum">i</tag></tags><para xml:id="S0.I1.i1.p1"><p>first</p></para></item><item xml:id="S0.I1.i2"><tags><tag>(ii)</tag><tag role="autoref">ii</tag><tag role="refnum">ii</tag></tags><para xml:id="S0.I1.i2.p1"><p>second</p></para></item></itemize>"#,
  );
}

/// 63a: etex.sty:382-425 `\globtoksblk\prooftoks{1000}` names the first of a block of registers; undefined, the
/// "proofs at the end" idiom's `\toks\numexpr\prooftoks+\count@` left `+\count@\relax`, which reset its `\loop`
/// counter: an endless loop (1610.01929, 1801.07292; Perl too). Repro macro-state/etex_register_blocks_allocate.
#[test]
fn etex_register_blocks_allocate() {
  assert_elements(
    include_str!(
      "../../../tools/perfect_kernel/repros/macro-state/etex_register_blocks_allocate.tex"
    ),
    RAW,
    (0, 0),
    &[
      (
        "para",
        "p1",
        r#"<para xml:id="p1"><p>First proof.</p></para>"#,
      ),
      (
        "para",
        "p2",
        r#"<para xml:id="p2"><p>Block: 31768.</p></para>"#,
      ),
    ],
  );
}

/// 63a: an equation's tags are digested in its math, where vdm.sty's `\everymath{\let~\hook}` made their `~` a
/// `\vbox{\ialign…}` that re-stepped the equation: an endless recursion (1601.02132). A reference prints a tag in text,
/// so a tag's `~` is the kernel's no-break space: the autoref tag reads "Equation 1" as it does without the
/// rebinding. Repro macro-state/autoref_tag_tilde_is_the_kernel_space.
#[test]
fn autoref_tag_tilde_is_the_kernel_space() {
  let xml = assert_elements(
    include_str!(
      "../../../tools/perfect_kernel/repros/macro-state/autoref_tag_tilde_is_the_kernel_space.tex"
    ),
    RAW,
    (0, 0),
    &[
      (
        "equation",
        "S0.E1",
        r#"<equation labels="LABEL:e" xml:id="S0.E1"><tags><tag>(1)</tag><tag role="autoref">Equation 1</tag><tag role="refnum">1</tag></tags><MathFork><Math tex="\displaystyle a=b" text="a = b" xml:id="S0.E1.m4"><XMath><XMApp><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok font="italic" role="UNKNOWN">a</XMTok><XMTok font="italic" role="UNKNOWN">b</XMTok></XMApp></XMath></Math><MathBranch><tr><td align="right"><Math mode="inline" tex="\displaystyle a" text="a" xml:id="S0.E1.m1"><XMath><XMTok font="italic" role="UNKNOWN">a</XMTok></XMath></Math></td><td align="center"><Math mode="inline" tex="\displaystyle=" text="=" xml:id="S0.E1.m2"><XMath><XMTok meaning="equals" role="RELOP">=</XMTok></XMath></Math></td><td align="left"><Math mode="inline" tex="\displaystyle b" text="b" xml:id="S0.E1.m3"><XMath><XMTok font="italic" role="UNKNOWN">b</XMTok></XMath></Math></td></tr></MathBranch></MathFork></equation>"#,
      ),
      (
        "equationgroup",
        "S0.EGx2",
        r#"<equationgroup class="ltx_eqn_eqnarray" xml:id="S0.EGx2"><equation xml:id="S0.Ex1"><MathFork><Math tex="\displaystyle\begin{array}[]{c}f\\&#10;g\end{array}" text="Array[[f], [g]]" xml:id="S0.Ex1.m2"><XMath><XMArray role="ARRAY" vattach="middle"><XMRow><XMCell align="center"><XMTok font="italic" role="UNKNOWN">f</XMTok></XMCell></XMRow><XMRow><XMCell align="center"><XMTok font="italic" role="UNKNOWN">g</XMTok></XMCell></XMRow></XMArray></XMath></Math><MathBranch><td align="right"><Math mode="inline" tex="\displaystyle\begin{array}[]{c}f\\&#10;g\end{array}" text="Array[[f], [g]]" xml:id="S0.Ex1.m1"><XMath><XMArray role="ARRAY" vattach="middle"><XMRow><XMCell align="center"><XMTok font="italic" role="UNKNOWN">f</XMTok></XMCell></XMRow><XMRow><XMCell align="center"><XMTok font="italic" role="UNKNOWN">g</XMTok></XMCell></XMRow></XMArray></XMath></Math></td></MathBranch></MathFork></equation></equationgroup>"#,
      ),
    ],
  );
  // The reference the tag is printed by.
  assert_element(
    &xml,
    "p",
    &[],
    r#"<p>See <ref class="ltx_refmacro_autoref" labelref="LABEL:e" show="autoref"/>.</p>"#,
  );
}

/// 63a: arxbj.cls shares arximspdf.cls's VTeX IMS markup — the structured bibliography (:2599-2897), `{pf}`/`{pf*}`
/// with the automatic `\qed` (:1428-1445), `\tablewidth` (:1684) and `\bolds` — now one binding, `ims_support`
/// (1003.1189 17 undefined, 1203.0186 `\tablewidth`). Repro index-bib/ims_structured_bibliography_and_proofs.
#[test]
fn ims_structured_bibliography_and_proofs() {
  let xml = assert_elements(
    include_str!(
      "../../../tools/perfect_kernel/repros/index-bib/ims_structured_bibliography_and_proofs.tex"
    ),
    RAW,
    (0, 0),
    &[(
      "bibitem",
      "bib.bib1",
      r#"<bibitem key="a" xml:id="bib.bib1"><tags><tag>[1]</tag><tag role="autoref">1</tag><tag role="refnum">1</tag></tags><bibblock>Doe, J. (2001). A title. Bernoulli 7 1–2. <ref class="ltx_href" href="http://www.ams.org/mathscinet-getitem?mr=2363971">MR2363971</ref></bibblock></bibitem>"#,
    )],
  );
  // `{pf}` and `{pf*}`, each ending in the class's square.
  assert_element(
    &xml,
    "proof",
    &[],
    r#"<proof><title class="ltx_runin">Proof.</title><para xml:id="p2"><p>Obvious.<Math mode="inline" tex="\square" text="square" xml:id="p2.m1"><XMath><XMTok name="square" role="UNKNOWN">□</XMTok></XMath></Math></p></para></proof>"#,
  );
  let second = &xml[xml.find("</proof>").expect("a first proof")..];
  assert_element(
    second,
    "proof",
    &[],
    r#"<proof><title class="ltx_runin">Proof of the claim.</title><para xml:id="p3"><p>Trivial.<Math mode="inline" tex="\square" text="square" xml:id="p3.m1"><XMath><XMTok name="square" role="UNKNOWN">□</XMTok></XMath></Math></p></para></proof>"#,
  );
}

/// 63b: nameref.sty:352-359 redeclares `\ref`, `\pageref` and `\Ref` at `\begin{document}`, so a self-recursive
/// preamble `\renewcommand{\ref}` never runs (2503.08060, 1908.01329, 1811.01873; KNOWN_PERL_ERRORS #519). nameref's
/// `\NR@setref`'s selector names the field a reference prints (`\@firstoffive`: the number). Repro
/// macro-state/preamble_ref_redefinition_reset_at_begin_document.
#[test]
fn preamble_ref_redefinition_reset_at_begin_document() {
  assert_elements(
    include_str!(
      "../../../tools/perfect_kernel/repros/macro-state/preamble_ref_redefinition_reset_at_begin_document.tex"
    ),
    RAW,
    (0, 0),
    &[
      (
        "para",
        "S1.p1",
        r#"<para xml:id="S1.p1"><p>See <ref labelref="LABEL:a"/>.</p></para>"#,
      ),
      (
        "para",
        "S1.p2",
        r#"<para xml:id="S1.p2"><p>Also <ref labelref="LABEL:a"/>.</p></para>"#,
      ),
      (
        "para",
        "S1.p3",
        r#"<para xml:id="S1.p3"><p>Named <ref class="ltx_refmacro_nameref" labelref="LABEL:a" show="title"/>.</p></para>"#,
      ),
    ],
  );
}

/// 63b: nameref's reset is a `nameref`-labelled begindocument chunk, so the document's own `\AtBeginDocument` code runs
/// after it and keeps its `\pageref`/`\Ref`, while a preamble `\DeclareRobustCommand{\ref}` (its `\ref␣` too) is
/// reset (KNOWN_PERL_ERRORS #519). Repro macro-state/begin_document_ref_redefinition_survives_nameref.
#[test]
fn begin_document_ref_redefinition_survives_nameref() {
  assert_elements(
    include_str!(
      "../../../tools/perfect_kernel/repros/macro-state/begin_document_ref_redefinition_survives_nameref.tex"
    ),
    RAW,
    (0, 0),
    &[(
      "para",
      "S1.p1",
      r#"<para xml:id="S1.p1"><p>See <ref labelref="LABEL:a"/>; <ref class="ltx_refmacro_autoref" labelref="LABEL:a" show="autoref"/>; the section a.</p></para>"#,
    )],
  );
}

/// 63b: an `\halign` or `{array}` in an `{eqnarray}` row inherited `\eqnarray@row@before` and stepped the equation once
/// per inner row (1601.02132; Perl alike, KNOWN_PERL_ERRORS #518); a new alignment's rows run no enclosing row hook,
/// and the eqnarray's own later rows still do. Repro kernel-alignment/nested_alignment_inherits_eqnarray_row_hook.
#[test]
fn nested_alignment_inherits_no_eqnarray_row_hook() {
  let xml = assert_elements(
    include_str!(
      "../../../tools/perfect_kernel/repros/kernel-alignment/nested_alignment_inherits_eqnarray_row_hook.tex"
    ),
    RAW,
    (0, 0),
    &[(
      "equationgroup",
      "S0.EGx4",
      r#"<equationgroup class="ltx_eqn_eqnarray" xml:id="S0.EGx4"><equation xml:id="S0.E4"><tags><tag>(4)</tag><tag role="refnum">4</tag></tags><MathFork><Math tex="\displaystyle a=\begin{array}[]{c}x\\&#10;y\end{array}" text="a = Array[[x], [y]]" xml:id="S0.E4.m4"><XMath><XMApp><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok font="italic" role="UNKNOWN">a</XMTok><XMArray role="ARRAY" vattach="middle"><XMRow><XMCell align="center"><XMTok font="italic" role="UNKNOWN">x</XMTok></XMCell></XMRow><XMRow><XMCell align="center"><XMTok font="italic" role="UNKNOWN">y</XMTok></XMCell></XMRow></XMArray></XMApp></XMath></Math><MathBranch><tr><td align="right"><Math mode="inline" tex="\displaystyle a" text="a" xml:id="S0.E4.m1"><XMath><XMTok font="italic" role="UNKNOWN">a</XMTok></XMath></Math></td><td align="center"><Math mode="inline" tex="\displaystyle=" text="=" xml:id="S0.E4.m2"><XMath><XMTok meaning="equals" role="RELOP">=</XMTok></XMath></Math></td><td align="left"><Math mode="inline" tex="\displaystyle\begin{array}[]{c}x\\&#10;y\end{array}" text="Array[[x], [y]]" xml:id="S0.E4.m3"><XMath><XMArray role="ARRAY" vattach="middle"><XMRow><XMCell align="center"><XMTok font="italic" role="UNKNOWN">x</XMTok></XMCell></XMRow><XMRow><XMCell align="center"><XMTok font="italic" role="UNKNOWN">y</XMTok></XMCell></XMRow></XMArray></XMath></Math></td></tr></MathBranch></MathFork></equation><equation xml:id="S0.E5"><tags><tag>(5)</tag><tag role="refnum">5</tag></tags><MathFork><Math tex="\displaystyle c=d" text="c = d" xml:id="S0.E5.m4"><XMath><XMApp><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok font="italic" role="UNKNOWN">c</XMTok><XMTok font="italic" role="UNKNOWN">d</XMTok></XMApp></XMath></Math><MathBranch><tr><td align="right"><Math mode="inline" tex="\displaystyle c" text="c" xml:id="S0.E5.m1"><XMath><XMTok font="italic" role="UNKNOWN">c</XMTok></XMath></Math></td><td align="center"><Math mode="inline" tex="\displaystyle=" text="=" xml:id="S0.E5.m2"><XMath><XMTok meaning="equals" role="RELOP">=</XMTok></XMath></Math></td><td align="left"><Math mode="inline" tex="\displaystyle d" text="d" xml:id="S0.E5.m3"><XMath><XMTok font="italic" role="UNKNOWN">d</XMTok></XMath></Math></td></tr></MathBranch></MathFork></equation></equationgroup>"#,
    )],
  );
  // The four `\theequation` reports, each a whole `<p>`; the last eqnarray numbers its two rows 4 and 5.
  let afters: Vec<&str> = xml
    .match_indices("<p>After")
    .map(|(at, _)| &xml[at..at + xml[at..].find("</p>").expect("a closed paragraph") + 4])
    .collect();
  assert_eq!(afters, [
    "<p>After: 1.</p>",
    "<p>After: 2.</p>",
    "<p>After: 3.</p>",
    "<p>After: 5.</p>"
  ]);
}

/// 63b: PiCTeX's `\plot` sets each dot with its own `\raise`; the stomach's loop detector hashed a register value by its
/// kind alone, so a long line of dots read as a loop past 50,000 boxes (0801.0709, Rust only). Repro
/// boxes-groups/pictex_finite_dots_are_not_a_loop.
#[test]
fn pictex_finite_dots_are_not_a_loop() {
  let xml = assert_elements(
    include_str!(
      "../../../tools/perfect_kernel/repros/boxes-groups/pictex_finite_dots_are_not_a_loop.tex"
    ),
    RAW,
    (0, 0),
    &[],
  );
  assert!(
    xml.matches("<text").count() >= 18_000,
    "every dot of the 30 lines is kept"
  );
}
