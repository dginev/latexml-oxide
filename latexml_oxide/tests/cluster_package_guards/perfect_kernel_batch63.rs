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
        r#"<equationgroup class="ltx_eqn_eqnarray" xml:id="S0.EGx2"><equation><MathFork><Math tex="\displaystyle\begin{array}[]{c}f\\&#10;g\end{array}" text="Array[[f], [g]]" xml:id="S0.EGx2.m2"><XMath><XMArray role="ARRAY" vattach="middle"><XMRow xml:id="S0.EGx2.m2.1"><XMCell align="center"><XMTok font="italic" role="UNKNOWN">f</XMTok></XMCell></XMRow><XMRow xml:id="S0.EGx2.m2.2"><XMCell align="center"><XMTok font="italic" role="UNKNOWN">g</XMTok></XMCell></XMRow></XMArray></XMath></Math><MathBranch><td align="right"><Math mode="inline" tex="\displaystyle\begin{array}[]{c}f\\&#10;g\end{array}" text="Array[[f], [g]]" xml:id="S0.EGx2.m1"><XMath><XMArray role="ARRAY" vattach="middle"><XMRow xml:id="S0.EGx2.m1.1"><XMCell align="center"><XMTok font="italic" role="UNKNOWN">f</XMTok></XMCell></XMRow><XMRow xml:id="S0.EGx2.m1.2"><XMCell align="center"><XMTok font="italic" role="UNKNOWN">g</XMTok></XMCell></XMRow></XMArray></XMath></Math></td></MathBranch></MathFork></equation></equationgroup>"#,
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
