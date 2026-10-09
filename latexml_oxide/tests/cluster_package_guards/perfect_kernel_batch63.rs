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

/// 63d: tex.web §1275 — `\openin` first closes the stream, so a name that cannot be opened leaves it closed; ours kept
/// the previous file open, and pinlabel's next figure read the last one's EPS (1010.6236; Perl alike, KNOWN_PERL_ERRORS
/// #520). Repro macro-state/openin_closes_the_stream_first.
#[test]
fn openin_closes_the_stream_first() {
  assert_elements(
    include_str!(
      "../../../tools/perfect_kernel/repros/macro-state/openin_closes_the_stream_first.tex"
    ),
    RAW,
    (0, 0),
    &[("para", "p1", r#"<para xml:id="p1"><p>CLOSED</p></para>"#)],
  );
}

/// 63d: `\pdfximage` takes pdfTeX's keywords (`cropbox` …) and an expanded file name, and `\pdfximagebbox` reports
/// the image's box corners as pdfTeX prints them (pinlabel.sty:588-592; 1904.09721, 1310.1838). Repro
/// expansion-primitives/pdfximagebbox_reads_the_cropbox.
#[test]
fn pdfximagebbox_reads_the_cropbox() {
  assert_elements(
    include_str!(
      "../../../tools/perfect_kernel/repros/expansion-primitives/pdfximagebbox_reads_the_cropbox.tex"
    ),
    RAW,
    (0, 0),
    &[(
      "para",
      "p1",
      r#"<para xml:id="p1"><p>BB: 0.0pt0.0pt321.2pt240.9pt</p></para>"#,
    )],
  );
}

/// 63d: pinlabel.sty is read raw, so every `\pinlabel`'s text stays with its figure, and the picture with it: the
/// pdfTeX branch's `\ps@begin` includes `<stem>.pdf` (math0412330, 1904.09721; 85 papers of run 336's first 265k had
/// `\labellist`/`\pinlabel` undefined). Repro graphics-tikz/pinlabel_labels_every_figure.
#[test]
fn pinlabel_labels_every_figure() {
  let xml = assert_elements(
    include_str!(
      "../../../tools/perfect_kernel/repros/graphics-tikz/pinlabel_labels_every_figure.tex"
    ),
    RAW,
    (0, 0),
    &[],
  );
  // mwe's images are found in the TeX tree, wherever it is installed
  let texmf = regex::Regex::new(r#"candidates="[^"]*/(example-image-[ab]\.pdf)""#).unwrap();
  let xml = texmf.replace_all(&xml, r#"candidates="TEXMF/$1""#);
  assert_element(
    &xml,
    "figure",
    &[r#"xml:id="fig1""#],
    concat!(
      r#"<figure xml:id="fig1"><block vattach="bottom"><p vattach="bottom" width="144.5pt"/><p width="0.0pt"><graphics candidates="TEXMF/example-image-a.pdf" graphic="example-image-a.pdf" options="width=144.54pt,height=108.40498pt" xml:id="g1"/></p><p vattach="bottom" width="144.5pt"><text fontsize="90%">"#,
      "\u{2003}\u{2003}\u{2003}\u{2003}\u{2003}\u{2009}",
      r#"</text><text fontsize="90%" width="7.4pt"><Math mode="inline" tex="P_{A}" text="P _ A" xml:id="m1"><XMath><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">P</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">A</XMTok></XMApp></XMath></Math></text></p></block></figure>"#
    ),
  );
  assert_element(
    &xml,
    "figure",
    &[r#"xml:id="fig2""#],
    concat!(
      r#"<figure xml:id="fig2"><block vattach="bottom"><p vattach="bottom" width="144.5pt"/><p width="0.0pt"><graphics candidates="TEXMF/example-image-b.pdf" graphic="example-image-b.pdf" options="width=144.54pt,height=108.40498pt" xml:id="g2"/></p><p vattach="bottom" width="144.5pt">"#,
      "\u{2003}\u{2003}\u{2005}",
      r#"<Math mode="inline" tex="P_{B}" text="P _ B" xml:id="m2"><XMath><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">P</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">B</XMTok></XMApp></XMath></Math></p></block></figure>"#
    ),
  );
}

/// 63d: an EPS figure puts pinlabel on its DVI branch (`\pdfoutput=0`), whose `\ps@begin` places the picture with
/// dvips's `\special{PSfile=…}` (pinlabel.sty:254-258) — not rendered, so the figure kept its labels and lost its
/// picture (1010.6236 12 → 2 images, 1112.5970 30 → 1). Both branches include through graphicx: the `.eps` the DVI
/// branch found, at the psfig size.
#[test]
fn pinlabel_dvi_figure_keeps_its_picture() {
  let (stderr, xml) = super::perfect_kernel_batch46::convert_files(
    r"\documentclass{article}
\usepackage{graphicx}
\usepackage{pinlabel}
\begin{document}
\begin{figure}
\labellist
\pinlabel $P$ at 10 10
\endlabellist
\includegraphics[width=2in]{fig}
\end{figure}
\end{document}
",
    &[(
      "fig.eps",
      "%!PS-Adobe-3.0 EPSF-3.0\n%%BoundingBox: 0 0 144 72\nnewpath 0 0 moveto 144 72 lineto stroke\nshowpage\n",
    )],
  );
  assert_eq!(
    super::perfect_kernel_batch46::error_count(&stderr),
    0,
    "{stderr}"
  );
  assert_eq!(
    super::perfect_kernel_batch46::warning_count(&stderr),
    0,
    "{stderr}"
  );
  assert_element(
    &xml,
    "figure",
    &[r#"xml:id="fig1""#],
    concat!(
      r#"<figure xml:id="fig1"><block vattach="bottom"><p vattach="bottom" width="144.5pt"/><p width="0.0pt"><graphics candidates="fig.eps" graphic="fig.eps" options="width=144.54pt,height=72.26999pt" xml:id="g1"/></p><p vattach="bottom" width="144.5pt">"#,
      "\u{2003}",
      r#"<Math mode="inline" tex="P" text="P" xml:id="m1"><XMath><XMTok font="italic" role="UNKNOWN">P</XMTok></XMath></Math></p></block></figure>"#
    ),
  );
}

/// The creators of an author repro under the arXiv profile, 0 errors and 0 warnings, each `<creator>` whole.
fn assert_creators(tex: &str, expected: &[&str]) {
  let (log, xml) = latexml::util::test::convert_with(tex, Some("ar5iv.sty"));
  assert_eq!(super::perfect_kernel_batch46::error_count(&log), 0, "{log}");
  assert_eq!(
    super::perfect_kernel_batch46::warning_count(&log),
    0,
    "{log}"
  );
  let found = super::perfect_kernel_batch61::creators_of(&xml);
  assert!(
    found == expected,
    "creators differ\n--- found:\n{}\n--- xml:\n{xml}",
    found.join("\n")
  );
}

/// 63e: a name list in one aastex/emulateapj `\author` is one author each, each with its `\altaffilmark` (1010.1318, 0908.0757,
/// astro-ph0502290; Perl aas_support.sty.ltxml:103 makes one author). Repro sectioning-frontmatter/author_emulateapj_altaffilmark_list_splits.
#[test]
fn author_emulateapj_altaffilmark_list_splits() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_emulateapj_altaffilmark_list_splits.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Joshua N. Winn</personname><contact name=\"Alternate Affiliation:\u{a0}\" role=\"altaffiliation\">MIT, Cambridge, MA</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Andrew W. Howard</personname><contact name=\"Alternate Affiliation:\u{a0}\" role=\"altaffiliation\">UC Berkeley, CA</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Avi Shporer</personname><contact name=\"Alternate Affiliation:\u{a0}\" role=\"altaffiliation\">MIT, Cambridge, MA</contact></creator>",
    ],
  );
}

/// 63e: aastex writes a name's mark after the comma that follows it (`Todd M. Tripp,\altaffilmark{2} Bart P. Wakker`,
/// astro-ph0302534): the mark stays with the name before. Repro sectioning-frontmatter/author_aastex_mark_after_comma_is_the_name_before.
#[test]
fn author_aastex_mark_after_comma_is_the_name_before() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_aastex_mark_after_comma_is_the_name_before.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Todd M. Tripp</personname><contact name=\"Alternate Affiliation:\u{a0}\" role=\"altaffiliation\">Princeton University Observatory, Princeton, NJ</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bart P. Wakker</personname><contact name=\"Alternate Affiliation:\u{a0}\" role=\"altaffiliation\">Department of Astronomy, University of Wisconsin, Madison, WI</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Edward B. Jenkins</personname><contact name=\"Alternate Affiliation:\u{a0}\" role=\"altaffiliation\">Princeton University Observatory, Princeton, NJ</contact></creator>",
    ],
  );
}

/// 63e: amsart's `\author{Klemens Fellner and Bao Quoc Tang}` names two (1708.01427, 1012.2719; Perl ams_support.sty.ltxml:99
/// makes one). Repro sectioning-frontmatter/author_per_author_class_and_list_splits.
#[test]
fn author_per_author_class_and_list_splits() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_per_author_class_and_list_splits.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Klemens Fellner</personname></creator>",
      "<creator before=\" and \" role=\"author\"><personname>Bao Quoc Tang</personname><contact name=\"Address:\u{a0}\" role=\"address\">Institute of Mathematics, University of Graz</contact></creator>",
    ],
  );
}

/// 63e: revtex's `\author{Brent Preston and Eric Poisson}` names two, both with the `\affiliation` after them
/// (gr-qc0606093, cond-mat0205631). Repro sectioning-frontmatter/author_revtex_and_list_splits.
#[test]
fn author_revtex_and_list_splits() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_revtex_and_list_splits.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Brent Preston</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Department of Physics, University of Guelph</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Eric Poisson</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Department of Physics, University of Guelph</contact></creator>",
    ],
  );
}

/// 63e: llncs/aa `\author{R. Braun\inst{1} and W. B. Burton\inst{2}}` names two, each linked to its institute
/// (astro-ph9810433; Perl inst_support.sty.ltxml:35 splits at `\and` and commas only). Repro
/// sectioning-frontmatter/author_inst_literal_and_splits.
#[test]
fn author_inst_literal_and_splits() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_inst_literal_and_splits.tex"
    ),
    &[
      "<creator role=\"author\"><personname>R. Braun</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Netherlands Foundation for Research in Astronomy</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>W. B. Burton</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Sterrewacht Leiden</contact></creator>",
    ],
  );
}

/// 63e: an "and" line between two name-and-address blocks separates two authors (cond-mat9705101; Perl
/// Base_Utility.pool.ltxml:720-725 reads every line after the first as an affiliation). Repro
/// sectioning-frontmatter/author_bare_and_line_separates_authors.
#[test]
fn author_bare_and_line_separates_authors() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_bare_and_line_separates_authors.tex"
    ),
    &[
      "<creator role=\"author\"><personname><text font=\"italic\" xml:id=\"id1\">D.A. Johnston</text></personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Dept. of Mathematics</contact><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Heriot-Watt University</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname><text font=\"italic\" xml:id=\"id2\">P. Plecháč</text></personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Mathematical Institute</contact><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Oxford</contact></creator>",
    ],
  );
}

/// 63e: a line opening with "and" before a name continues the names, and a written membership grade (`\emph{Member,
/// IEEE}`) stays with the name before it (1611.04834). Repro sectioning-frontmatter/author_ieee_member_grade_is_not_a_name.
#[test]
fn author_ieee_member_grade_is_not_a_name() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_ieee_member_grade_is_not_a_name.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Carlo Condo</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Department of Electrical and Computer Engineering, McGill University</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Pascal Giard, <emph font=\"italic\" xml:id=\"id1\">Member, IEEE</emph></personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Department of Electrical and Computer Engineering, McGill University</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Warren J. Gross, <emph font=\"italic\" xml:id=\"id2\">Senior Member, IEEE</emph></personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Department of Electrical and Computer Engineering, McGill University</contact></creator>",
    ],
  );
}

/// 63e: name lines alternating with font-switched affiliations are one author each (hep-ph9306253, hep-ph9306209).
/// Repro sectioning-frontmatter/author_name_lines_alternate_with_styled_affiliations.
#[test]
fn author_name_lines_alternate_with_styled_affiliations() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_name_lines_alternate_with_styled_affiliations.tex"
    ),
    &[
      "<creator role=\"author\"><personname>R.S. Fletcher</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><text font=\"italic\" fontsize=\"90%\" xml:id=\"id1\">Bartol Research Institute, University of Delaware</text></contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>T. Stelzer</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><text font=\"italic\" fontsize=\"90%\" xml:id=\"id2\">Physics Dept., University of Wisconsin</text></contact></creator>",
    ],
  );
}

/// 63e: names after an affiliation, styled as the group's names were, open the next names (0811.1526). Repro
/// sectioning-frontmatter/author_third_line_names_after_affiliation.
#[test]
fn author_third_line_names_after_affiliation() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_third_line_names_after_affiliation.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Christian Gollwitzer</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><text font=\"slanted\" xml:id=\"id1\">Experimentalphysik V, Univ. Bayreuth</text></contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Marina Krekhova</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><text font=\"slanted\" xml:id=\"id2\">Makromolekulare Chemie I, Univ. Bayreuth</text></contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Günter Lattermann</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><text font=\"slanted\" xml:id=\"id3\">Makromolekulare Chemie I, Univ. Bayreuth</text></contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Ingo Rehberg</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><text font=\"slanted\" xml:id=\"id4\">Experimentalphysik V, Univ. Bayreuth</text></contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Reinhard Richter</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><text font=\"slanted\" xml:id=\"id5\">Experimentalphysik V, Univ. Bayreuth</text></contact></creator>",
    ],
  );
}

/// 63e: `\newline` breaks an author line as `\\` does (latex.ltx:9256; 2401.14196). Repro
/// sectioning-frontmatter/author_newline_separates_marked_name_lines.
#[test]
fn author_newline_separates_marked_name_lines() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_newline_separates_marked_name_lines.tex"
    ),
    &[
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Ann Able</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><text fontsize=\"90%\" xml:id=\"id1\">Univ A</text></contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><text fontsize=\"90%\" xml:id=\"id2\">Univ A</text></contact><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><text fontsize=\"90%\" xml:id=\"id3\">Univ B</text></contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Cat Cole</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><text fontsize=\"90%\" xml:id=\"id4\">Univ A</text></contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Dan Dunn</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><text fontsize=\"90%\" xml:id=\"id5\">Univ B</text></contact></creator>",
    ],
  );
}

/// 63e: an `\IEEEauthorblockN` name list is one author each, every one with the block's affiliation (2410.19527); the
/// `\\` between the blocks is no affiliation. Repro sectioning-frontmatter/author_ieee_blockn_name_list_splits.
#[test]
fn author_ieee_blockn_name_list_splits() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_ieee_blockn_name_list_splits.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Clemens Paul Zengler<sup xml:id=\"id1\">1</sup></personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Technical University of Denmark.</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Niels Troldborg</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Technical University of Denmark.</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Mac Gaunaa</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Technical University of Denmark.</contact></creator>",
    ],
  );
}

/// 63e: authblk's `\author[1]{Ann Able, Bob Baker}` is one author each, both with the label (2401.14196's
/// deepseek.cls). Repro sectioning-frontmatter/author_authblk_labelled_list_splits.
#[test]
fn author_authblk_labelled_list_splits() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_authblk_labelled_list_splits.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact></creator>",
    ],
  );
}

/// 63e: an author holding two names is a loss the document cannot show — LaTeX typesets two people, the XML has one —
/// so it is an error (`Error:frontmatter:merged_creators`), once per such author. acmart's `\author` is one author by
/// the class's contract, so the list stays as written and is reported (the run-336 fidelity audit's 16 silent
/// merged-author papers). Repro sectioning-frontmatter/author_merged_names_are_an_error.
#[test]
fn author_merged_names_are_an_error() {
  let (log, xml) = latexml::util::test::convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_merged_names_are_an_error.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!(super::perfect_kernel_batch46::error_count(&log), 1, "{log}");
  assert_eq!(
    super::perfect_kernel_batch46::warning_count(&log),
    0,
    "{log}"
  );
  assert!(
    log
      .lines()
      .any(|l| l.starts_with("Error:frontmatter:merged_creators")
        && l.contains("Ann Able and Bob Baker")),
    "{log}"
  );
  assert_eq!(
    super::perfect_kernel_batch61::creators_of(&xml),
    [
      "<creator role=\"author\"><personname>Ann Able and Bob Baker</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><text class=\"ltx_affiliation_institution\" xml:id=\"id1\">Univ A</text>, <text class=\"ltx_affiliation_country\" xml:id=\"id2\">Country</text></contact></creator>"
    ],
    "{xml}"
  );
}

/// 63e (negative): the lines under a name are its affiliations, a city line with its postal code included
/// (`Los Angeles, CA 90095, USA`: a code after a space is text, not a mark). Repro
/// sectioning-frontmatter/author_affiliation_lines_stay_affiliations.
#[test]
fn author_affiliation_lines_stay_affiliations() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_affiliation_lines_stay_affiliations.tex"
    ),
    &[
      "<creator role=\"author\"><personname>John Smith</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><text font=\"italic\" xml:id=\"id1\">Dept. of Physics, UCLA</text></contact><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Los Angeles, CA 90095, USA</contact></creator>",
    ],
  );
}

/// 63e (negative): a line opening with "and" before an institution stays an affiliation (math0208081). Repro
/// sectioning-frontmatter/author_and_led_institution_stays_an_affiliation.
#[test]
fn author_and_led_institution_stays_an_affiliation() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_and_led_institution_stays_an_affiliation.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Ralph M. Kaufmann</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">University of Southern California, Los Angeles, USA</contact><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">and Max–Planck Institut für Mathematik, Bonn, Germany</contact></creator>",
    ],
  );
}

/// 63e (negative): "and INFN Sezione di Roma, Italy" stays an affiliation: institution words of the languages
/// affiliations are written in. Repro sectioning-frontmatter/author_and_led_italian_institute_stays_an_affiliation.
#[test]
fn author_and_led_italian_institute_stays_an_affiliation() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_and_led_italian_institute_stays_an_affiliation.tex"
    ),
    &[
      "<creator role=\"author\"><personname>A. Smith</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Dipartimento di Fisica, Università di Roma</contact><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">and INFN Sezione di Roma, Italy</contact><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">and Max Planck Inst.</contact></creator>",
    ],
  );
}

/// 63e: amsart's `\author{Name\\ \small Faculty …}` is a name over its affiliations and address (math0606082; Perl one
/// personname). Repro sectioning-frontmatter/author_amsart_name_then_affiliation_lines.
#[test]
fn author_amsart_name_then_affiliation_lines() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_amsart_name_then_affiliation_lines.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Masao Ishikawa</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><text fontsize=\"90%\" xml:id=\"id1\">Faculty of Education, Tottori University</text></contact><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><text fontsize=\"90%\" xml:id=\"id2\">Koyama, Tottori, Japan</text></contact><contact name=\"Email:\u{a0}\" role=\"email\"><text font=\"typewriter\" fontsize=\"90%\" xml:id=\"id3\">ishikawa@fed.tottori-u.ac.jp</text></contact></creator>",
    ],
  );
}

/// 63e (negative): an all-capitals name is a name, not a society after a grade (`WEI LI\inst{1}, HONG ZHANG\inst{2}`).
/// Repro sectioning-frontmatter/author_uppercase_names_stay_apart.
#[test]
fn author_uppercase_names_stay_apart() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_uppercase_names_stay_apart.tex"
    ),
    &[
      "<creator role=\"author\"><personname>WEI LI</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>HONG ZHANG</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ B</contact></creator>",
    ],
  );
}

/// 63e (negative): an `\IEEEauthorblockN` line under the name is its affiliation, not more authors. Repro
/// sectioning-frontmatter/author_ieee_blockn_lines_after_the_name_are_affiliations.
#[test]
fn author_ieee_blockn_lines_after_the_name_are_affiliations() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_ieee_blockn_lines_after_the_name_are_affiliations.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">University of X, City, Country</contact></creator>",
    ],
  );
}

/// 63e (negative): degrees after a name stay with it (`\author[1]{Jane Doe, MD, PhD}`). Repro
/// sectioning-frontmatter/author_degrees_stay_with_the_name.
#[test]
fn author_degrees_stay_with_the_name() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_degrees_stay_with_the_name.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Jane Doe, MD, PhD</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact></creator>",
    ],
  );
}

/// 63e (negative): a names line ending in a comma does not make an affiliation styled unlike it a name
/// (`John Smith,\\ {\it Bell Labs, Murray Hill}`). Repro sectioning-frontmatter/author_comma_before_styled_affiliation.
#[test]
fn author_comma_before_styled_affiliation() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_comma_before_styled_affiliation.tex"
    ),
    &[
      "<creator role=\"author\"><personname>John Smith</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><text font=\"italic\" xml:id=\"id1\">Bell Labs, Murray Hill</text></contact></creator>",
    ],
  );
}

/// 63e (negative): a name beside its institution is not two people — no `merged_creators` error for acmart's
/// `\author{Jane Doe, Bell Labs}`. Repro sectioning-frontmatter/author_name_with_affiliation_is_not_merged.
#[test]
fn author_name_with_affiliation_is_not_merged() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_name_with_affiliation_is_not_merged.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Jane Doe, Bell Labs</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><text class=\"ltx_affiliation_institution\" xml:id=\"id1\">Univ A</text>, <text class=\"ltx_affiliation_country\" xml:id=\"id2\">Country</text></contact></creator>",
    ],
  );
}

/// 63e: after affiliations, a line of names opening with "and" opens the next author (math-ph0303018). Repro
/// sectioning-frontmatter/author_and_led_names_after_affiliations_open_an_author.
#[test]
fn author_and_led_names_after_affiliations_open_an_author() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_and_led_names_after_affiliations_open_an_author.tex"
    ),
    &[
      "<creator role=\"author\"><personname>James Brink</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Dept of Math., UC Berkeley</contact></creator>",
      "<creator before=\" and \" role=\"author\"><personname>Zhenghan Wang</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Dept of Math., Indiana University</contact></creator>",
    ],
  );
}

/// 63e: a names line wrapped whole in a group with its declarations splits like `\textbf{A, B}` does (hep-th9212083,
/// 2608.16650). Repro sectioning-frontmatter/author_group_wrapped_names_split.
#[test]
fn author_group_wrapped_names_split() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_group_wrapped_names_split.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Daniel Boyanovsky</personname></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Da-Shin Lee</personname></creator>",
    ],
  );
}

/// 63e: a thin space before "and" is the space of " and " (`Aghil Alaee\footnote{…} \,\,and Hari K. Kunduri`, 1407.0988),
/// and an unmarked names line on the marked path is split when it names several; the department, whose mark no author
/// requests, keeps a creator of its own (OXIDIZED_DESIGN #159, repro author_block_orphan_mark_is_kept).
/// Repro sectioning-frontmatter/author_thin_space_before_and_separates.
#[test]
fn author_thin_space_before_and_separates() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_thin_space_before_and_separates.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Aghil Alaee</personname><contact name=\"Note:\u{a0}\" role=\"note\">aak818@mun.ca</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Hari K. Kunduri</personname><contact name=\"Note:\u{a0}\" role=\"note\">hkkunduri@mun.ca</contact></creator>",
      "<creator role=\"author\"><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><text font=\"slanted\" fontsize=\"90%\" xml:id=\"id1\"> Department of Mathematics and Statistics, Memorial University of Newfoundland</text></contact></creator>",
    ],
  );
}

/// 63e: the marks written after the commas inside a group-wrapped name list stay with the names before them
/// (`{\bf Todd M. Tripp,\altaffilmark{2} Bart P. Wakker,\altaffilmark{4}}`, astro-ph0302534's shape). Repro
/// sectioning-frontmatter/author_group_wrapped_marks_after_commas.
#[test]
fn author_group_wrapped_marks_after_commas() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_group_wrapped_marks_after_commas.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Todd M. Tripp</personname><contact name=\"Alternate Affiliation:\u{a0}\" role=\"altaffiliation\">Princeton University Observatory, Princeton, NJ</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bart P. Wakker</personname><contact name=\"Alternate Affiliation:\u{a0}\" role=\"altaffiliation\">Department of Astronomy, University of Wisconsin, Madison, WI</contact></creator>",
    ],
  );
}

/// 63e: an "and" inside the markup that opens a names line after affiliations is dropped from the name
/// (`{\it and Zhenghan Wang}`). Repro sectioning-frontmatter/author_and_inside_markup_opens_an_author.
#[test]
fn author_and_inside_markup_opens_an_author() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_and_inside_markup_opens_an_author.tex"
    ),
    &[
      "<creator role=\"author\"><personname><text font=\"italic\" xml:id=\"id1\">James Brink</text></personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Dept of Math., UC Berkeley</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname><text font=\"italic\" xml:id=\"id2\">Zhenghan Wang</text></personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Dept of Math., Indiana University</contact></creator>",
    ],
  );
}

/// 63e: "A, B, and C" makes three authors and no empty fourth, so an affiliation for them all is each one's once
/// (an empty creator took a copy and merged it onto B). Repro sectioning-frontmatter/author_serial_comma_list_shares_one_affiliation.
#[test]
fn author_serial_comma_list_shares_one_affiliation() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_serial_comma_list_shares_one_affiliation.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Department of Physics, University X</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Department of Physics, University X</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Cat Cole</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Department of Physics, University X</contact></creator>",
    ],
  );
}

/// 63e (negative): a name list part naming an institution rejoins the name before it (`\author{John Smith,
/// University of Toronto}`). Repro sectioning-frontmatter/author_name_then_institution_stays_one_author.
#[test]
fn author_name_then_institution_stays_one_author() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_name_then_institution_stays_one_author.tex"
    ),
    &[
      "<creator role=\"author\"><personname>John Smith, University of Toronto</personname></creator>",
    ],
  );
}

/// 63e (negative): a line of only "and" before an affiliation is affiliation text, not the start of an author.
/// Repro sectioning-frontmatter/author_and_line_before_an_institution_stays_text.
#[test]
fn author_and_line_before_an_institution_stays_text() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_and_line_before_an_institution_stays_text.tex"
    ),
    &[
      "<creator role=\"author\"><personname>John Smith</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Dept. of Physics</contact><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">and</contact><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Center for Theoretical Physics</contact></creator>",
    ],
  );
}

/// 63e: an aastex `\affiliation` is every preceding author's that has none (AASTeX 6, revtex-derived). Repro
/// sectioning-frontmatter/author_aastex_affiliation_goes_to_the_authors_before.
#[test]
fn author_aastex_affiliation_goes_to_the_authors_before() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_aastex_affiliation_goes_to_the_authors_before.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ</contact></creator>",
    ],
  );
}

/// 63e (negative): institution words are whole words, not word openings — "Patrick Strasser" is a name, not
/// `strasse` (folded into the name before it, silently). Repro sectioning-frontmatter/author_surname_like_an_institution_word_is_a_name.
#[test]
fn author_surname_like_an_institution_word_is_a_name() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_surname_like_an_institution_word_is_a_name.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Patrick Strasser</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ B</contact></creator>",
    ],
  );
}

/// 63e (negative): amsart's `\author{Ann Able and Patrick Strasser}` is two authors. Repro
/// sectioning-frontmatter/author_amsart_and_list_with_institution_like_surname.
#[test]
fn author_amsart_and_list_with_institution_like_surname() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_amsart_and_list_with_institution_like_surname.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Ann Able</personname></creator>",
      "<creator before=\" and \" role=\"author\"><personname>Patrick Strasser</personname></creator>",
    ],
  );
}

/// 63e: "A, B, and C" inside a wrapper (`\textbf{…}`) makes three authors and no empty fourth, each with the
/// affiliation once. Repro sectioning-frontmatter/author_wrapped_serial_comma_list_shares_one_affiliation.
#[test]
fn author_wrapped_serial_comma_list_shares_one_affiliation() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_wrapped_serial_comma_list_shares_one_affiliation.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Department of Physics, University X</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Department of Physics, University X</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Cat Cole</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Department of Physics, University X</contact></creator>",
    ],
  );
}

/// 63e: a membership grade in an IEEE name block (`Ann Able,~\IEEEmembership{Member,~IEEE}`) stays with its name, not an
/// empty author taking a copy of the block's affiliation (conference mode prints no grade, IEEEtran.cls:6270). Repro
/// sectioning-frontmatter/author_ieee_membership_in_a_block_stays_with_its_name.
#[test]
fn author_ieee_membership_in_a_block_stays_with_its_name() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_ieee_membership_in_a_block_stays_with_its_name.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ</contact></creator>",
    ],
  );
}

/// 63e (negative): an "and" line before "SISSA, Trieste, Italy" is affiliation text. Repro
/// sectioning-frontmatter/author_and_line_before_an_acronym_institution_stays_text.
#[test]
fn author_and_line_before_an_acronym_institution_stays_text() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_and_line_before_an_acronym_institution_stays_text.tex"
    ),
    &[
      "<creator role=\"author\"><personname>John Smith</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Dept. of Physics, Univ. X</contact><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">and</contact><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">SISSA, Trieste, Italy</contact></creator>",
    ],
  );
}

/// 63e (negative): names set in math (`$\mbox{\rm F. del Aguila}^{1}$ and $\mbox{\rm M. Zra{\l }ek}^{2}$`,
/// hep-ph9504228) stay two authors: a piece holding math is not a piece with nothing to read. Repro
/// sectioning-frontmatter/author_names_set_in_math_stay_apart.
#[test]
fn author_names_set_in_math_stay_apart() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_names_set_in_math_stay_apart.tex"
    ),
    &[
      "<creator role=\"author\"><personname><text class=\"ltx_markedasmath\" xml:id=\"id1\">F. del Aguila</text></personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"> Universidad de Granada</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname><text class=\"ltx_markedasmath\" xml:id=\"id2\">M. Zrałek</text></personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"> University of Silesia</contact></creator>",
    ],
  );
}

/// 63e (negative): a per-author class splits only a line of names — `\author{John Smith, Max Planck Institute for
/// Mathematics}` stays one author as written. Repro sectioning-frontmatter/author_person_named_institution_stays_with_the_name.
#[test]
fn author_person_named_institution_stays_with_the_name() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_person_named_institution_stays_with_the_name.tex"
    ),
    &[
      "<creator role=\"author\"><personname>John Smith, Max Planck Institute for Mathematics</personname></creator>",
    ],
  );
}

/// 63e (negative): authors written as macros (`\author{\A, \B}`) are something to read, not marks: two authors. Repro
/// sectioning-frontmatter/author_macro_names_stay_apart.
#[test]
fn author_macro_names_stay_apart() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_macro_names_stay_apart.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Ann Able</personname></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname></creator>",
    ],
  );
}

/// 63e (negative): "Street" is a surname (`Rachel Street$^{2}$`), not an institution word. Repro
/// sectioning-frontmatter/author_surname_street_is_a_name.
#[test]
fn author_surname_street_is_a_name() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_surname_street_is_a_name.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Rachel Street</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ B</contact></creator>",
    ],
  );
}

/// 63e (negative): a per-author binding keeps its authors' marks visible, as Perl does — an IEEE block's
/// `\IEEEauthorblockA` answers no mark request. Repro sectioning-frontmatter/author_ieee_block_marks_stay_visible.
#[test]
fn author_ieee_block_marks_stay_visible() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_ieee_block_marks_stay_visible.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Ann Able<sup xml:id=\"id1\"><text font=\"italic\" xml:id=\"id1.1\">1</text></sup></personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><sup xml:id=\"id2\"><text font=\"italic\" xml:id=\"id2.1\">1</text></sup>Univ A<break/><sup xml:id=\"id3\"><text font=\"italic\" xml:id=\"id3.1\">2</text></sup>Univ B</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker<sup xml:id=\"id4\"><text font=\"italic\" xml:id=\"id4.1\">2</text></sup></personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><sup xml:id=\"id5\"><text font=\"italic\" xml:id=\"id5.1\">1</text></sup>Univ A<break/><sup xml:id=\"id6\"><text font=\"italic\" xml:id=\"id6.1\">2</text></sup>Univ B</contact></creator>",
    ],
  );
}

/// 63e (negative): amsart's marked `\author{Ann Able$^{1}$ and Bob Baker$^{2}$}` keeps both marks visible. Repro
/// sectioning-frontmatter/author_amsart_marks_stay_visible.
#[test]
fn author_amsart_marks_stay_visible() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_amsart_marks_stay_visible.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Ann Able<sup xml:id=\"id1\"><text font=\"italic\" xml:id=\"id1.1\">1</text></sup></personname><contact name=\"Address:\u{a0}\" role=\"address\"><sup xml:id=\"id2\">1</sup>Univ A</contact></creator>",
      "<creator before=\" and \" role=\"author\"><personname>Bob Baker<sup xml:id=\"id3\"><text font=\"italic\" xml:id=\"id3.1\">2</text></sup></personname><contact name=\"Address:\u{a0}\" role=\"address\"><sup xml:id=\"id4\">2</sup>Univ B</contact></creator>",
    ],
  );
}

/// 63e (negative): a per-author binding's line that is not a list of names stays as written, its wrapper whole
/// (`\textbf{John Smith, Max Planck Institute for Mathematics}`). Repro
/// sectioning-frontmatter/author_wrapped_name_and_institution_kept_as_written.
#[test]
fn author_wrapped_name_and_institution_kept_as_written() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_wrapped_name_and_institution_kept_as_written.tex"
    ),
    &[
      "<creator role=\"author\"><personname>John Smith, Max Planck Institute for Mathematics</personname></creator>",
    ],
  );
}

/// 63e: a piece of marks and notes only (`$^{*}$\thanks{…}`) stays with the name before it, not an author showing
/// only "*". Repro sectioning-frontmatter/author_symbol_mark_piece_stays_with_the_name.
#[test]
fn author_symbol_mark_piece_stays_with_the_name() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_symbol_mark_piece_stays_with_the_name.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Ann Able<sup xml:id=\"id1\">*</sup></personname><note class=\"ltx_note_frontmatter ltx_thanks_contribution\" role=\"thanks\" xml:id=\"id2\">Equal contribution</note></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname></creator>",
    ],
  );
}

/// 63e: in an IEEE block, a line of names each marked after it continues the names above, as authors' marks stand
/// (`Ann Able$^{1}$\\ Bob Baker$^{2}$`), not an affiliation of the first (review r7). Repro
/// sectioning-frontmatter/author_ieee_blockn_marked_name_lines_are_names.
#[test]
fn author_ieee_blockn_marked_name_lines_are_names() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_ieee_blockn_marked_name_lines_are_names.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Ann Able<sup xml:id=\"id1\"><text font=\"italic\" xml:id=\"id1.1\">1</text></sup></personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><sup xml:id=\"id2\"><text font=\"italic\" xml:id=\"id2.1\">1</text></sup>Univ A<break/><sup xml:id=\"id3\"><text font=\"italic\" xml:id=\"id3.1\">2</text></sup>Univ B</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker<sup xml:id=\"id4\"><text font=\"italic\" xml:id=\"id4.1\">2</text></sup></personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><sup xml:id=\"id5\"><text font=\"italic\" xml:id=\"id5.1\">1</text></sup>Univ A<break/><sup xml:id=\"id6\"><text font=\"italic\" xml:id=\"id6.1\">2</text></sup>Univ B</contact></creator>",
    ],
  );
}

/// 63e: an emulateapj name list over two lines, every name with its `\altaffilmark` after it, is one list of authors
/// with no comma before the break (review r7). Repro sectioning-frontmatter/author_emulateapj_marked_name_lines_continue.
#[test]
fn author_emulateapj_marked_name_lines_continue() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_emulateapj_marked_name_lines_continue.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Alternate Affiliation:\u{a0}\" role=\"altaffiliation\">Univ A</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname><contact name=\"Alternate Affiliation:\u{a0}\" role=\"altaffiliation\">Univ B</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Cat Cole</personname><contact name=\"Alternate Affiliation:\u{a0}\" role=\"altaffiliation\">Univ C</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Dan Dole</personname><contact name=\"Alternate Affiliation:\u{a0}\" role=\"altaffiliation\">Univ D</contact></creator>",
    ],
  );
}

/// 63e: a block of names marked before them (`$^{1}$Ann Able \quad $^{2}$Bob Baker`) holds no affiliation its marks
/// request: two authors, not an author and an affiliation labelled 2 (review r7). Repro
/// sectioning-frontmatter/author_ieee_blockn_prefix_marks_across_quad.
#[test]
fn author_ieee_blockn_prefix_marks_across_quad() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_ieee_blockn_prefix_marks_across_quad.tex"
    ),
    &[
      "<creator role=\"author\"><personname><sup xml:id=\"id1\"><text font=\"italic\" xml:id=\"id1.1\">1</text></sup>Ann Able</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><sup xml:id=\"id2\"><text font=\"italic\" xml:id=\"id2.1\">1</text></sup>Univ A<break/><sup xml:id=\"id3\"><text font=\"italic\" xml:id=\"id3.1\">2</text></sup>Univ B</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname><sup xml:id=\"id4\"><text font=\"italic\" xml:id=\"id4.1\">2</text></sup>Bob Baker</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><sup xml:id=\"id5\"><text font=\"italic\" xml:id=\"id5.1\">1</text></sup>Univ A<break/><sup xml:id=\"id6\"><text font=\"italic\" xml:id=\"id6.1\">2</text></sup>Univ B</contact></creator>",
    ],
  );
}

/// 63e (negative): a two-word place after a name (`\author{John Smith, New York}`) invents no author (review r7). Repro
/// sectioning-frontmatter/author_amsart_name_and_place_stays_one_author.
#[test]
fn author_amsart_name_and_place_stays_one_author() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_amsart_name_and_place_stays_one_author.tex"
    ),
    &["<creator role=\"author\"><personname>John Smith, New York</personname></creator>"],
  );
}

/// 63e: a per-author class's `\author` is added whole, so an `\affiliations` marker in it (`\let\affiliations\relax`)
/// does not re-enter as a replacing `\author` that drops the authors before it (review r7). Repro
/// sectioning-frontmatter/author_revtex_ijcai_marker_keeps_earlier_authors.
#[test]
fn author_revtex_ijcai_marker_keeps_earlier_authors() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_revtex_ijcai_marker_keeps_earlier_authors.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Ann Able</personname></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker Univ A</personname></creator>",
    ],
  );
}

/// 63e: names marked before them that share a label, side by side in an IEEE block (`$^{1}$Ann Able \quad
/// $^{1}$Bob Baker`), are two authors keeping their marks, not an author and her affiliation (review r8). Repro
/// sectioning-frontmatter/author_ieee_blockn_prefix_marks_shared_label.
#[test]
fn author_ieee_blockn_prefix_marks_shared_label() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_ieee_blockn_prefix_marks_shared_label.tex"
    ),
    &[
      "<creator role=\"author\"><personname><sup xml:id=\"id1\"><text font=\"italic\" xml:id=\"id1.1\">1</text></sup>Ann Able</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><sup xml:id=\"id2\"><text font=\"italic\" xml:id=\"id2.1\">1</text></sup>Univ A</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname><sup xml:id=\"id3\"><text font=\"italic\" xml:id=\"id3.1\">1</text></sup>Bob Baker</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><sup xml:id=\"id4\"><text font=\"italic\" xml:id=\"id4.1\">1</text></sup>Univ A</contact></creator>",
    ],
  );
}

/// 63e: three names marked before them on one printed line of an IEEE block are three authors (review r8). Repro
/// sectioning-frontmatter/author_ieee_blockn_prefix_marks_three_names.
#[test]
fn author_ieee_blockn_prefix_marks_three_names() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_ieee_blockn_prefix_marks_three_names.tex"
    ),
    &[
      "<creator role=\"author\"><personname><sup xml:id=\"id1\"><text font=\"italic\" xml:id=\"id1.1\">1</text></sup>Ann Able</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><sup xml:id=\"id2\"><text font=\"italic\" xml:id=\"id2.1\">1</text></sup>Univ A<break/><sup xml:id=\"id3\"><text font=\"italic\" xml:id=\"id3.1\">2</text></sup>Univ B</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname><sup xml:id=\"id4\"><text font=\"italic\" xml:id=\"id4.1\">2</text></sup>Bob Baker</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><sup xml:id=\"id5\"><text font=\"italic\" xml:id=\"id5.1\">1</text></sup>Univ A<break/><sup xml:id=\"id6\"><text font=\"italic\" xml:id=\"id6.1\">2</text></sup>Univ B</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname><sup xml:id=\"id7\"><text font=\"italic\" xml:id=\"id7.1\">1</text></sup>Cat Cole</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><sup xml:id=\"id8\"><text font=\"italic\" xml:id=\"id8.1\">1</text></sup>Univ A<break/><sup xml:id=\"id9\"><text font=\"italic\" xml:id=\"id9.1\">2</text></sup>Univ B</contact></creator>",
    ],
  );
}

/// 63e: in the article `\author`, names marked before them beside the first on its printed line are authors, the
/// marked lines after the break their affiliations, linked by mark (review r8). Repro
/// sectioning-frontmatter/author_prefix_marked_names_beside_on_one_line.
#[test]
fn author_prefix_marked_names_beside_on_one_line() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_prefix_marked_names_beside_on_one_line.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Cat Cole</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ B</contact></creator>",
    ],
  );
}

/// 63e: in the article `\author`, a line of names marked before it, its mark one no author before it requests, is
/// an author before the marked affiliations (review r8). Repro sectioning-frontmatter/author_prefix_marked_name_lines.
#[test]
fn author_prefix_marked_name_lines() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_prefix_marked_name_lines.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ B</contact></creator>",
    ],
  );
}

/// 63e: amsart's `\author` with a name marked before it on each line and the marked affiliations after them links
/// each author to its affiliation (review r8). Repro sectioning-frontmatter/author_amsart_prefix_marked_name_lines.
#[test]
fn author_amsart_prefix_marked_name_lines() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_amsart_prefix_marked_name_lines.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact></creator>",
      "<creator before=\" and \" role=\"author\"><personname>Bob Baker</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ B</contact></creator>",
    ],
  );
}

/// 63e: an `\altaffilmark` on the affiliation line under the names annotates the author whose affiliation it is,
/// not a creator of its own (review r8). The line sits under both names in the PDF and is both names' (63m); the note
/// it carries stays with the last name, where the line is digested (OD #459). Repro
/// sectioning-frontmatter/author_emulateapj_affiliation_line_altaffilmark_stays_with_author.
#[test]
fn author_emulateapj_affiliation_line_altaffilmark_stays_with_author() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_emulateapj_affiliation_line_altaffilmark_stays_with_author.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Steward Observatory</contact><contact name=\"Alternate Affiliation:\u{a0}\" role=\"altaffiliation\">Fellow</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Steward Observatory</contact><contact name=\"Alternate Affiliation:\u{a0}\" role=\"altaffiliation\">Tucson</contact></creator>",
    ],
  );
}

/// 63e: a `\thanks` on the affiliation line under a revtex name is a note of that author, as Perl gives it (the
/// article `\author` keeps Perl's document-level note, html_feedback#6888) (review r8). Repro
/// sectioning-frontmatter/author_revtex_affiliation_line_thanks_stays_with_author.
#[test]
fn author_revtex_affiliation_line_thanks_stays_with_author() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_revtex_affiliation_line_thanks_stays_with_author.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact><note class=\"ltx_note_frontmatter ltx_thanks_funding\" role=\"thanks\" xml:id=\"id1\">Grant</note></creator>",
    ],
  );
}

/// 63e: a line break before a `\quad` ends the printed line of names marked before them: the marked lines after it
/// are affiliations (review r9). Repro sectioning-frontmatter/author_prefix_marked_affiliations_after_break_and_quad.
#[test]
fn author_prefix_marked_affiliations_after_break_and_quad() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_prefix_marked_affiliations_after_break_and_quad.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ B</contact></creator>",
    ],
  );
}

/// 63e: an IEEE block of names marked before them and their affiliations after a break, each after `\quad`, is two
/// authors linked to their affiliations (review r9). Repro
/// sectioning-frontmatter/author_ieee_blockn_prefix_marks_affiliations_after_break.
#[test]
fn author_ieee_blockn_prefix_marks_affiliations_after_break() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_ieee_blockn_prefix_marks_affiliations_after_break.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ B</contact></creator>",
    ],
  );
}

/// 63e (negative): an affiliation beside a name marked before it on its printed line (`$^{1}$Ann Able \quad
/// $^{1}$University of Arizona`) stays her affiliation; only a line that reads as names stands beside as one (review
/// r9). Repro sectioning-frontmatter/author_prefix_marked_affiliation_beside_name_stays_affiliation.
#[test]
fn author_prefix_marked_affiliation_beside_name_stays_affiliation() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_prefix_marked_affiliation_beside_name_stays_affiliation.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">University of Arizona</contact></creator>",
    ],
  );
}

/// 63e: a name marked before it after the affiliations began (`… $^{1}$Univ A \and $^{2}$Bob Baker\\ $^{2}$Univ B`) is
/// an author when a line below answers its mark (review r9). Repro
/// sectioning-frontmatter/author_prefix_marked_name_after_affiliations.
#[test]
fn author_prefix_marked_name_after_affiliations() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_prefix_marked_name_after_affiliations.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ B</contact></creator>",
    ],
  );
}

/// 63e (negative): a name-shaped affiliation line whose mark nothing below answers (`$^{3}$Google DeepMind`) stays an
/// affiliation, kept apart as no author requests it (OD #159) (review r9). Repro
/// sectioning-frontmatter/author_prefix_marked_unrequested_affiliation_stays_affiliation.
#[test]
fn author_prefix_marked_unrequested_affiliation_stays_affiliation() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_prefix_marked_unrequested_affiliation_stays_affiliation.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ B</contact></creator>",
      "<creator role=\"author\"><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Google DeepMind</contact></creator>",
    ],
  );
}

/// 63e (negative): a name-shaped company affiliation (`$^{1}$Google DeepMind`) stays an affiliation when the line
/// below led by its mark is an email: only an affiliation-shaped line answers a mark (review r10). Repro
/// sectioning-frontmatter/author_prefix_marked_company_affiliation_with_email_line.
#[test]
fn author_prefix_marked_company_affiliation_with_email_line() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_prefix_marked_company_affiliation_with_email_line.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Google DeepMind</contact><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><text font=\"typewriter\" xml:id=\"id1\">ann@google.com</text></contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Huawei Technologies</contact><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><text font=\"typewriter\" xml:id=\"id2\">bob@huawei.com</text></contact></creator>",
    ],
  );
}

/// 63e (negative): a name-shaped company affiliation stays an affiliation when the line below led by its mark is a
/// place (`$^{1}$Hong Kong`) (review r10). Repro
/// sectioning-frontmatter/author_prefix_marked_company_affiliation_with_place_line.
#[test]
fn author_prefix_marked_company_affiliation_with_place_line() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_prefix_marked_company_affiliation_with_place_line.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Google DeepMind</contact><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Hong Kong</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Hong Kong Baptist University</contact></creator>",
    ],
  );
}

/// 63e: names marked before them side by side, their company affiliations and emails below, are two authors each
/// with theirs (review r10). Repro sectioning-frontmatter/author_prefix_marked_names_beside_company_affiliations_below.
#[test]
fn author_prefix_marked_names_beside_company_affiliations_below() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_prefix_marked_names_beside_company_affiliations_below.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Google DeepMind</contact><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><text font=\"typewriter\" xml:id=\"id1\">ann@google.com</text></contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Huawei Technologies</contact><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><text font=\"typewriter\" xml:id=\"id2\">bob@huawei.com</text></contact></creator>",
    ],
  );
}

/// 63e (negative): a name-shaped company affiliation beside a name marked before it, the block one printed line
/// (`$^{1}$Ann Able \quad $^{1}$Google DeepMind`), stays her affiliation (review r10). Repro
/// sectioning-frontmatter/author_prefix_marked_company_affiliation_beside_name.
#[test]
fn author_prefix_marked_company_affiliation_beside_name() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_prefix_marked_company_affiliation_beside_name.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Google DeepMind</contact></creator>",
    ],
  );
}

/// 63e (negative): a name-shaped affiliation line whose mark a name above requests (`$^{1}$Carnegie Mellon`) stays an
/// affiliation, though a second line with that mark answers it (review r11). Repro
/// sectioning-frontmatter/author_prefix_marked_company_affiliation_repeated_mark_line.
#[test]
fn author_prefix_marked_company_affiliation_repeated_mark_line() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_prefix_marked_company_affiliation_repeated_mark_line.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Carnegie Mellon</contact><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">School of Computer Science</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ B</contact></creator>",
    ],
  );
}

/// 62zp/63e: `\and` groups whose names open with the mark of an author before them are authors when an affiliation
/// below answers the mark (`\textsuperscript{1}Ann Able \and \textsuperscript{1}Bob Baker\\ \textsuperscript{1}Univ A`)
/// (review r12). Repro sectioning-frontmatter/author_and_groups_sharing_a_prefix_mark_are_names.
#[test]
fn author_and_groups_sharing_a_prefix_mark_are_names() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_and_groups_sharing_a_prefix_mark_are_names.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact></creator>",
    ],
  );
}

/// 63f: acmart's `\received` adds to one history (acmart.cls:1859-1872, "Received 20 February 2007; revised 12 March
/// 2009; revised 3 May 2009; accepted 5 June 2009"): the first unlabelled date is received, a later unlabelled one
/// revised, a labelled one its label — each its own date, the later ones `accumulate`. Perl's `\lx@add@date[role=received]` per call (acmart.cls.ltxml:58) cleared the
/// earlier ones, so only the last survived, as "Received". Repro sectioning-frontmatter/acmart_received_dates_all_kept.
#[test]
fn acmart_received_dates_all_kept() {
  let (log, xml) = latexml::util::test::convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/acmart_received_dates_all_kept.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!(
    (
      super::perfect_kernel_batch46::error_count(&log),
      super::perfect_kernel_batch46::warning_count(&log)
    ),
    (0, 0),
    "{log}"
  );
  let mut dates = Vec::new();
  let mut from = 0;
  while let Some(at) = xml[from..].find("<date") {
    dates.extend(latexml::util::test::xml_element(
      &xml[from + at..],
      "date",
      &[],
    ));
    from += at + 1;
  }
  // acmart's own copyright date carries the year of the run
  dates.retain(|date| !date.contains("role=\"copyright\""));
  assert_eq!(
    dates,
    [
      "<date name=\"Received\u{a0}\" role=\"received\">20 February 2007</date>",
      "<date name=\"revised\u{a0}\" role=\"revised\">12 March 2009</date>",
      "<date name=\"revised\u{a0}\" role=\"revised\">3 May 2009</date>",
      "<date name=\"accepted\u{a0}\" role=\"accepted\">5 June 2009</date>",
    ],
    "{xml}"
  );
}

/// 63f: initials glued to the surname they precede (`A.G.Bogdanchikov`, `Yu.M.Shatunov`) read as a name, and a
/// `\vspace{1mm}` on the last names line is no text, so a collaboration list continued over `\\` lines stays names
/// (hep-ex0105093: 5 creators, 38 in 63f). Repro sectioning-frontmatter/author_glued_initials_list_continues.
#[test]
fn author_glued_initials_list_continues() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_glued_initials_list_continues.tex"
    ),
    &[
      "<creator role=\"author\"><personname>G.N.Abramov</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Budker Institute of Nuclear Physics</contact><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">630090, Novosibirsk, Russia</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>M.N.Achasov</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Budker Institute of Nuclear Physics</contact><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">630090, Novosibirsk, Russia</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>A.G.Bogdanchikov</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Budker Institute of Nuclear Physics</contact><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">630090, Novosibirsk, Russia</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Yu.M.Shatunov</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Budker Institute of Nuclear Physics</contact><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">630090, Novosibirsk, Russia</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>A.V.Vasiljev</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Budker Institute of Nuclear Physics</contact><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">630090, Novosibirsk, Russia</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Yu.S.Velikzhanin</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Budker Institute of Nuclear Physics</contact><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">630090, Novosibirsk, Russia</contact></creator>",
    ],
  );
}

/// 63f: the period of an initial or of a suffix written with one ends the name (`Sridhar K.`, hep-ph9306209; `John
/// Smith Jr.`); Perl's personname clean-up strips every trailing non-word character (KNOWN_PERL_ERRORS #523). A
/// two-letter last word is a surname as often as an initial (`Wei Li.`): its period goes, as Perl's. Repro
/// sectioning-frontmatter/author_name_ending_in_an_initial_keeps_its_period.
#[test]
fn author_name_ending_in_an_initial_keeps_its_period() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_name_ending_in_an_initial_keeps_its_period.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Sridhar K.</personname></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>John Smith Jr.</personname></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Wei Li</personname></creator>",
    ],
  );
}

/// 63f (negative): a glued place abbreviation (`St.Petersburg`) under an unfinished names list is no name: St., Mt., Ft.
/// are no initials (review r1). Repro sectioning-frontmatter/author_glued_place_abbreviation_stays_affiliation.
#[test]
fn author_glued_place_abbreviation_stays_affiliation() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_glued_place_abbreviation_stays_affiliation.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Ivan Petrov</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">St.Petersburg</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Oleg Sidorov</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">St.Petersburg</contact></creator>",
    ],
  );
}

/// 63g: svjour3's EPJ style links an author to its institutes and notes by key (svepjc3.clo:317-410):
/// `\thanksref{addr2,e1}` requests them, an institute's `\label{addr2}` labels its affiliation, a `\thankstext{e1}`
/// is a thanks contact so labelled; printed as the keys, they leaked and nothing linked (2304.02920). Repro
/// sectioning-frontmatter/svjour3_epj_thanksref_links_institutes_and_notes.
#[test]
fn svjour3_epj_thanksref_links_institutes_and_notes() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/svjour3_epj_thanksref_links_institutes_and_notes.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname><note class=\"ltx_note_frontmatter ltx_thanks_correspondence\" role=\"thanks\" xml:id=\"id1\">Corresponding author: bob@b.org</note><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ B</contact></creator>",
    ],
  );
}

/// 63g: an EPJ title's `\thanksref` gets its `\thankstext` as a note (a note is valid in a title, a contact is not),
/// an institute's own `\thanksref` leaves the institute's label alone — its note, cited by no author, kept — and both
/// authors keep Univ A (review r1). Repro sectioning-frontmatter/svjour3_epj_title_and_institute_notes.
#[test]
fn svjour3_epj_title_and_institute_notes() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/svjour3_epj_title_and_institute_notes.tex"
  );
  let (log, xml) = latexml::util::test::convert_with(tex, Some("ar5iv.sty"));
  assert_eq!(
    (
      super::perfect_kernel_batch46::error_count(&log),
      super::perfect_kernel_batch46::warning_count(&log)
    ),
    (0, 0),
    "{log}"
  );
  let title = xml
    .find("<title")
    .and_then(|at| latexml::util::test::xml_element(&xml[at..], "title", &[]));
  assert_eq!(
    title.as_deref(),
    Some(
      "<title>My Title<note class=\"ltx_note_frontmatter ltx_thanks_funding\" role=\"thanks\" xml:id=\"id1\">Grant note XYZ.</note></title>"
    ),
    "{xml}"
  );
  let found = super::perfect_kernel_batch61::creators_of(&xml);
  assert!(
    found
      == [
        "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A </contact></creator>",
        "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A </contact></creator>",
        "<creator role=\"author\"><note class=\"ltx_note_frontmatter ltx_thanks_note\" role=\"thanks\" xml:id=\"id2\">Visiting from Z.</note></creator>",
      ],
    "creators differ\n--- found:\n{}\n--- xml:\n{xml}",
    found.join("\n")
  );
}

/// The title element and the creators of `tex`, which converts with no error or warning.
fn assert_title_and_creators(tex: &str, title: &str, creators: &[&str]) {
  let (log, xml) = latexml::util::test::convert_with(tex, Some("ar5iv.sty"));
  assert_eq!(
    (
      super::perfect_kernel_batch46::error_count(&log),
      super::perfect_kernel_batch46::warning_count(&log)
    ),
    (0, 0),
    "{log}"
  );
  let found_title = xml
    .find("<title")
    .and_then(|at| latexml::util::test::xml_element(&xml[at..], "title", &[]));
  assert_eq!(found_title.as_deref(), Some(title), "{xml}");
  let found = super::perfect_kernel_batch61::creators_of(&xml);
  assert!(
    found == creators,
    "creators differ\n--- found:\n{}\n--- xml:\n{xml}",
    found.join("\n")
  );
}

/// 63h: the AAS title footnote (`\title{..\altaffilmark{1}}` answered by `\altaffiltext{1}`; 0704.0478, 1001.2402) is
/// the title's note: a title holds no contact (title_model), so the relocated annotation becomes the frontmatter thanks
/// note `\lx@add@thanks` makes. An author's `\altaffiltext` stays its contact. Repro
/// sectioning-frontmatter/aastex_title_altaffilmark_is_a_note.
#[test]
fn aastex_title_altaffilmark_is_a_note() {
  assert_title_and_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/aastex_title_altaffilmark_is_a_note.tex"
    ),
    "<title>Velocity Dispersions in M82<note class=\"ltx_note_frontmatter ltx_thanks_note\" role=\"thanks\" xml:id=\"id1\">Based on observations made at the Keck Observatory.</note></title>",
    &[
      "<creator role=\"author\"><personname>Nate Able</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact><contact name=\"Alternate Affiliation:\u{a0}\" role=\"altaffiliation\">Hubble Fellow.</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>James Baker</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact></creator>",
    ],
  );
}

/// 63h: elsarticle's `\tnoteref{t1}` in `\title` and its `\tnotetext[t1]{..}` (the Elsevier template's title note) give
/// the title a frontmatter note, as for aastex. Repro sectioning-frontmatter/elsarticle_title_tnote_is_a_note.
#[test]
fn elsarticle_title_tnote_is_a_note() {
  assert_title_and_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/elsarticle_title_tnote_is_a_note.tex"
    ),
    "<title>My Title<note class=\"ltx_note_frontmatter ltx_thanks_funding\" role=\"thanks\" xml:id=\"id1\">Funded by grant XYZ.</note></title>",
    &[
      "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Email:\u{a0}\" role=\"email\">ann@a.org</contact><contact name=\"Corresponding author:\u{a0}\" role=\"correspondent\">Corresponding author.</contact><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname><contact name=\"Note:\u{a0}\" role=\"note\">Visiting from Z.</contact><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ B</contact></creator>",
    ],
  );
}

/// 63h: one unlabelled svjour3 `\institute` under two authors is shared by both, as the PDF prints it once below them:
/// the trailing creator of OXIDIZED_DESIGN_DIVERGENCES #159, not the last author's alone (63g review r2). Repro
/// sectioning-frontmatter/svjour3_epj_one_institute_shared.
#[test]
fn svjour3_epj_one_institute_shared() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/svjour3_epj_one_institute_shared.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Ann Able</personname></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname></creator>",
      "<creator role=\"author\"><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact></creator>",
    ],
  );
}

/// 63h: one author and an unlabelled svjour3 `\institute`: the institute and its e-mail are that author's (1709.03696,
/// 2003.07473; Perl alike), not a name-less creator's (OXIDIZED_DESIGN_DIVERGENCES #159). Repro
/// sectioning-frontmatter/svjour3_sole_author_keeps_the_institute.
#[test]
fn svjour3_sole_author_keeps_the_institute() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/svjour3_sole_author_keeps_the_institute.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A </contact><contact name=\"E-mail: \" role=\"email\">ann@a.org</contact></creator>",
    ],
  );
}

/// 63h: the same for llncs, whose `\institute` without `\inst` went to a name-less creator even under one author (#159).
/// Repro sectioning-frontmatter/llncs_sole_author_keeps_the_institute.
#[test]
fn llncs_sole_author_keeps_the_institute() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/llncs_sole_author_keeps_the_institute.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A </contact><contact name=\"E-mail\u{a0}\" role=\"email\">ann@a.org</contact></creator>",
    ],
  );
}

/// 63h: as many unlabelled svjour3 institutes as authors pair by position, the i-th to the i-th author, as
/// `distribute_upfront_contacts` and Perl's numeric fallback pair them (63h review r1). Repro
/// sectioning-frontmatter/svjour3_two_institutes_pair_by_position.
#[test]
fn svjour3_two_institutes_pair_by_position() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/svjour3_two_institutes_pair_by_position.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ B</contact></creator>",
    ],
  );
}

/// 63h: a svjour3 `\and` piece holding "A and B" is two people (1406.5162, 2105.12728), split as llncs splits its pieces.
/// Repro sectioning-frontmatter/author_svjour3_and_piece_splits.
#[test]
fn author_svjour3_and_piece_splits() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_svjour3_and_piece_splits.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Ann Able</personname></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Carl Cole</personname></creator>",
      "<creator role=\"author\"><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact></creator>",
    ],
  );
}

/// 63h: an elsarticle `\fntext[t1]` no `\fnref` cites does not reach the title's `\tnoteref{t1}` by the prefix-stripped
/// label fallback, which serves creators only; it is kept on the sole author, with the orphan warning (63h review r1,
/// r2).
/// Repro sectioning-frontmatter/elsarticle_orphan_fntext_stays_off_the_title.
#[test]
fn elsarticle_orphan_fntext_stays_off_the_title() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/elsarticle_orphan_fntext_stays_off_the_title.tex"
  );
  let (log, xml) = latexml::util::test::convert_with(tex, Some("ar5iv.sty"));
  assert_eq!(
    (
      super::perfect_kernel_batch46::error_count(&log),
      super::perfect_kernel_batch46::warning_count(&log)
    ),
    (0, 1),
    "{log}"
  );
  assert!(
    log.contains("Orphaned frontmatter annotation couldn't find target for label=fn:t1"),
    "{log}"
  );
  let title = xml
    .find("<title")
    .and_then(|at| latexml::util::test::xml_element(&xml[at..], "title", &[]));
  assert_eq!(
    title.as_deref(),
    Some(
      "<title>My Title<note class=\"ltx_note_frontmatter ltx_thanks_funding\" role=\"thanks\" xml:id=\"id1\">Funded by grant XYZ.</note></title>"
    ),
    "{xml}"
  );
  let found = super::perfect_kernel_batch61::creators_of(&xml);
  assert!(
    found
      == [
        "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact><contact name=\"Note:\u{a0}\" role=\"note\">Visiting from Z.</contact></creator>",
      ],
    "creators differ\n--- found:\n{}\n--- xml:\n{xml}",
    found.join("\n")
  );
}

/// 63h: svjour3 `\at` institutes go to the author whose surname each names, in any order (`B. Baker \at Univ B \and
/// A. Able \at Univ A`; the abbreviated names miss the exact match; 63h review r2). Repro
/// sectioning-frontmatter/svjour3_at_institutes_link_by_surname_in_any_order.
#[test]
fn svjour3_at_institutes_link_by_surname_in_any_order() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/svjour3_at_institutes_link_by_surname_in_any_order.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ B</contact></creator>",
    ],
  );
}

/// 63h: a name-only svjour3 institute piece (`\institute{A. Able \and B. Baker \at Univ A}`; 1406.4834, 1603.00632,
/// 1709.00485, 2311.02489, 2208.03119) pairs nothing by position: the institute stays shared, as at 63g (63h review r2).
/// The name-only piece "A. Able" pinned as a shared affiliation (at 63g it sat on the last author) is the known residual of
/// RED svjour3_name_only_piece_shares_the_next_institute: turning that green updates this guard.
/// Repro sectioning-frontmatter/svjour3_name_only_piece_keeps_the_institute_shared.
#[test]
fn svjour3_name_only_piece_keeps_the_institute_shared() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/svjour3_name_only_piece_keeps_the_institute_shared.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Ann Able</personname></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname></creator>",
      "<creator role=\"author\"><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">A. Able</contact><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A </contact><contact name=\"E-mail: \" role=\"email\">ann@a.org</contact></creator>",
    ],
  );
}

/// 63h: lettered elsarticle addresses no author cites are no numbering to pair by: both stay shared (63h review r2).
/// Repro sectioning-frontmatter/elsarticle_lettered_addresses_no_one_cites_stay_shared.
#[test]
fn elsarticle_lettered_addresses_no_one_cites_stay_shared() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/elsarticle_lettered_addresses_no_one_cites_stay_shared.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Ann Able</personname></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname></creator>",
      "<creator role=\"author\"><contact name=\"Address:\u{a0}\" role=\"address\">Univ A</contact><contact name=\"Address:\u{a0}\" role=\"address\">Univ B</contact></creator>",
    ],
  );
}

/// 63h: an svjour3 `\at` piece naming several authors (`Olivier Augereau, Koichi Kise, and Motoi Iwata \at …`,
/// 1811.03214; 1001.2544) is every one of theirs. Repro sectioning-frontmatter/svjour3_at_piece_naming_several_authors.
#[test]
fn svjour3_at_piece_naming_several_authors() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/svjour3_at_piece_naming_several_authors.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ B </contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Carl Cole</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ B </contact></creator>",
      "<creator role=\"author\"><contact name=\"E-mail: \" role=\"email\">carl@b.org</contact></creator>",
    ],
  );
}

/// 63i: revtex authors all first, then `$^{a}$`-marked `\affiliation`s (2011.01984, 2301.08449): each affiliation is
/// the authors' whose names show its mark, not every author's (revtex's group rule). Repro
/// sectioning-frontmatter/revtex_marked_affiliations_link_by_mark.
#[test]
fn revtex_marked_affiliations_link_by_mark() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/revtex_marked_affiliations_link_by_mark.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Ann Able<sup xml:id=\"id1\"><text font=\"italic\" xml:id=\"id1.1\">a</text></sup></personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker<sup xml:id=\"id2\"><text font=\"italic\" xml:id=\"id2.1\">b</text></sup></personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ B</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Cat Cole<sup xml:id=\"id3\"><text font=\"italic\" xml:id=\"id3.1\">a</text></sup></personname><contact name=\"Email:\u{a0}\" role=\"email\">Corresponding author: ann@a.org</contact><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact></creator>",
    ],
  );
}

/// 63i: svjour3 marked authors and institutes listed out of order link by mark (63h review r1). Repro
/// sectioning-frontmatter/svjour3_marked_institutes_link_by_mark.
#[test]
fn svjour3_marked_institutes_link_by_mark() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/svjour3_marked_institutes_link_by_mark.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Ann Able<sup xml:id=\"id1\"><text font=\"italic\" xml:id=\"id1.1\">1</text></sup></personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker<sup xml:id=\"id2\"><text font=\"italic\" xml:id=\"id2.1\">2</text></sup></personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ B</contact></creator>",
    ],
  );
}

/// 63i: author marks link institutes only where every mark is answered: one `\institute` whose lines carry numbers
/// (`{1} Univ A \\ {2} Univ B`, aa astro-ph0305539) stays shared, not the first author's by its `affiliation:1`. Repro
/// sectioning-frontmatter/institute_with_numbered_lines_is_no_marked_list.
#[test]
fn institute_with_numbered_lines_is_no_marked_list() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/institute_with_numbered_lines_is_no_marked_list.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Ann Able<sup xml:id=\"id1\"><text font=\"italic\" xml:id=\"id1.1\">1,2</text></sup></personname></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker<sup xml:id=\"id2\"><text font=\"italic\" xml:id=\"id2.1\">2</text></sup></personname></creator>",
      "<creator role=\"author\"><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">1 Univ A, City A<break/>2 Univ B, City B</contact></creator>",
    ],
  );
}

/// 63i: an interleaved revtex group whose author shows `$^{2,\dagger}$` keeps its affiliation (the group rule; 63i review r1). Repro
/// sectioning-frontmatter/revtex_interleaved_mark_with_symbol_keeps_the_group_rule.
#[test]
fn revtex_interleaved_mark_with_symbol_keeps_the_group_rule() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/revtex_interleaved_mark_with_symbol_keeps_the_group_rule.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Ann Able<sup xml:id=\"id1\"><text font=\"italic\" xml:id=\"id1.1\">1</text></sup></personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><sup xml:id=\"id2\"><text font=\"italic\" xml:id=\"id2.1\">1</text></sup>Univ A</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker<sup xml:id=\"id3\"><text font=\"italic\" xml:id=\"id3.1\">2,†</text></sup></personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><sup xml:id=\"id4\"><text font=\"italic\" xml:id=\"id4.1\">2</text></sup>Univ B</contact></creator>",
    ],
  );
}

/// 63i: interleaved revtex groups keep the group rule though one author mark is answered by none (63i review r1). Repro
/// sectioning-frontmatter/revtex_interleaved_group_with_unanswered_mark.
#[test]
fn revtex_interleaved_group_with_unanswered_mark() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/revtex_interleaved_group_with_unanswered_mark.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Ann Able<sup xml:id=\"id1\"><text font=\"italic\" xml:id=\"id1.1\">1</text></sup></personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><sup xml:id=\"id2\"><text font=\"italic\" xml:id=\"id2.1\">1</text></sup>Univ A</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker<sup xml:id=\"id3\"><text font=\"italic\" xml:id=\"id3.1\">1,5</text></sup></personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><sup xml:id=\"id4\"><text font=\"italic\" xml:id=\"id4.1\">1</text></sup>Univ A</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Cat Cole<sup xml:id=\"id5\"><text font=\"italic\" xml:id=\"id5.1\">2</text></sup></personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><sup xml:id=\"id6\"><text font=\"italic\" xml:id=\"id6.1\">2</text></sup>Univ B</contact></creator>",
    ],
  );
}

/// 63i: a revtex `\affiliation` with a superscript inside it (`Laboratory for $^{3}$He`) is no marked line (63i review r1). Repro
/// sectioning-frontmatter/revtex_affiliation_with_inner_superscript_is_no_marked_line.
#[test]
fn revtex_affiliation_with_inner_superscript_is_no_marked_line() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/revtex_affiliation_with_inner_superscript_is_no_marked_line.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Ann Able<sup xml:id=\"id1\"><text font=\"italic\" xml:id=\"id1.1\">1</text></sup></personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><sup xml:id=\"id2\"><text font=\"italic\" xml:id=\"id2.1\">1</text></sup>Univ A</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker<sup xml:id=\"id3\"><text font=\"italic\" xml:id=\"id3.1\">2</text></sup></personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Laboratory for <sup xml:id=\"id4\"><text font=\"italic\" xml:id=\"id4.1\">3</text></sup>He Physics, Univ B</contact></creator>",
    ],
  );
}

/// 63i: revtex authors first, one marked `$^{b,\dagger}$`, link to their marked affiliations by the letter (63i review r1). Repro
/// sectioning-frontmatter/revtex_marked_affiliations_with_symbol_marks_link_by_mark.
#[test]
fn revtex_marked_affiliations_with_symbol_marks_link_by_mark() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/revtex_marked_affiliations_with_symbol_marks_link_by_mark.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Ann Able<sup xml:id=\"id1\"><text font=\"italic\" xml:id=\"id1.1\">a</text></sup></personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker<sup xml:id=\"id2\"><text font=\"italic\" xml:id=\"id2.1\">b,†</text></sup></personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ B</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Cat Cole<sup xml:id=\"id3\"><text font=\"italic\" xml:id=\"id3.1\">a</text></sup></personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact></creator>",
    ],
  );
}

/// 63i: svjour3 marked institute lines link by mark though one carries its own `\thanksref` (63i review r1). Repro
/// sectioning-frontmatter/svjour3_marked_institutes_with_thanksref_link_by_mark.
#[test]
fn svjour3_marked_institutes_with_thanksref_link_by_mark() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/svjour3_marked_institutes_with_thanksref_link_by_mark.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Ann Able<sup xml:id=\"id1\"><text font=\"italic\" xml:id=\"id1.1\">1</text></sup></personname><note class=\"ltx_note_frontmatter ltx_thanks_note\" role=\"thanks\" xml:id=\"id2\">e-mail: ann@a.org</note><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker<sup xml:id=\"id3\"><text font=\"italic\" xml:id=\"id3.1\">2</text></sup></personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ B</contact></creator>",
      "<creator role=\"author\"><note class=\"ltx_note_frontmatter ltx_thanks_note\" role=\"thanks\" xml:id=\"id4\">Institute note.</note></creator>",
    ],
  );
}

/// 63i: an svjour3 institute led by a symbol (`$^{\star}$`) is no marked list: the institutes pair by position (63i review r1). Repro
/// sectioning-frontmatter/svjour3_symbol_marked_institute_pairs_by_position.
#[test]
fn svjour3_symbol_marked_institute_pairs_by_position() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/svjour3_symbol_marked_institute_pairs_by_position.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Ann Able</personname><note class=\"ltx_note_frontmatter ltx_thanks_note\" role=\"thanks\" xml:id=\"id1\">e-mail: ann@a.org</note><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><sup xml:id=\"id2\"><text font=\"italic\" xml:id=\"id2.1\">⋆</text></sup>Univ A</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ B</contact></creator>",
      "<creator role=\"author\"><note class=\"ltx_note_frontmatter ltx_thanks_note\" role=\"thanks\" xml:id=\"id3\">Institute note.</note></creator>",
    ],
  );
}

/// 63i: a `$^{\mathrm{a}}$` mark is read alike on the author and the affiliation: an interleaved group keeps the group rule (63i review r2). Repro
/// sectioning-frontmatter/revtex_mathrm_mark_interleaved_group_keeps_the_group_rule.
#[test]
fn revtex_mathrm_mark_interleaved_group_keeps_the_group_rule() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/revtex_mathrm_mark_interleaved_group_keeps_the_group_rule.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Ann Able<sup xml:id=\"id1\"><text font=\"italic\" xml:id=\"id1.1\">a</text></sup></personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><sup xml:id=\"id2\"><text font=\"italic\" xml:id=\"id2.1\">a</text></sup>Univ A</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><sup xml:id=\"id3\"><text font=\"italic\" xml:id=\"id3.1\">a</text></sup>Univ A</contact></creator>",
    ],
  );
}

/// 63i: revtex authors first with `$^{\mathrm{a}}$` marks link to their marked affiliations (63i review r2). Repro
/// sectioning-frontmatter/revtex_mathrm_marked_affiliations_link_by_mark.
#[test]
fn revtex_mathrm_marked_affiliations_link_by_mark() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/revtex_mathrm_marked_affiliations_link_by_mark.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Ann Able<sup xml:id=\"id1\"><text font=\"italic\" xml:id=\"id1.1\">a</text></sup></personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker<sup xml:id=\"id2\"><text font=\"italic\" xml:id=\"id2.1\">b</text></sup></personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ B</contact></creator>",
    ],
  );
}

/// 63i: `\textsuperscript` marks count as `^` ones in the group check: an interleaved group keeps the group rule (63i review r2). Repro
/// sectioning-frontmatter/revtex_textsuperscript_interleaved_group_keeps_the_group_rule.
#[test]
fn revtex_textsuperscript_interleaved_group_keeps_the_group_rule() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/revtex_textsuperscript_interleaved_group_keeps_the_group_rule.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Ann Able<sup xml:id=\"id1\">1</sup></personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><sup xml:id=\"id2\">1</sup>Univ A</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><sup xml:id=\"id3\">1</sup>Univ A</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Cat Cole<sup xml:id=\"id4\">2</sup></personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><sup xml:id=\"id5\">2</sup>Univ B</contact></creator>",
    ],
  );
}

/// 63i: an interleaved group keeps its affiliation though no author shows its mark (63i review r2). Repro
/// sectioning-frontmatter/revtex_interleaved_typo_mark_keeps_the_group_rule.
#[test]
fn revtex_interleaved_typo_mark_keeps_the_group_rule() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/revtex_interleaved_typo_mark_keeps_the_group_rule.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Ann Able<sup xml:id=\"id1\"><text font=\"italic\" xml:id=\"id1.1\">1</text></sup></personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><sup xml:id=\"id2\"><text font=\"italic\" xml:id=\"id2.1\">1</text></sup>Univ A</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker<sup xml:id=\"id3\"><text font=\"italic\" xml:id=\"id3.1\">1</text></sup></personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><sup xml:id=\"id4\"><text font=\"italic\" xml:id=\"id4.1\">1</text></sup>Univ A</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Cat Cole<sup xml:id=\"id5\"><text font=\"italic\" xml:id=\"id5.1\">2</text></sup></personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><sup xml:id=\"id6\"><text font=\"italic\" xml:id=\"id6.1\">3</text></sup>Univ B</contact></creator>",
    ],
  );
}

/// 63j: algorithmic's `\REQUIRE`/`\ENSURE` (`\item[\algorithmicrequire]`, algorithmic.sty:155) tag their lines with
/// their labels ("Input:" as 2201.01230 renames it), not the line counter's "0:". Repro
/// list-structure/algorithmic_labelled_item_shows_its_label.
#[test]
fn algorithmic_labelled_item_shows_its_label() {
  let (log, xml) = latexml::util::test::convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/list-structure/algorithmic_labelled_item_shows_its_label.tex"
    ),
    None,
  );
  assert_eq!(
    (
      super::perfect_kernel_batch46::error_count(&log),
      super::perfect_kernel_batch46::warning_count(&log)
    ),
    (0, 0),
    "{log}"
  );
  let listing = xml
    .find("<listing ")
    .and_then(|at| latexml::util::test::xml_element(&xml[at..], "listing", &[]));
  assert_eq!(
    listing.as_deref(),
    Some(
      "<listing framed=\"topbottom\"><listingline xml:id=\"alg1.l0\"><tags><tag><text font=\"bold\">Input:</text></tag></tags>\u{2002}the data</listingline><listingline xml:id=\"alg1.l0a\"><tags><tag><text font=\"bold\">Ensure:</text></tag></tags>\u{2002}the model</listingline><listingline xml:id=\"alg1.l1\"><tags><tag><text fontsize=\"80%\">1:</text></tag><tag role=\"refnum\">1</tag></tags>\u{2002}x</listingline></listing>"
    ),
    "{xml}"
  );
}

/// The `<tags>` of the equation carrying `label` in `xml`.
fn tags_of_labelled_equation(xml: &str, label: &str) -> Option<String> {
  let at = xml.find(&format!("<equation labels=\"{label}\""))?;
  let eq = &xml[at..];
  let tags = eq.find("<tags>")?;
  latexml::util::test::xml_element(&eq[tags..], "tags", &[])
}

/// 63j: an eqnarray row holding only `\label` keeps its number and label (latex.ltx:15795 numbers it; 1011.4399,
/// hep-th9412215, 1305.3072): the last row's (1) joins the unnumbered equation above, a middle one's (2) stands alone.
/// Repro alignment/eqnarray_empty_labelled_row_keeps_number.
#[test]
fn eqnarray_empty_labelled_row_keeps_number() {
  let (log, xml) = latexml::util::test::convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/alignment/eqnarray_empty_labelled_row_keeps_number.tex"
    ),
    None,
  );
  assert_eq!(
    (
      super::perfect_kernel_batch46::error_count(&log),
      super::perfect_kernel_batch46::warning_count(&log)
    ),
    (0, 0),
    "{log}"
  );
  assert_eq!(
    (
      tags_of_labelled_equation(&xml, "LABEL:last").as_deref(),
      tags_of_labelled_equation(&xml, "LABEL:mid").as_deref()
    ),
    (
      Some("<tags><tag>(1)</tag><tag role=\"refnum\">1</tag></tags>"),
      Some("<tags><tag>(2)</tag><tag role=\"refnum\">2</tag></tags>")
    ),
    "{xml}"
  );
}

/// 63j: amsmath's align and gather keep a row holding only `\label`, with its number (Perl amsmath.sty.ltxml:483
/// "Numbered ones still show in print out!!!"). Repro alignment/align_empty_labelled_row_keeps_number.
#[test]
fn align_empty_labelled_row_keeps_number() {
  let (log, xml) = latexml::util::test::convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/alignment/align_empty_labelled_row_keeps_number.tex"
    ),
    None,
  );
  assert_eq!(
    (
      super::perfect_kernel_batch46::error_count(&log),
      super::perfect_kernel_batch46::warning_count(&log)
    ),
    (0, 0),
    "{log}"
  );
  assert_eq!(
    (
      tags_of_labelled_equation(&xml, "LABEL:al").as_deref(),
      tags_of_labelled_equation(&xml, "LABEL:ga").as_deref()
    ),
    (
      Some("<tags><tag>(2)</tag><tag role=\"refnum\">2</tag></tags>"),
      Some("<tags><tag>(4)</tag><tag role=\"refnum\">4</tag></tags>")
    ),
    "{xml}"
  );
}

/// 63j: an author list's "and~" (the "and" tied to the last name: 2011.10474, 2408.09035, 1406.6147) separates as
/// " and " does. Repro sectioning-frontmatter/author_and_tied_to_the_last_name_splits.
#[test]
fn author_and_tied_to_the_last_name_splits() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_and_tied_to_the_last_name_splits.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Ann\u{a0}Able</personname></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob\u{a0}Baker</personname></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Cat\u{a0}Cole</personname></creator>",
    ],
  );
}

/// 63j review: an empty row holding `\label` but no number (eqnarray's `\nonumber`, align*) names no number, so it is
/// still dropped and its label left unresolved, while align's numbered one keeps (2). Repro
/// alignment/unnumbered_empty_labelled_row_stays_unresolved.
#[test]
fn unnumbered_empty_labelled_row_stays_unresolved() {
  let (log, xml) = latexml::util::test::convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/alignment/unnumbered_empty_labelled_row_stays_unresolved.tex"
    ),
    None,
  );
  assert_eq!(
    (
      super::perfect_kernel_batch46::error_count(&log),
      super::perfect_kernel_batch46::warning_count(&log)
    ),
    (0, 0),
    "{log}"
  );
  let labelled: Vec<&str> = xml
    .match_indices(" labels=\"")
    .map(|(at, attr)| {
      let value = &xml[at + attr.len()..];
      &value[..value.find('"').unwrap_or(0)]
    })
    .collect();
  assert_eq!(
    (
      labelled,
      tags_of_labelled_equation(&xml, "LABEL:nu").as_deref()
    ),
    (
      vec!["LABEL:nu"],
      Some("<tags><tag>(2)</tag><tag role=\"refnum\">2</tag></tags>")
    ),
    "{xml}"
  );
}

/// 63j review: algpseudocode's `\Require`/`\Ensure` (`\item[\algorithmicrequire]`, algpseudocode.sty:78-79) tag their
/// lines with their labels, the line counter unstepped. Repro list-structure/algpseudocode_require_shows_its_label.
#[test]
fn algpseudocode_require_shows_its_label() {
  let (log, xml) = latexml::util::test::convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/list-structure/algpseudocode_require_shows_its_label.tex"
    ),
    None,
  );
  assert_eq!(
    (
      super::perfect_kernel_batch46::error_count(&log),
      super::perfect_kernel_batch46::warning_count(&log)
    ),
    (0, 0),
    "{log}"
  );
  let listing = xml
    .find("<listing ")
    .and_then(|at| latexml::util::test::xml_element(&xml[at..], "listing", &[]));
  assert_eq!(
    listing.as_deref(),
    Some(
      "<listing framed=\"topbottom\"><listingline xml:id=\"alg1.l0\"><tags><tag><text font=\"bold\">Require:</text></tag></tags>data</listingline><listingline xml:id=\"alg1.l0a\"><tags><tag><text font=\"bold\">Ensure:</text></tag></tags>out</listingline><listingline xml:id=\"alg1.l1\"><tags><tag><text fontsize=\"80%\">1:</text></tag><tag role=\"refnum\">1</tag></tags>x</listingline></listing>"
    ),
    "{xml}"
  );
}

/// 63j review: algpseudocode's `\Statex` (`\item[]`, algorithmicx.sty:632) is an unnumbered line whose own text may
/// open with "[" (`\Statex [Phase one] begins`); a braced label keeps its "]". Repro
/// list-structure/algpseudocode_statex_keeps_its_bracket_text.
#[test]
fn algpseudocode_statex_keeps_its_bracket_text() {
  let (log, xml) = latexml::util::test::convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/list-structure/algpseudocode_statex_keeps_its_bracket_text.tex"
    ),
    None,
  );
  assert_eq!(
    (
      super::perfect_kernel_batch46::error_count(&log),
      super::perfect_kernel_batch46::warning_count(&log)
    ),
    (0, 0),
    "{log}"
  );
  let listing = xml
    .find("<listing")
    .and_then(|at| latexml::util::test::xml_element(&xml[at..], "listing", &[]));
  assert_eq!(
    listing.as_deref(),
    Some(
      "<listing><listingline xml:id=\"algx1.l0\"><tags><tag><text font=\"bold\">Require:</text></tag></tags>data</listingline><listingline xml:id=\"algx1.l0a\">[Phase one] begins</listingline><listingline xml:id=\"algx1.l1\"><tags><tag><text fontsize=\"80%\">1:</text></tag><tag role=\"refnum\">1</tag></tags>x</listingline><listingline xml:id=\"algx1.l1a\"><tags><tag>a]b</tag></tags>c</listingline><listingline xml:id=\"algx1.l1b\"><tags><tag><text font=\"bold\">Ensure:</text></tag></tags>out</listingline><listingline xml:id=\"algx1.l2\"><tags><tag><text fontsize=\"80%\">2:</text></tag><tag role=\"refnum\">2</tag></tags>y</listingline></listing>"
    ),
    "{xml}"
  );
}

/// 63j review: algorithmic's `\item[{a]b}]` keeps its label whole and `\item[]` is an untagged line, the line counter
/// stepped by `\STATE` alone. Repro list-structure/algorithmic_empty_and_bracketed_item_labels.
#[test]
fn algorithmic_empty_and_bracketed_item_labels() {
  let (log, xml) = latexml::util::test::convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/list-structure/algorithmic_empty_and_bracketed_item_labels.tex"
    ),
    None,
  );
  assert_eq!(
    (
      super::perfect_kernel_batch46::error_count(&log),
      super::perfect_kernel_batch46::warning_count(&log)
    ),
    (0, 0),
    "{log}"
  );
  let listing = xml
    .find("<listing")
    .and_then(|at| latexml::util::test::xml_element(&xml[at..], "listing", &[]));
  assert_eq!(
    listing.as_deref(),
    Some(
      "<listing><listingline xml:id=\"algx1.l0\"><tags><tag><text font=\"bold\">Require:</text></tag></tags>\u{2002}data</listingline><listingline xml:id=\"algx1.l1\"><tags><tag><text fontsize=\"80%\">1:</text></tag><tag role=\"refnum\">1</tag></tags>\u{2002}x</listingline><listingline xml:id=\"algx1.l1a\"><tags><tag>a]b</tag></tags>\u{2002} c</listingline><listingline xml:id=\"algx1.l1b\">\u{2002} blank</listingline><listingline xml:id=\"algx1.l2\"><tags><tag><text fontsize=\"80%\">2:</text></tag><tag role=\"refnum\">2</tag></tags>\u{2002}y</listingline></listing>"
    ),
    "{xml}"
  );
}

/// Core XML of `tex` (0 errors, 0 warnings) through the post-processor's CrossRef pass, as XML (no stylesheet).
fn post_xml(tex: &str) -> String {
  let (log, xml) = latexml::util::test::convert_with(tex, None);
  assert_eq!(
    (
      super::perfect_kernel_batch46::error_count(&log),
      super::perfect_kernel_batch46::warning_count(&log)
    ),
    (0, 0),
    "{log}"
  );
  latexml_core::util::logger::bind_log();
  let opts = latexml::post::PostOptions {
    pmml:                      true,
    cmml:                      false,
    keep_xmath:                false,
    stylesheet:                None,
    destination:               None,
    source_directory:          None,
    site_directory:            None,
    search_paths:              &[],
    nodefaultresources:        true,
    css_files:                 &[],
    js_files:                  &[],
    noinvisibletimes:          false,
    plane1:                    true,
    hackplane1:                false,
    mathtex:                   false,
    url_style:                 latexml_post::crossref::UrlStyle::File,
    navigationtoc:             None,
    schemadocs:                false,
    split:                     false,
    split_xpath:               None,
    split_naming:              None,
    xslt_parameters:           &[],
    graphics_svg_threshold_kb: 0,
    graphicimages:             false,
    timestamp:                 None,
    icon:                      None,
    whatsout:                  latexml_post::extract::Whatsout::default(),
  };
  let out = latexml::post::run_post_processing(&xml, &opts);
  let log = latexml_core::util::logger::flush_log();
  assert_eq!(
    (
      latexml::util::test::error_count(&log),
      super::perfect_kernel_batch46::warning_count(&log)
    ),
    (0, 0),
    "POST diagnostics:\n{log}"
  );
  out
}

/// The `<ref>` element filled for `idref` in the post XML's table of contents.
fn toc_ref(xml: &str, idref: &str) -> Option<String> {
  let toc = xml.find("<toclist")?;
  let at = toc + xml[toc..].find(&format!("<ref idref=\"{idref}\""))?;
  latexml::util::test::xml_element(&xml[at..], "ref", &[])
}

/// 63k: a `\ref` in a section title fills the TOC entry and the tooltips that reuse the title, unlinked (Perl
/// CrossRef.pm:882-904 `fillInTitle`; 1011.3492, 1111.3672). Repro sectioning-frontmatter/ref_in_section_title_fills_toc_and_tooltip.
#[test]
fn ref_in_section_title_fills_toc_and_tooltip() {
  let xml = post_xml(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/ref_in_section_title_fills_toc_and_tooltip.tex"
  ));
  let see = xml
    .find("See Section")
    .and_then(|at| latexml::util::test::xml_element(&xml[at..], "ref", &[]));
  assert_eq!(
    (
      toc_ref(&xml, "S1").as_deref(),
      toc_ref(&xml, "S1.SS1").as_deref(),
      see.as_deref()
    ),
    (
      Some(
        "<ref idref=\"S1\" show=\"toctitle\"><text class=\"ltx_ref_title\"><tag close=\" \">1</tag>Proof of Theorem\u{a0}<text class=\"ltx_ref_tag\">1</text></text></ref>"
      ),
      Some(
        "<ref idref=\"S1.SS1\" show=\"toctitle\" title=\"In 1 Proof of Theorem 1\"><text class=\"ltx_ref_title\"><tag close=\" \">1.1</tag>Case (<text class=\"ltx_ref_tag\">1</text>) and\u{a0}(<text class=\"ltx_ref_tag\">1</text>)</text></ref>"
      ),
      Some(
        "<ref idref=\"S1\" labelref=\"LABEL:sec\" title=\"1 Proof of Theorem 1\"><text class=\"ltx_ref_tag\">1</text></ref>"
      )
    ),
    "{xml}"
  );
}

/// 63k: `\nameref` shows the section's title without its number (Perl CrossRef.pm:774-777), in the text and inside
/// another section's title, in its TOC entry (KNOWN_PERL_ERRORS #531). Repro
/// sectioning-frontmatter/nameref_drops_the_section_number.
#[test]
fn nameref_drops_the_section_number() {
  let xml = post_xml(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/nameref_drops_the_section_number.tex"
  ));
  let see = xml
    .find("See <ref")
    .and_then(|at| latexml::util::test::xml_element(&xml[at + 4..], "ref", &[]));
  assert_eq!(
    (see.as_deref(), toc_ref(&xml, "S3").as_deref()),
    (
      Some(
        "<ref class=\"ltx_refmacro_nameref\" idref=\"S2\" labelref=\"LABEL:tgt\" show=\"title\"><text class=\"ltx_ref_title\">Target</text></ref>"
      ),
      Some(
        "<ref idref=\"S3\" show=\"toctitle\"><text class=\"ltx_ref_title\"><tag close=\" \">3</tag>After <text class=\"ltx_ref_title\">Target</text></text></ref>"
      )
    ),
    "{xml}"
  );
}

/// 63k: each `\author` of the JHEP family adds an author (JHEP.cls:513, JHEP3.cls:764, PoS.cls:666; astro-ph0611258),
/// its `E-mail:` line an email named by that label. Repro sectioning-frontmatter/jhep_author_calls_accumulate.
#[test]
fn jhep_author_calls_accumulate() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/jhep_author_calls_accumulate.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Dept. of Physics, Univ. A</contact><contact name=\"E-mail: \" role=\"email\">ann@a.edu</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Dept. of Physics, Univ. B</contact><contact name=\"E-mail: \" role=\"email\">bob@b.edu</contact></creator>",
    ],
  );
}

/// 63k: an author block's email line is the email, its label (`E-mail:`, `Emails:`) the contact's name; a lone class
/// `\email{x}` line adds one contact; a group's shared line gives each address to the name it spells (0911.0082), and
/// a `$^a$` in a footnote marks no name (0911.0568). Repro sectioning-frontmatter/author_email_line_label_is_its_name.
#[test]
fn author_email_line_label_is_its_name() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_email_line_label_is_its_name.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact><contact name=\"Email:\u{a0}\" role=\"email\">ann@a.edu</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ B</contact><contact name=\"E-mail: \" role=\"email\"><text font=\"typewriter\" xml:id=\"id1\">bob@b.edu</text></contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Cat Cole</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ C</contact><contact name=\"Emails: \" role=\"email\">cat@c.edu</contact><contact name=\"Emails: \" role=\"email\">cc@c.edu</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Dan Doe</personname><contact name=\"E-mail: \" role=\"email\">dan.doe@d.edu</contact><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ D</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Eve Eng</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ D</contact><contact name=\"E-mail: \" role=\"email\">eve.eng@d.edu</contact></creator>",
    ],
  );
}

/// 63k review: an author block's unlabelled class `\email` keeps the name its class gives ("Email address: ",
/// amsart), and a label followed by a spacing command (`E-mail:\ \email{x}`) names its one contact. Repro
/// sectioning-frontmatter/author_class_email_line_keeps_its_name.
#[test]
fn author_class_email_line_keeps_its_name() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_class_email_line_keeps_its_name.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact><contact name=\"Email address: \" role=\"email\">ann@a.edu</contact></creator>",
      "<creator before=\" and \" role=\"author\"><personname>Bob Baker</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ B</contact><contact name=\"E-mail: \" role=\"email\">bob@b.edu</contact></creator>",
    ],
  );
}

/// 63k review: a display copy keeps its `xml:` attributes namespaced — the TOC's copy of a `\foreignlanguage` title
/// was `<text lang="de">`, which the schema refuses — cloned as stored, or rebuilt around a filled `\ref`. Repro sectioning-frontmatter/toc_copy_keeps_its_language.
#[test]
fn toc_copy_keeps_its_language() {
  let xml = post_xml(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/toc_copy_keeps_its_language.tex"
  ));
  assert_eq!(
    (
      toc_ref(&xml, "S1").as_deref(),
      toc_ref(&xml, "S2").as_deref()
    ),
    (
      Some(
        "<ref idref=\"S1\" show=\"toctitle\"><text class=\"ltx_ref_title\"><tag close=\" \">1</tag><text xml:lang=\"de\">Beweis</text></text></ref>"
      ),
      Some(
        "<ref idref=\"S2\" show=\"toctitle\"><text class=\"ltx_ref_title\"><tag close=\" \">2</tag><text xml:lang=\"de\">Beweis von Satz <text class=\"ltx_ref_tag\">1</text></text></text></ref>"
      )
    ),
    "{xml}"
  );
}

/// 63l: an "and" tied to the name before it (`Ohta\ddag~and Kenji`, nlin0101056) or set in a group (`Price {\ and}
/// Ken`, 1508.01140) separates two authors. Repro sectioning-frontmatter/author_and_tied_or_grouped_splits.
#[test]
fn author_and_tied_or_grouped_splits() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_and_tied_or_grouped_splits.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Tetsu Masuda†</personname></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Yasuhiro Ohta‡</personname></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Kenji Kajiwara†</personname></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Huw Price</personname></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Ken Wharton</personname></creator>",
    ],
  );
}

/// 63l: `\fnmsep` between an author's marks leaves no comma in the name once the marks become a link and a note
/// (astro-ph0001054, astro-ph0611016). Repro sectioning-frontmatter/author_fnmsep_leaves_no_comma.
#[test]
fn author_fnmsep_leaves_no_comma() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_fnmsep_leaves_no_comma.tex"
    ),
    &[
      "<creator role=\"author\"><personname>U.\u{a0}Hopp</personname><note class=\"ltx_note_frontmatter ltx_thanks_note\" role=\"thanks\" xml:id=\"id1\">Visiting astronomer</note><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">A</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>D.\u{a0}Engels</personname><note class=\"ltx_note_frontmatter ltx_thanks_note\" role=\"thanks\" xml:id=\"id2\">Fellow<sup xml:id=\"id2.1\"><text font=\"italic\" xml:id=\"id2.1.1\">,</text></sup> of X</note><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">B</contact></creator>",
    ],
  );
}

/// 63l: `\IEEEmembership` is its author's unlabelled membership contact, the comma before it no author separator
/// (2306.15457, 2508.00603), nor the comma at the end of a grade (2408.00327). Repro
/// sectioning-frontmatter/ieee_membership_is_a_contact.
#[test]
fn ieee_membership_is_a_contact() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/ieee_membership_is_a_contact.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Yun-Chih\u{a0}Chen</personname><contact role=\"membership\">Member,\u{a0}IEEE</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Yuan-Hao\u{a0}Chang</personname><contact role=\"membership\">Fellow,\u{a0}IEEE</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Hong Joo Lee</personname></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Yong Man Ro</personname><contact role=\"membership\">Senior Member,\u{a0}IEEE</contact></creator>",
    ],
  );
}

/// 63l review: a grade whose author cannot be known — a biography heading, the body — stays dropped, as in Perl,
/// rather than going to the author before it; names continued past a `\\` (2408.01902) keep theirs (63m). Repro
/// sectioning-frontmatter/ieee_membership_outside_the_names_stays_dropped.
#[test]
fn ieee_membership_outside_the_names_stays_dropped() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/ieee_membership_outside_the_names_stays_dropped.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Wenming\u{a0}Li</personname></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Xiaochun\u{a0}Ye</personname></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Dongrui\u{a0}Fan</personname><contact role=\"membership\">Senior\u{a0}Member,\u{a0}IEEE</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Yuan\u{a0}Xie</personname><contact role=\"membership\">Fellow,\u{a0}IEEE</contact></creator>",
    ],
  );
}

/// 63l review: grades in an `\IEEEauthorblockN` name list followed by the author's own comma (2408.01956, 2408.00368)
/// stay with their names, the list split as before. Repro sectioning-frontmatter/ieee_membership_in_a_block_list.
#[test]
fn ieee_membership_in_a_block_list() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/ieee_membership_in_a_block_list.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Huizhi\u{a0}Wang</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Southeast University</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Yong\u{a0}Zeng</personname><contact role=\"membership\">Senior Member, IEEE</contact><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Southeast University</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Shi\u{a0}Jin</personname><contact role=\"membership\">Fellow, IEEE</contact><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Southeast University</contact></creator>",
    ],
  );
}

/// 63l review: a document's own `\IEEEmembership` (2408.00647 prints its grades in italics) keeps its meaning; the
/// grades are not made contacts behind its back. Repro sectioning-frontmatter/ieee_membership_own_definition_kept.
#[test]
fn ieee_membership_own_definition_kept() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/ieee_membership_own_definition_kept.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Nuno C. Martins<text font=\"italic\" xml:id=\"id1\">Senior Member, IEEE</text></personname></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Jair Certorio<text font=\"italic\" xml:id=\"id2\">Student Member, IEEE</text></personname></creator>",
    ],
  );
}

/// 63l review: `\IEEEoverridecommandlockouts` (IEEEtran.cls:6278-6288, in the IEEE conference template) lets the grade
/// print in conference mode, so it is the author's contact there too. Repro
/// sectioning-frontmatter/ieee_membership_conference_override.
#[test]
fn ieee_membership_conference_override() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/ieee_membership_conference_override.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Ann Able</personname><contact role=\"membership\">Member,\u{a0}IEEE</contact><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ</contact></creator>",
    ],
  );
}

/// 63m: an affiliation line under a group of names is each name's, as LaTeX prints it under them (0911.0568's five
/// LPT Orsay names over one line); an email still goes to the name it spells, ahead of the group's line there (the
/// line is digested once, under the group's last name). Repro
/// sectioning-frontmatter/author_group_affiliation_goes_to_each_name.
#[test]
fn author_group_affiliation_goes_to_each_name() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_group_affiliation_goes_to_each_name.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Dept. of Physics, Univ. A</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname><contact name=\"E-mail: \" role=\"email\"><text font=\"typewriter\" xml:id=\"id1\">bob.baker@a.edu</text></contact><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Dept. of Physics, Univ. A</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Cat Cole</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Dept. of Physics, Univ. A</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Dan Doe</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ. B</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Eve Elm</personname></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Fay Fox</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ. C</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Gus Gray</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ. C</contact></creator>",
    ],
  );
}

/// 63m: IEEE names continued past a `\\` after an unfinished list are names, their grades membership contacts, and a
/// compsoc `\IEEEcompsocitemizethanks` after the last is its note (2408.01902). Repro
/// sectioning-frontmatter/ieee_compsoc_names_continue_past_a_break.
#[test]
fn ieee_compsoc_names_continue_past_a_break() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/ieee_compsoc_names_continue_past_a_break.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Ann\u{a0}Able</personname></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob\u{a0}Baker</personname><contact role=\"membership\">Member,\u{a0}IEEE</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Cat\u{a0}Cole</personname></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Dan\u{a0}Doe</personname></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Eve\u{a0}Elm</personname><contact role=\"membership\">Senior\u{a0}Member,\u{a0}IEEE</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Fay\u{a0}Fox</personname><contact role=\"membership\">Fellow,\u{a0}IEEE</contact><note class=\"ltx_note_frontmatter ltx_thanks_note\" role=\"thanks\" xml:id=\"id1\">A. Able, B. Baker and C. Cole are with Univ A. F. Fox is with Univ B.</note></creator>",
    ],
  );
}

/// 63m: a grade carrying the list's comma before a `\\` leaves the names unfinished, so the next line's names are
/// authors too (2408.02464). Repro sectioning-frontmatter/ieee_grade_comma_before_a_break_continues_names.
#[test]
fn ieee_grade_comma_before_a_break_continues_names() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/ieee_grade_comma_before_a_break_continues_names.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Gus Gray<sup xml:id=\"id1\">1</sup></personname></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Hal Hill</personname></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Ida Ivy</personname><contact role=\"membership\">Fellow,\u{a0}IEEE</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Jon Jay</personname><contact role=\"membership\">Member,\u{a0}IEEE</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Kim Key</personname><contact role=\"membership\">Member,\u{a0}IEEE</contact><note class=\"ltx_note_frontmatter ltx_thanks_note\" role=\"thanks\" xml:id=\"id2\">G. Gray and K. Key are with Univ C.</note></creator>",
    ],
  );
}

/// 63m: a second line of two or more names under the names is more names (2308.07107), and a line of bare addresses
/// is their emails, each to the name it spells, else the last (2401.15897, 2402.02746). Repro
/// sectioning-frontmatter/author_names_line_and_address_line.
#[test]
fn author_names_line_and_address_line() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_names_line_and_address_line.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Email:\u{a0}\" role=\"email\"><text font=\"typewriter\" xml:id=\"id1\">ann.able@a.edu</text></contact><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Dept. of Physics, Univ. A</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Dept. of Physics, Univ. A</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Cat Cole</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Dept. of Physics, Univ. A</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Dan Doe</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Dept. of Physics, Univ. A</contact><contact name=\"Email:\u{a0}\" role=\"email\"><text font=\"typewriter\" xml:id=\"id2\">dan.doe@a.edu</text></contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Eve Elm</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ. B</contact><contact name=\"Email:\u{a0}\" role=\"email\"><text font=\"typewriter\" xml:id=\"id3\">eve@b.edu</text></contact><contact name=\"Email:\u{a0}\" role=\"email\"><text font=\"typewriter\" xml:id=\"id4\">staff@b.edu</text></contact></creator>",
    ],
  );
}

/// 63m: a line of names joined by "&" alone is an affiliation (`Meta FAIR \& Inria Rennes`, 2402.14904); a names line
/// is a comma list. Repro sectioning-frontmatter/author_ampersand_affiliation_line_is_not_names.
#[test]
fn author_ampersand_affiliation_line_is_not_names() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_ampersand_affiliation_line_is_not_names.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Meta FAIR &amp; Inria Rennes</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Meta FAIR</contact></creator>",
    ],
  );
}

/// 63m: a line of more addresses than its group has names gives each to the name it spells in any group of the
/// author block (`\author{Rose Bohrer \and Ashe Neth\\ … \\ \texttt{\{rbohrer,aneth\}@wpi.edu}}`, 2409.18978), by the
/// names as printed (1706.03762's `\thanks` names no one); a line of one address under one name is that name's
/// (2409.00286). Repro
/// sectioning-frontmatter/author_email_owner_in_another_group.
#[test]
fn author_email_owner_in_another_group() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_email_owner_in_another_group.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Ann Able</personname><note class=\"ltx_note_frontmatter ltx_thanks_contribution\" role=\"thanks\" xml:id=\"id1\">Equal contribution with Bob.</note><contact name=\"Email:\u{a0}\" role=\"email\"><text font=\"typewriter\" xml:id=\"id2\">aable@x.edu</text></contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Dept. of CS</contact><contact name=\"Email:\u{a0}\" role=\"email\"><text font=\"typewriter\" xml:id=\"id3\">bob@x.edu</text></contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Zed Chen</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ. Z</contact><contact name=\"Email:\u{a0}\" role=\"email\"><text font=\"typewriter\" xml:id=\"id4\">zc@z.edu</text></contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Chengxi Li</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ. Y</contact><contact name=\"Email:\u{a0}\" role=\"email\"><text font=\"typewriter\" xml:id=\"id5\">chengxil@y.edu</text></contact></creator>",
    ],
  );
}

/// 63m: a line of several brace groups of addresses (`{a, b}@x, {c, d}@y`) is the names' emails, each group's local
/// parts taking its domain (2402.02746, 2410.19160). Repro sectioning-frontmatter/author_brace_email_groups_on_one_line.
#[test]
fn author_brace_email_groups_on_one_line() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_brace_email_groups_on_one_line.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Email:\u{a0}\" role=\"email\"><text font=\"typewriter\" xml:id=\"id1\">able@a.edu</text></contact><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Dept. of CS</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname><contact name=\"Email:\u{a0}\" role=\"email\"><text font=\"typewriter\" xml:id=\"id2\">baker@a.edu</text></contact><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Dept. of CS</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Cat Cole</personname><contact name=\"Email:\u{a0}\" role=\"email\"><text font=\"typewriter\" xml:id=\"id3\">cole@b.edu</text></contact><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Dept. of CS</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Dan Doe</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Dept. of CS</contact><contact name=\"Email:\u{a0}\" role=\"email\"><text font=\"typewriter\" xml:id=\"id4\">dan.doe@b.edu</text></contact></creator>",
    ],
  );
}

/// 63m: a marked block's address line in another order than the authors is the block's, not the authors' in order
/// (2509.10377 lists one address for each author, by institution; 2406.06326). Repro
/// sectioning-frontmatter/author_email_list_out_of_author_order.
#[test]
fn author_email_list_out_of_author_order() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_email_list_out_of_author_order.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ. A</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ. B</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Cat Cole</personname><contact name=\"Email:\u{a0}\" role=\"email\"><text font=\"typewriter\" xml:id=\"id1\">{able, cole}@a.edu, baker@b.edu</text></contact><contact name=\"Email:\u{a0}\" role=\"email\"><text font=\"typewriter\" xml:id=\"id2\">zed@z.edu</text></contact><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ. A</contact></creator>",
    ],
  );
}

/// 63m: an address line under the first group may name an author of a later one; every group's names are read first
/// (the shape of 2409.18978 reversed; a panic before). Repro sectioning-frontmatter/author_email_owner_in_a_later_group.
#[test]
fn author_email_owner_in_a_later_group() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_email_owner_in_a_later_group.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ. A</contact><contact name=\"Email:\u{a0}\" role=\"email\"><text font=\"typewriter\" xml:id=\"id1\">able@x.edu</text></contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname><contact name=\"Email:\u{a0}\" role=\"email\"><text font=\"typewriter\" xml:id=\"id2\">baker@x.edu</text></contact><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ. B</contact></creator>",
    ],
  );
}

/// 63m: an address split out of a line of several keeps its underscore (`wu\_zhiliang`, 2509.10377). Repro
/// sectioning-frontmatter/author_email_underscore_kept.
#[test]
fn author_email_underscore_kept() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_email_underscore_kept.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Email:\u{a0}\" role=\"email\"><text font=\"typewriter\" xml:id=\"id1\">ann_able@x.edu</text></contact><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ. A</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ. A</contact><contact name=\"Email:\u{a0}\" role=\"email\"><text font=\"typewriter\" xml:id=\"id2\">bob@y.edu</text></contact></creator>",
    ],
  );
}

/// 63m: a bare word before an address is no local part (`Berlin, Germany, foo@bar.de` invents no address); the line
/// is the names' affiliation. Repro sectioning-frontmatter/author_place_before_address_is_affiliation.
#[test]
fn author_place_before_address_is_affiliation() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_place_before_address_is_affiliation.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Berlin, Germany, foo@bar.de</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Berlin, Germany, foo@bar.de</contact></creator>",
    ],
  );
}

/// 63m: a comma line of name-shaped parts under one name is its affiliation (`Jane Doe\\ Carnegie Mellon, Pittsburgh
/// PA`); only a list of names continues (2308.07107). Repro
/// sectioning-frontmatter/author_affiliation_comma_line_under_one_name.
#[test]
fn author_affiliation_comma_line_under_one_name() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_affiliation_comma_line_under_one_name.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Jane Doe</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Carnegie Mellon, Pittsburgh PA</contact></creator>",
    ],
  );
}

/// 63m: a line of addresses parted by a spacing macro alone (no space token between them, 2410.07147's `\nsone`) is
/// still an email line, one contact, not welded onto the affiliation. Repro
/// sectioning-frontmatter/author_addresses_parted_by_a_macro.
#[test]
fn author_addresses_parted_by_a_macro() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_addresses_parted_by_a_macro.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ. A</contact><contact name=\"Email:\u{a0}\" role=\"email\">able@a.edu\u{2003}baker@a.edu</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ. A</contact></creator>",
    ],
  );
}

/// The start tag of the first `<name …>` element of `xml`, and the whole first `<note …>…</note>` of class `class`.
fn hf4_elements(xml: &str, name: &str) -> (String, String) {
  let open = xml
    .find(&format!("<{name} "))
    .map(|at| xml[at..at + xml[at..].find('>').unwrap() + 1].to_string())
    .unwrap_or_default();
  let note = xml
    .find("<note class=\"ltx_nodisplay ltx_acm_description\"")
    .or_else(|| xml.find("<note class=\"ltx_acm_description"))
    .map(|at| xml[at..at + xml[at..].find("</note>").unwrap() + "</note>".len()].to_string())
    .unwrap_or_default();
  (open, note)
}

/// 63n: a `\label` after acmart's `\Description` names the float, not the hidden description note (2403.09168: ten
/// labels on notes; Perl the same, KNOWN_PERL_ERRORS #535). Repro captions-floats/acmart_description_label_after_names_the_float.
#[test]
fn acmart_description_label_after_names_the_float() {
  let (log, xml) = latexml::util::test::convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/captions-floats/acmart_description_label_after_names_the_float.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!(super::perfect_kernel_batch46::error_count(&log), 0, "{log}");
  assert_eq!(
    super::perfect_kernel_batch46::warning_count(&log),
    0,
    "{log}"
  );
  let (table, note) = hf4_elements(&xml, "table");
  assert_eq!(
    (table.as_str(), note.as_str()),
    (
      "<table aria:describedby=\"acmlabel1\" inlist=\"lot\" labels=\"LABEL:tab:x\" xml:id=\"S0.T1\">",
      "<note class=\"ltx_nodisplay ltx_acm_description\" xml:id=\"acmlabel1\">Hidden description.</note>"
    ),
    "{xml}"
  );
}

/// 63n: acmart's `\Description` digests its description, so its markup is markup in the note, not raw TeX
/// (`\sysname{}`, 2403.09168; 2401.04997, 2304.01062, 2312.11013; Perl acmart.cls.ltxml:78 digests `{}`). Repro
/// captions-floats/acmart_description_markup_is_digested.
#[test]
fn acmart_description_markup_is_digested() {
  let (log, xml) = latexml::util::test::convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/captions-floats/acmart_description_markup_is_digested.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!(super::perfect_kernel_batch46::error_count(&log), 0, "{log}");
  assert_eq!(
    super::perfect_kernel_batch46::warning_count(&log),
    0,
    "{log}"
  );
  let (table, note) = hf4_elements(&xml, "table");
  assert_eq!(
    (table.as_str(), note.as_str()),
    (
      "<table aria:describedby=\"acmlabel1\" inlist=\"lot\" xml:id=\"S0.T1\">",
      "<note class=\"ltx_nodisplay ltx_acm_description\" xml:id=\"acmlabel1\">The table of <emph font=\"italic\" xml:id=\"acmlabel1.1\">all</emph> SysName results.</note>"
    ),
    "{xml}"
  );
}

/// 63n: a display listing is as high as TeX sets it — aboveskip, a `\baselineskip` per line, belowskip — so a
/// tcolorbox around it is too (html_feedback HF1: 230 of 290 boxed listings a line high; 2406.06469, 2402.10176;
/// OXIDIZED_DESIGN_DIVERGENCES #462). Repro boxes-groups/tcb_listing_body_height.
#[test]
fn tcb_listing_body_height() {
  let (log, xml) = latexml::util::test::convert_with(
    include_str!("../../../tools/perfect_kernel/repros/boxes-groups/tcb_listing_body_height.tex"),
    Some("ar5iv.sty"),
  );
  assert_eq!(super::perfect_kernel_batch46::error_count(&log), 0, "{log}");
  assert_eq!(
    super::perfect_kernel_batch46::warning_count(&log),
    0,
    "{log}"
  );
  let at = xml.find("<svg:foreignObject").expect("a foreignObject");
  let object = &xml[at..at + xml[at..].find("</svg:foreignObject>").unwrap()];
  assert_eq!(
    &object[..object.find('>').unwrap() + 1],
    "<svg:foreignObject height=\"116.23\" overflow=\"visible\" style=\"--ltx-fo-width:31.37em;--ltx-fo-height:8.4em;--ltx-fo-depth:0em;font-size:10pt;\" transform=\"matrix(1 0 0 -1 0 116.23)\" width=\"434.07\">",
    "{xml}"
  );
  assert_eq!(object.matches("<listingline ").count(), 6, "{object}");
}

/// 63n: acmart's `\Description` typesets nothing, so `_ ^ & #` written plain in it are the characters they show, not
/// errors (2502.07049 `\Description{Complaint_Loss}`, 2404.00573, 2503.04114). Repro
/// captions-floats/acmart_description_unescaped_characters.
#[test]
fn acmart_description_unescaped_characters() {
  let (log, xml) = latexml::util::test::convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/captions-floats/acmart_description_unescaped_characters.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!(super::perfect_kernel_batch46::error_count(&log), 0, "{log}");
  assert_eq!(
    super::perfect_kernel_batch46::warning_count(&log),
    0,
    "{log}"
  );
  let (_, note) = hf4_elements(&xml, "table");
  assert_eq!(
    note,
    "<note class=\"ltx_nodisplay ltx_acm_description\" xml:id=\"acmlabel1\">Complaint_Loss, 5^2, A &amp; B, #1 and <Math mode=\"inline\" tex=\"x_{i}\" text=\"x _ i\" xml:id=\"acmlabel1.m1\">\n          <XMath xml:id=\"acmlabel1.m1.1\">\n            <XMApp xml:id=\"acmlabel1.m1.1.1\">\n              <XMTok role=\"SUBSCRIPTOP\" scriptpos=\"post1\"/>\n              <XMTok font=\"italic\" role=\"UNKNOWN\">x</XMTok>\n              <XMTok font=\"italic\" fontsize=\"70%\" role=\"UNKNOWN\">i</XMTok>\n            </XMApp>\n          </XMath>\n        </Math></note>",
    "{xml}"
  );
}

/// 63n: a comment spanning lines counts each of its lines in the listing's height (its class group stays open across
/// them; OXIDIZED_DESIGN_DIVERGENCES #462). Repro boxes-groups/tcb_listing_comment_lines.
#[test]
fn tcb_listing_comment_lines() {
  let (log, xml) = latexml::util::test::convert_with(
    include_str!("../../../tools/perfect_kernel/repros/boxes-groups/tcb_listing_comment_lines.tex"),
    Some("ar5iv.sty"),
  );
  assert_eq!(super::perfect_kernel_batch46::error_count(&log), 0, "{log}");
  assert_eq!(
    super::perfect_kernel_batch46::warning_count(&log),
    0,
    "{log}"
  );
  let at = xml.find("<svg:foreignObject").expect("a foreignObject");
  let object = &xml[at..at + xml[at..].find("</svg:foreignObject>").unwrap()];
  assert_eq!(
    &object[..object.find('>').unwrap() + 1],
    "<svg:foreignObject height=\"83.02\" overflow=\"visible\" style=\"--ltx-fo-width:31.37em;--ltx-fo-height:6em;--ltx-fo-depth:0em;font-size:10pt;\" transform=\"matrix(1 0 0 -1 0 83.02)\" width=\"434.07\">",
    "{xml}"
  );
  assert_eq!(object.matches("<listingline ").count(), 4, "{object}");
}

/// The first whole `<name …>…</name>` (or `<name …/>`) element of `xml` whose text contains `needle`.
fn element_with(xml: &str, name: &str, needle: &str) -> String {
  let mut at = 0;
  while let Some(start) = xml[at..].find(&format!("<{name} ")).map(|i| at + i) {
    let open_end = start + xml[start..].find('>').unwrap();
    let end = if xml.as_bytes()[open_end - 1] == b'/' {
      open_end + 1
    } else {
      xml[start..]
        .find(&format!("</{name}>"))
        .map_or(open_end + 1, |i| start + i + name.len() + 3)
    };
    if xml[start..end].contains(needle) {
      return xml[start..end].to_string();
    }
    at = open_end;
  }
  String::new()
}

/// Converts the repro `tex`, asserts 0 errors and 0 warnings, and returns the element `element_with` finds.
fn hf3_element(tex: &str, name: &str, needle: &str) -> String {
  let (log, xml) = latexml::util::test::convert_with(tex, Some("ar5iv.sty"));
  assert_eq!(super::perfect_kernel_batch46::error_count(&log), 0, "{log}");
  assert_eq!(
    super::perfect_kernel_batch46::warning_count(&log),
    0,
    "{log}"
  );
  element_with(&xml, name, needle)
}

/// 63o: svmult's run-in headings (svmult.cls:701-710; 1805.00023). Repro sectioning-frontmatter/svmult_runinhead.
#[test]
fn svmult_runinhead() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/svmult_runinhead.tex"
  );
  assert_eq!(
    hf3_element(tex, "paragraph", "Transiting"),
    "<paragraph inlist=\"toc\" xml:id=\"Ch0.S0.SS0.SSS0.Px1\">\n    <title>Transiting Giant Planets</title>\n    <para xml:id=\"Ch0.S0.SS0.SSS0.Px1.p1\">\n      <p xml:id=\"Ch0.S0.SS0.SSS0.Px1.p1.1\">Some text.</p>\n    </para>\n  </paragraph>"
  );
  assert_eq!(
    hf3_element(tex, "paragraph", "Dwarfs"),
    "<paragraph inlist=\"toc\" xml:id=\"Ch0.S0.SS0.SSS0.Px2\">\n    <title>Dwarfs</title>\n    <para xml:id=\"Ch0.S0.SS0.SSS0.Px2.p1\">\n      <p xml:id=\"Ch0.S0.SS0.SSS0.Px2.p1.1\">More text.</p>\n    </para>\n  </paragraph>"
  );
}

/// 63o: interact's `\tbl{caption}{body}` (interact.cls:493-499; 2312.11500, 2501.02233). Repro
/// captions-floats/interact_tbl.
#[test]
fn interact_tbl() {
  let tex = include_str!("../../../tools/perfect_kernel/repros/captions-floats/interact_tbl.tex");
  assert_eq!(
    hf3_element(tex, "table", "IMO"),
    "<table inlist=\"lot\" xml:id=\"S0.T1\">\n    <tags>\n      <tag>Table 1</tag>\n      <tag role=\"autoref\">Table\u{a0}1<text xml:id=\"S0.T1.1\"/></tag>\n      <tag role=\"refnum\">1</tag>\n      <tag role=\"typerefnum\">Table 1</tag>\n    </tags>\n    <toccaption><tag close=\" \">1</tag>IMO definitions.</toccaption>\n    <caption><tag close=\": \">Table 1</tag>IMO definitions.</caption>\n    <tabular vattach=\"middle\" xml:id=\"S0.T1.2\">\n      <tbody>\n        <tr xml:id=\"S0.T1.2.1\">\n          <td align=\"center\" xml:id=\"S0.T1.2.1.1\">a</td>\n          <td align=\"center\" xml:id=\"S0.T1.2.1.2\">b</td>\n        </tr>\n      </tbody>\n    </tabular>\n  </table>"
  );
}

/// 63o: IEEEtaes's `\member` is the author's membership; empty `\editor`/`\supplementary` are nothing
/// (IEEEtaes.cls:4882, :3455-3456; 2403.15966). Repro sectioning-frontmatter/ieeetaes_member.
#[test]
fn ieeetaes_member() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/sectioning-frontmatter/ieeetaes_member.tex");
  let (log, xml) = latexml::util::test::convert_with(tex, Some("ar5iv.sty"));
  assert_eq!(super::perfect_kernel_batch46::error_count(&log), 0, "{log}");
  assert_eq!(
    super::perfect_kernel_batch46::warning_count(&log),
    0,
    "{log}"
  );
  assert_eq!(
    super::perfect_kernel_batch61::creators_of(&xml),
    [
      "<creator role=\"author\"><personname>SHASHWAT JAIN</personname><contact role=\"membership\">Student Member, IEEE</contact></creator>"
    ],
    "{xml}"
  );
  assert_eq!(
    element_with(&xml, "note", "Recommended"),
    "<note role=\"editor\" xml:id=\"id1\">Recommended by X.</note>",
    "{xml}"
  );
  assert!(!xml.contains("role=\"supplementary\""), "{xml}");
}

/// 63o: jcappub's journal abbreviations (jcappub.sty:72-146; 2404.02153). Repro index-bib/jcappub_journal_macros.
#[test]
fn jcappub_journal_macros() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/index-bib/jcappub_journal_macros.tex");
  assert_eq!(
    hf3_element(tex, "p", "See"),
    "<p xml:id=\"p1.1\">See JCAP, ApJ, PhRvL, MNRAS.</p>"
  );
}

/// 63o: svg's `\svgsetup`/`\svgpath` (svg.sty:810, :814-823), and the `\setsvg` options reach `\includegraphics`
/// expanded (2401.10458, 2402.15627). Repro graphics-tikz/svg_setup_options.
#[test]
fn svg_setup_options() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/graphics-tikz/svg_setup_options.tex");
  assert_eq!(
    hf3_element(tex, "graphics", "fig"),
    "<graphics graphic=\"fig\" options=\"inkscapelatex=false,inkscapelatex=false,width=85.35826pt,keepaspectratio=true\" xml:id=\"p1.g1\"/>"
  );
}

/// 63o: mdpi loads soul (mdpi.cls:48; `\hl`, 2312.16815) and has its reference shorthands (mdpi.cls:381-385).
/// Repro loader/mdpi_requires_soul.
#[test]
fn mdpi_requires_soul() {
  let tex = include_str!("../../../tools/perfect_kernel/repros/loader/mdpi_requires_soul.tex");
  assert_eq!(
    hf3_element(tex, "p", "important"),
    "<p xml:id=\"p1.1\">This is <text backgroundcolor=\"#FFFF00\" xml:id=\"p1.1.1\">important</text> text, see Figure\u{a0}<ref labelref=\"LABEL:f1\"/>.</p>"
  );
}

/// 63o: bmvc2k loads xspace (bmvc2k.cls:118; 2606.17384). Repro loader/bmvc2k_requires_xspace.
#[test]
fn bmvc2k_requires_xspace() {
  let tex = include_str!("../../../tools/perfect_kernel/repros/loader/bmvc2k_requires_xspace.tex");
  assert_eq!(
    hf3_element(tex, "p", "here"),
    "<p xml:id=\"p1.1\">BMVC is here.</p>"
  );
}

/// The start tag of the `<picture>` in the `<figure>` of `xml`, and how many `<graphics>` that figure holds.
fn figure_picture(xml: &str) -> (String, usize) {
  let figure = element_with(xml, "figure", "<");
  let picture = figure
    .find("<picture")
    .map(|at| figure[at..at + figure[at..].find('>').unwrap() + 1].to_string())
    .unwrap_or_default();
  (picture, figure.matches("<graphics").count())
}

/// 63p: `\includestandalone` of a sub-file that ships only as `.tex` inputs it, as standalone.sty's default `tex` mode
/// does (standalone.sty:211, :1014-1093), scaled to the width asked for (html_feedback HF12: 2412.12317, 2505.19304,
/// 2608.05283). Repro graphics-tikz/includestandalone_tex_figure.
#[test]
fn includestandalone_tex_figure() {
  let (log, xml) = latexml::util::test::convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/graphics-tikz/includestandalone_tex_figure.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!(super::perfect_kernel_batch46::error_count(&log), 0, "{log}");
  assert_eq!(
    super::perfect_kernel_batch46::warning_count(&log),
    0,
    "{log}"
  );
  assert_eq!(
    figure_picture(&xml),
    (
      "<picture height=\"39.92\" width=\"39.92\" xml:id=\"S0.F1.pic1\">".to_string(),
      0
    ),
    "{xml}"
  );
  // scaled to half the line width as gincltex's `\resizebox` does
  let figure = element_with(&xml, "figure", "<");
  let at = figure.find("<inline-block").expect("the resized box");
  assert_eq!(
    &figure[at..at + figure[at..].find('>').unwrap() + 1],
    "<inline-block align=\"center\" depth=\"0.0pt\" height=\"128.1pt\" width=\"172.5pt\" xscale=\"4.43984160254303\" xtranslate=\"66.8pt\" yscale=\"4.43984160254303\" ytranslate=\"-49.6pt\" xml:id=\"S0.F1.1\">",
    "{xml}"
  );
}

/// 63p: the sub-file is found as a graphic is, through `\graphicspath` (2403.01643 `\graphicspath{{figs/}}` and
/// `\includestandalone{archs/eff_att}`).
#[test]
fn includestandalone_through_graphicspath() {
  let (log, xml) = latexml::util::test::convert_files_with(
    "\\documentclass{article}\n\\usepackage{tikz}\n\\usepackage{standalone}\n\\graphicspath{{sfigs/}}\n\
     \\begin{document}\n\\begin{figure}\\includestandalone{archs/fig}\\caption{C.}\\end{figure}\n\\end{document}\n",
    &[(
      "sfigs/archs/fig.tex",
      "\\documentclass{standalone}\n\\usepackage{tikz}\n\\begin{document}\n\
       \\begin{tikzpicture}\\draw (0,0)--(1,1);\\end{tikzpicture}\n\\end{document}\n",
    )],
    Some("ar5iv.sty"),
  );
  assert_eq!(super::perfect_kernel_batch46::error_count(&log), 0, "{log}");
  assert_eq!(
    super::perfect_kernel_batch46::warning_count(&log),
    0,
    "{log}"
  );
  assert_eq!(
    figure_picture(&xml),
    (
      "<picture height=\"39.92\" width=\"39.92\" xml:id=\"S0.F1.pic1\">".to_string(),
      0
    ),
    "{xml}"
  );
}

/// 63p: a sub-file's preamble is skipped as standalone.sty's `\sa@documentclass` skips it (standalone.sty:602-646): its
/// `\title`/`\author` print nothing in the figure (2504.17583), its packages and inputs are not loaded (2403.17633,
/// 2508.06316). Repro graphics-tikz/standalone_subfile_preamble_skipped.
#[test]
fn standalone_subfile_preamble_skipped() {
  let (log, xml) = latexml::util::test::convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/graphics-tikz/standalone_subfile_preamble_skipped.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!(super::perfect_kernel_batch46::error_count(&log), 0, "{log}");
  assert_eq!(
    super::perfect_kernel_batch46::warning_count(&log),
    0,
    "{log}"
  );
  let (picture, graphics) = figure_picture(&xml);
  assert_eq!(
    (picture.as_str(), graphics),
    (
      "<picture class=\"ltx_centering\" height=\"39.92\" width=\"39.92\" xml:id=\"S0.F1.pic1\">",
      0
    ),
    "{xml}"
  );
  assert!(!element_with(&xml, "figure", "<").contains("Adar"), "{xml}");
}

/// 63q: author markup a class defines only inside `\@maketitle`'s group (melba.cls:279-283 `\def\aff`, `\def\name`;
/// dmlr2e.sty:249-252 `\def\addr`, `\def\email`) is defined while the author content is digested, before the title
/// code runs (2405.09787, 2404.08403). Repro sectioning-frontmatter/author_markup_defined_in_title_code.
#[test]
fn author_markup_defined_in_title_code() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_markup_defined_in_title_code.tex"
  );
  assert_eq!(
    hf3_element(tex, "creator", "Ann"),
    "<creator role=\"author\">\n    <personname>Ann Author<sup xml:id=\"id1\"><text font=\"bold\" xml:id=\"id1.1\">1</text></sup></personname>\n  </creator>"
  );
}

/// 63q: the forms a title code defines its author markup with — `\let`, a spaced `\newcommand [1] {…}`, `\def\x#1`, two
/// `\def`s of one name (the last wins) — in force for the author and its affiliation lines (2406.07496's `\email`
/// line), none left defined after it; a definition already in force stands; one in a group closed before `\@author`
/// is not replayed. Repro sectioning-frontmatter/author_markup_title_code_forms.
#[test]
fn author_markup_title_code_forms() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_markup_title_code_forms.tex"
  );
  let (log, xml) = latexml::util::test::convert_with(tex, Some("ar5iv.sty"));
  assert_eq!(super::perfect_kernel_batch46::error_count(&log), 0, "{log}");
  assert_eq!(
    super::perfect_kernel_batch46::warning_count(&log),
    0,
    "{log}"
  );
  assert_eq!(
    element_with(&xml, "creator", "Ann"),
    "<creator role=\"author\">\n    <personname>Ann Author [x] b Given</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><text font=\"bold italic\" xml:id=\"id1\">Some University</text></contact>\n    <contact name=\"Email:\u{a0}\" role=\"email\"><text font=\"typewriter bold\" xml:id=\"id2\">ann@uni.edu</text></contact>\n  </creator>",
    "{xml}"
  );
  assert_eq!(
    element_with(&xml, "para", "clean"),
    "<para xml:id=\"p1\">\n    <p xml:id=\"p1.1\">clean clean</p>\n  </para>",
    "{xml}"
  );
}

/// 63r: imsart's `\ead[label=e1]{x}` is the author's email (a URL with `url`), `\printead` in an address leaves it out
/// (each address already the author's contact), `\author[A]`/`\address[A]` link by their marks and `\thanksref{l}` by
/// its labels, `\thankstext{l}{…}` is the note it names, `\arxiv{id}` the paper's pubnote (imsart.sty:944, :965, :2003,
/// :2088, :746, :789, :1186; 2201.11773, 2507.12447, 2201.09706, 2201.08502). Repro
/// sectioning-frontmatter/imsart_ead_and_address_marks.
#[test]
fn imsart_ead_and_address_marks() {
  let (log, xml) = latexml::util::test::convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/imsart_ead_and_address_marks.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!(super::perfect_kernel_batch46::error_count(&log), 0, "{log}");
  assert_eq!(
    super::perfect_kernel_batch46::warning_count(&log),
    0,
    "{log}"
  );
  // the title's `\thanksref{T0}` note, and a document-level `\renewcommand\arxiv` in the bibliography
  for whole in [
    "<title>A Title<note class=\"ltx_note_frontmatter ltx_thanks_funding\" role=\"thanks\" xml:id=\"id1\">Supported by a fund.</note></title>",
    "<bibblock> A. Author. A paper. arXiv:1234.5678.\n</bibblock>",
  ] {
    assert!(xml.contains(whole), "{whole}\n{xml}");
  }
  assert_eq!(
    element_with(&xml, "creator", "Ann"),
    "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <contact name=\"Email:\u{a0}\" role=\"email\">ann_a@uni.edu</contact>\n    <contact name=\"Address:\u{a0}\" role=\"address\">Department of Mathematics, McGill University</contact>\n    <contact name=\"Thanks:\u{a0}\" role=\"thanks\">Supported by a grant.</contact>\n  </creator>"
  );
  assert_eq!(
    element_with(&xml, "creator", "Bob"),
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Bob Baker</personname>\n    <contact name=\"Email:\u{a0}\" role=\"email\">bob@rug.nl</contact>\n    <contact name=\"URL:\u{a0}\" role=\"url\">https://bob.example.org</contact>\n    <contact name=\"Address:\u{a0}\" role=\"address\">Bernoulli Institute, University of Groningen</contact>\n  </creator>"
  );
  // a name list in one `\author`, each name an author with the address its `\thanksref` names
  assert_eq!(
    element_with(&xml, "creator", "Cy"),
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Cy Cole</personname>\n    <contact name=\"Address:\u{a0}\" role=\"address\">Cole Institute</contact>\n  </creator>"
  );
  assert_eq!(
    element_with(&xml, "creator", "Di Dunn"),
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Di Dunn</personname>\n    <contact name=\"Address:\u{a0}\" role=\"address\">Cole Institute</contact>\n  </creator>"
  );
  // an `\ead` after a space is the author's email, once
  assert_eq!(
    element_with(&xml, "creator", "Eve"),
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Eve Earl</personname>\n    <contact name=\"Email:\u{a0}\" role=\"email\">eve@uni.edu</contact>\n  </creator>"
  );
  assert_eq!(
    element_with(&xml, "pubnote", "0000"),
    "<pubnote name=\"arXiv:\u{a0}\" role=\"arxiv\">0000.00000</pubnote>"
  );
}

/// 63r: a name line's trailing address is the author's email (`\name Ann Able \email ann@uni.edu`, jmlr2e.sty:271-273;
/// 2405.13980, 2409.06765), a note's address glued to a name stays in its note (2402.08164), and a line printing
/// nothing (`\vspace{-0.6cm}`) is no affiliation. Repro sectioning-frontmatter/name_line_trailing_email.
#[test]
fn name_line_trailing_email() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/name_line_trailing_email.tex"
  );
  assert_eq!(
    hf3_element(tex, "creator", "Ann"),
    "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <contact name=\"Note:\u{a0}\" role=\"note\">Equal contribution.</contact>\n    <contact name=\"Email:\u{a0}\" role=\"email\">ann@uni.edu</contact>\n  </creator>"
  );
  assert_eq!(
    hf3_element(tex, "creator", "Bob"),
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Bob Baker</personname>\n    <contact name=\"Email:\u{a0}\" role=\"email\">bob_b@uni.edu</contact>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Other Place</contact>\n  </creator>"
  );
  assert_eq!(
    hf3_element(tex, "creator", "Cy"),
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Cy Cole</personname>\n    <note class=\"ltx_note_frontmatter ltx_thanks_note\" role=\"thanks\" xml:id=\"id1\">cy@uni.edu</note>\n  </creator>"
  );
}

/// 63s: an empty spanner over a column the pruning removes shrinks by one rather than being dropped with it, so the
/// header row stays as wide as the data rows (2406.06521 Table 2; Perl Alignment.pm L822-855 removes the spanner). Repro
/// alignment/empty_spanner_shrinks_with_pruned_column.
#[test]
fn empty_spanner_shrinks_with_pruned_column() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/alignment/empty_spanner_shrinks_with_pruned_column.tex"
  );
  assert_eq!(
    hf3_element(tex, "tabular", "24"),
    "<tabular class=\"ltx_guessed_headers\" vattach=\"middle\" xml:id=\"tab1.1\">\n      <tbody>\n        <tr xml:id=\"tab1.1.1\">\n          <td colspan=\"2\" xml:id=\"tab1.1.1.1\"/>\n          <td align=\"center\" thead=\"column\" xml:id=\"tab1.1.1.2\">24</td>\n        </tr>\n        <tr xml:id=\"tab1.1.2\">\n          <td align=\"center\" xml:id=\"tab1.1.2.1\">VolSDF</td>\n          <td xml:id=\"tab1.1.2.2\"/>\n          <td align=\"center\" xml:id=\"tab1.1.2.3\">1.14</td>\n        </tr>\n        <tr xml:id=\"tab1.1.3\">\n          <td align=\"center\" xml:id=\"tab1.1.3.1\">NeuS</td>\n          <td xml:id=\"tab1.1.3.2\"/>\n          <td align=\"center\" xml:id=\"tab1.1.3.3\">1.00</td>\n        </tr>\n      </tbody>\n    </tabular>"
  );
}

/// 63s: an empty `\multirow` an outer `\multirow` covers spans no rows of its own, so the empty rows pruned below
/// shrink the outer span (2507.20312 Table 2; Perl Alignment.pm L697-726 lets it re-mark the rows). Repro
/// alignment/nested_empty_multirow_keeps_outer_span.
#[test]
fn nested_empty_multirow_keeps_outer_span() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/alignment/nested_empty_multirow_keeps_outer_span.tex"
  );
  assert_eq!(
    hf3_element(tex, "tabular", "Nodes"),
    "<tabular class=\"ltx_guessed_headers\" vattach=\"middle\" xml:id=\"p1.1\">\n      <tbody>\n        <tr xml:id=\"p1.1.1\">\n          <td align=\"left\" border=\"r t\" rowspan=\"3\" thead=\"column row\" xml:id=\"p1.1.1.1\"><text xml:id=\"p1.1.1.1.1\">Nodes</text></td>\n          <td align=\"left\" border=\"r t\" thead=\"column\" xml:id=\"p1.1.1.2\">xeon</td>\n          <td align=\"left\" border=\"t\" thead=\"column\" xml:id=\"p1.1.1.3\">A</td>\n        </tr>\n        <tr xml:id=\"p1.1.2\">\n          <td align=\"left\" border=\"r t\" xml:id=\"p1.1.2.1\">gpu</td>\n          <td align=\"left\" border=\"t\" xml:id=\"p1.1.2.2\">B</td>\n        </tr>\n        <tr xml:id=\"p1.1.3\">\n          <td align=\"left\" border=\"r t\" xml:id=\"p1.1.3.1\">amd</td>\n          <td align=\"left\" border=\"t\" xml:id=\"p1.1.3.2\">C</td>\n        </tr>\n        <tr xml:id=\"p1.1.4\">\n          <td align=\"left\" border=\"b r t\" thead=\"row\" xml:id=\"p1.1.4.1\">Metrics</td>\n          <td align=\"left\" border=\"b r t\" xml:id=\"p1.1.4.2\">time</td>\n          <td align=\"left\" border=\"b t\" xml:id=\"p1.1.4.3\">D</td>\n        </tr>\n      </tbody>\n    </tabular>"
  );
  // the witness's form, each multirow inside a `\multicolumn`
  assert_eq!(
    hf3_element(tex, "tabular", "Hosts"),
    "<tabular class=\"ltx_guessed_headers\" vattach=\"middle\" xml:id=\"p2.1\">\n      <tbody>\n        <tr xml:id=\"p2.1.1\">\n          <td align=\"left\" border=\"r t\" rowspan=\"4\" thead=\"row\" xml:id=\"p2.1.1.1\"><text xml:id=\"p2.1.1.1.1\">Hosts</text></td>\n          <td align=\"left\" border=\"r t\" xml:id=\"p2.1.1.2\">xeon</td>\n          <td align=\"left\" border=\"t\" xml:id=\"p2.1.1.3\">A</td>\n        </tr>\n        <tr xml:id=\"p2.1.2\">\n          <td align=\"left\" border=\"r t\" xml:id=\"p2.1.2.1\">gpu</td>\n          <td align=\"left\" border=\"t\" xml:id=\"p2.1.2.2\">B</td>\n        </tr>\n        <tr xml:id=\"p2.1.3\">\n          <td align=\"left\" border=\"r t\" xml:id=\"p2.1.3.1\">amd</td>\n          <td align=\"left\" border=\"t\" xml:id=\"p2.1.3.2\">C</td>\n        </tr>\n        <tr xml:id=\"p2.1.4\">\n          <td border=\"t\" xml:id=\"p2.1.4.1\"/>\n          <td border=\"t\" xml:id=\"p2.1.4.2\"/>\n        </tr>\n        <tr xml:id=\"p2.1.5\">\n          <td align=\"left\" border=\"b r t\" thead=\"row\" xml:id=\"p2.1.5.1\">Metrics</td>\n          <td align=\"left\" border=\"b r t\" xml:id=\"p2.1.5.2\">time</td>\n          <td align=\"left\" border=\"b t\" xml:id=\"p2.1.5.3\">D</td>\n        </tr>\n      </tbody>\n    </tabular>"
  );
}

/// 63s: `\lxRequireResource[content=…]` builds the resource with its content (Package.pm:3139-3167), and under ar5iv a
/// style sheet given inline is kept, the paper's and a class option's (2606.12996). Repro
/// loader/lx_require_resource_content.
#[test]
fn lx_require_resource_content() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/loader/lx_require_resource_content.tex");
  // openbib's own rules (article.cls.ltxml:35), then the paper's
  assert_eq!(
    hf3_element(tex, "resource", "bibblock"),
    "<resource type=\"text/css\">.ltx_bibblock{display:block;}</resource>"
  );
  assert_eq!(
    hf3_element(tex, "resource", "width"),
    "<resource type=\"text/css\">.ltx_tabular .ltx_tabular { width: auto; }</resource>"
  );
}

/// 63t: jmlr.cls's object references print their name and number through jmlrutils.sty's `\objectref`
/// (jmlrutils.sty:116-165; 2507.03899, 2409.07012) rather than the label key. Repro loader/jmlr_object_refs.
#[test]
fn jmlr_object_refs() {
  let tex = include_str!("../../../tools/perfect_kernel/repros/loader/jmlr_object_refs.tex");
  assert_eq!(
    hf3_element(tex, "p", "Table"),
    "<p xml:id=\"p1.1\">Table\u{a0}<ref labelref=\"LABEL:tab:raw_perf\"/> and Figure\u{a0}<ref labelref=\"LABEL:tab:raw_perf\"/> and Section\u{a0}<ref labelref=\"LABEL:tab:raw_perf\"/>.</p>"
  );
}

/// 63t: fcs.cls's `{biography}` sets the photo beside the text (fcs.cls:637-660, :684) and `{competinginterest}` heads
/// its paragraph (fcs.cls:570-588; 2504.14891). Repro loader/fcs_biography.
#[test]
fn fcs_biography() {
  let tex = include_str!("../../../tools/perfect_kernel/repros/loader/fcs_biography.tex");
  assert_eq!(
    hf3_element(tex, "float", "biography"),
    "<float class=\"biography\" xml:id=\"tab1\">\n    <tabular xml:id=\"tab1.1\">\n      <tr xml:id=\"tab1.1.1\">\n        <td xml:id=\"tab1.1.1.1\"><graphics graphic=\"photo/Aoran_Gan.jpg\" options=\"width=71.13188pt,keepaspectratio=true\" xml:id=\"g1\"/></td>\n        <td xml:id=\"tab1.1.1.2\"><inline-block xml:id=\"tab1.1.1.2.1\">\n            <p xml:id=\"tab1.1.1.2.1.1\">Aoran Gan is a PhD student.</p>\n          </inline-block></td>\n      </tr>\n    </tabular>\n  </float>"
  );
  assert_eq!(
    hf3_element(tex, "p", "Competing"),
    "<p xml:id=\"p2.1\"><text font=\"bold\" xml:id=\"p2.1.1\">Competing interests</text>\u{2003}\nThe authors declare none.</p>"
  );
}

/// 63t: newclude's `\include*{file}` inputs the file, and `\include[pre]{file}[post]` it between the two hooks
/// (newclude.sty:743-750; 2507.19635). Repro loader/newclude_include_star.
#[test]
fn newclude_include_star() {
  let tex = include_str!("../../../tools/perfect_kernel/repros/loader/newclude_include_star.tex");
  assert_eq!(
    hf3_element(tex, "p", "Intro"),
    "<p xml:id=\"p1.1\">Intro text.\nPREMore text.\nPOST</p>"
  );
}

/// 63t: biblatex's `\Citeauthor` is its capitalized `\citeauthor` (biblatex.def:2624-2625; 2410.22329). In this
/// numeric style the citation is the bracketed number, where biblatex prints the name: an open residual, RED
/// index-bib/biblatex_citeauthor_numeric_prints_name. Repro loader/biblatex_capital_citeauthor.
#[test]
fn biblatex_capital_citeauthor() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/loader/biblatex_capital_citeauthor.tex");
  assert_eq!(
    hf3_element(tex, "p", "showed"),
    "<p xml:id=\"p1.1\"><cite class=\"ltx_citemacro_cite\">[<bibref bibrefs=\"Hampe_2017\" separator=\",\" yyseparator=\",\"/>]</cite> showed it.</p>"
  );
}

/// 63t: jfm.cls builds subeqnarray in (jfm.cls:663-712; 2512.18771). Repro loader/jfm_subeqnarray.
#[test]
fn jfm_subeqnarray() {
  let (log, xml) = latexml::util::test::convert_with(
    include_str!("../../../tools/perfect_kernel/repros/loader/jfm_subeqnarray.tex"),
    Some("ar5iv.sty"),
  );
  assert_eq!(super::perfect_kernel_batch46::error_count(&log), 0, "{log}");
  assert_eq!(
    super::perfect_kernel_batch46::warning_count(&log),
    0,
    "{log}"
  );
  // the rows numbered (1a), (1b) under the group's (1)
  for (row, id) in [("1a", "S0.E1.1.1"), ("1b", "S0.E1.2.1")] {
    let tags = format!(
      "<tags>\n            <tag>({row})</tag>\n            <tag role=\"autoref\">Equation\u{a0}{row}<text xml:id=\"{id}\"/></tag>\n            <tag role=\"refnum\">{row}</tag>\n          </tags>"
    );
    assert!(xml.contains(&tags), "{xml}");
  }
  assert_eq!(
    element_with(&xml, "Math", "p_{x}="),
    "<Math tex=\"\\displaystyle p_{x}=\\mu u_{zz}\" text=\"p _ x = mu * u _ (z * z)\" xml:id=\"S0.E1.1.m4\">\n              <XMath xml:id=\"S0.E1.1.m4.1\">\n                <XMApp xml:id=\"S0.E1.1.m4.1.1\">\n                  <XMTok meaning=\"equals\" role=\"RELOP\">=</XMTok>\n                  <XMApp xml:id=\"S0.E1.1.m4.1.1.2\">\n                    <XMTok role=\"SUBSCRIPTOP\" scriptpos=\"post1\"/>\n                    <XMTok font=\"italic\" role=\"UNKNOWN\">p</XMTok>\n                    <XMTok font=\"italic\" fontsize=\"70%\" role=\"UNKNOWN\">x</XMTok>\n                  </XMApp>\n                  <XMApp xml:id=\"S0.E1.1.m4.1.1.3\">\n                    <XMTok meaning=\"times\" role=\"MULOP\">⁢</XMTok>\n                    <XMTok font=\"italic\" name=\"mu\" role=\"UNKNOWN\">μ</XMTok>\n                    <XMApp xml:id=\"S0.E1.1.m4.1.1.3.3\">\n                      <XMTok role=\"SUBSCRIPTOP\" scriptpos=\"post1\"/>\n                      <XMTok font=\"italic\" role=\"UNKNOWN\">u</XMTok>\n                      <XMApp xml:id=\"S0.E1.1.m4.1.1.3.3.3\">\n                        <XMTok meaning=\"times\" role=\"MULOP\">⁢</XMTok>\n                        <XMTok font=\"italic\" fontsize=\"70%\" role=\"UNKNOWN\">z</XMTok>\n                        <XMTok font=\"italic\" fontsize=\"70%\" role=\"UNKNOWN\">z</XMTok>\n                      </XMApp>\n                    </XMApp>\n                  </XMApp>\n                </XMApp>\n              </XMath>\n            </Math>"
  );
  assert_eq!(
    element_with(&xml, "Math", "p_{z}="),
    "<Math tex=\"\\displaystyle p_{z}=-\\rho g\" text=\"p _ z = - rho * g\" xml:id=\"S0.E1.2.m4\">\n              <XMath xml:id=\"S0.E1.2.m4.1\">\n                <XMApp xml:id=\"S0.E1.2.m4.1.1\">\n                  <XMTok meaning=\"equals\" role=\"RELOP\">=</XMTok>\n                  <XMApp xml:id=\"S0.E1.2.m4.1.1.2\">\n                    <XMTok role=\"SUBSCRIPTOP\" scriptpos=\"post1\"/>\n                    <XMTok font=\"italic\" role=\"UNKNOWN\">p</XMTok>\n                    <XMTok font=\"italic\" fontsize=\"70%\" role=\"UNKNOWN\">z</XMTok>\n                  </XMApp>\n                  <XMApp xml:id=\"S0.E1.2.m4.1.1.3\">\n                    <XMTok meaning=\"minus\" role=\"ADDOP\">-</XMTok>\n                    <XMApp xml:id=\"S0.E1.2.m4.1.1.3.2\">\n                      <XMTok meaning=\"times\" role=\"MULOP\">⁢</XMTok>\n                      <XMTok font=\"italic\" name=\"rho\" role=\"UNKNOWN\">ρ</XMTok>\n                      <XMTok font=\"italic\" role=\"UNKNOWN\">g</XMTok>\n                    </XMApp>\n                  </XMApp>\n                </XMApp>\n              </XMath>\n            </Math>"
  );
}

/// 63t: breqn's math-active `_` and `^` (mathstyle.sty:233-236, :250-261) still script in math, siunitx's units
/// among them (siunitx.sty:6810-6813), and `_` in text prints (2402.04396, 2403.03720). The one warning is the binding's own "breqn.sty is not implemented". Repro
/// loader/breqn_text_underscore.
#[test]
fn breqn_text_underscore() {
  let (log, xml) = latexml::util::test::convert_with(
    include_str!("../../../tools/perfect_kernel/repros/loader/breqn_text_underscore.tex"),
    Some("ar5iv.sty"),
  );
  assert_eq!(super::perfect_kernel_batch46::error_count(&log), 0, "{log}");
  assert_eq!(
    super::perfect_kernel_batch46::warning_count(&log),
    1,
    "{log}"
  );
  assert_eq!(
    element_with(&xml, "p", "boiler"),
    "<p xml:id=\"p1.1\">Code at https://github.com/x/boiler_room and <Math mode=\"inline\" tex=\"a_{1}\" text=\"a _ 1\" xml:id=\"p1.m1\">\n        <XMath xml:id=\"p1.m1.1\">\n          <XMApp xml:id=\"p1.m1.1.1\">\n            <XMTok role=\"SUBSCRIPTOP\" scriptpos=\"post1\"/>\n            <XMTok font=\"italic\" role=\"UNKNOWN\">a</XMTok>\n            <XMTok fontsize=\"70%\" meaning=\"1\" role=\"NUMBER\">1</XMTok>\n          </XMApp>\n        </XMath>\n      </Math>, <Math mode=\"inline\" tex=\"\\mathrm{m}^{2}\" text=\"m ^ 2\" xml:id=\"p1.m2\">\n        <XMath xml:id=\"p1.m2.1\">\n          <XMApp xml:id=\"p1.m2.1.1\">\n            <XMTok role=\"SUPERSCRIPTOP\" scriptpos=\"post1\"/>\n            <XMTok role=\"UNKNOWN\">m</XMTok>\n            <XMTok fontsize=\"70%\" meaning=\"2\" role=\"NUMBER\">2</XMTok>\n          </XMApp>\n        </XMath>\n      </Math>.</p>"
  );
  assert_eq!(
    element_with(&xml, "equation", "y_{2}"),
    "<equation xml:id=\"S0.E1\">\n      <tags>\n        <tag>(1)</tag>\n        <tag role=\"refnum\">1</tag>\n      </tags>\n      <Math mode=\"display\" tex=\"y_{2}=x^{2}\" text=\"y _ 2 = x ^ 2\" xml:id=\"S0.E1.m1\">\n        <XMath xml:id=\"S0.E1.m1.1\">\n          <XMApp xml:id=\"S0.E1.m1.1.1\">\n            <XMTok meaning=\"equals\" role=\"RELOP\">=</XMTok>\n            <XMApp xml:id=\"S0.E1.m1.1.1.2\">\n              <XMTok role=\"SUBSCRIPTOP\" scriptpos=\"post1\"/>\n              <XMTok font=\"italic\" role=\"UNKNOWN\">y</XMTok>\n              <XMTok fontsize=\"70%\" meaning=\"2\" role=\"NUMBER\">2</XMTok>\n            </XMApp>\n            <XMApp xml:id=\"S0.E1.m1.1.1.3\">\n              <XMTok role=\"SUPERSCRIPTOP\" scriptpos=\"post1\"/>\n              <XMTok font=\"italic\" role=\"UNKNOWN\">x</XMTok>\n              <XMTok fontsize=\"70%\" meaning=\"2\" role=\"NUMBER\">2</XMTok>\n            </XMApp>\n          </XMApp>\n        </XMath>\n      </Math>\n    </equation>"
  );
  // `_` is "other" in the body, unless `mathstyleoff` (breqn.sty:42-44, flexisym.sty:401-402) asks for TeX's 8
  for (options, catcode) in [("", "12"), ("[mathstyleoff]", "8")] {
    let (log, xml) = latexml::util::test::convert_with(
      &format!(
        "\\documentclass{{article}}\\usepackage{options}{{breqn}}\\begin{{document}}\\the\\catcode`\\_\\end{{document}}"
      ),
      Some("ar5iv.sty"),
    );
    assert_eq!(super::perfect_kernel_batch46::error_count(&log), 0, "{log}");
    assert_eq!(
      super::perfect_kernel_batch46::warning_count(&log),
      1,
      "{log}"
    );
    assert_eq!(
      element_with(&xml, "p", catcode),
      format!("<p xml:id=\"p1.1\">{catcode}</p>")
    );
  }
}

/// 63t: cas-common.sty prints `\ead`'s address stringified (cas-common.sty:362; 2410.07921), so its `_` prints, in the
/// T1 font the class loads; the contact holds the address as its own text, which the HTML `mailto:` link is made of.
/// Repro sectioning-frontmatter/cas_ead_verbatim.
#[test]
fn cas_ead_verbatim() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/cas_ead_verbatim.tex"
  );
  assert_eq!(
    hf3_element(tex, "creator", "A."),
    "<creator role=\"author\">\n    <personname>A. B</personname>\n    <contact name=\"Email:\u{a0}\" role=\"email\">arash_khajooei@alumni.iust.ac.ir</contact>\n  </creator>"
  );
  let html = latexml::api::convert_to_html(tex).expect("HTML");
  assert!(
    html.contains(
      "<a href=\"mailto:arash_khajooei@alumni.iust.ac.ir\">arash_khajooei@alumni.iust.ac.ir</a>"
    ),
    "{html}"
  );
}

/// 63t: acmart prints `\acmDOI`'s value through `\url` (acmart.cls:2052; 2401.02563). Repro
/// sectioning-frontmatter/acmart_doi_verbatim.
#[test]
fn acmart_doi_verbatim() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/acmart_doi_verbatim.tex"
  );
  assert_eq!(
    hf3_element(tex, "pubnote", "doi"),
    "<pubnote name=\"DOI:\u{a0}\" role=\"doi\">10.475/123_4</pubnote>"
  );
}

/// 63t: `\underbar` boxes its argument in text mode (latex.ltx:619), so `\underbar{$\psi$}` in math stays in math
/// (2308.06669; Perl lets it to `\underline`, KPE #542); an argument without a `$` keeps the formula reading (OD #469). Repro expansion-primitives/underbar_boxes_its_argument.
#[test]
fn underbar_boxes_its_argument() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/underbar_boxes_its_argument.tex"
  );
  assert_eq!(
    hf3_element(tex, "Math", "underline"),
    "<Math mode=\"inline\" tex=\"\\underline{\\hbox{$\\psi$}}\\in P\" text=\"underline@(psi) element-of P\" xml:id=\"p1.m1\">\n        <XMath xml:id=\"p1.m1.1\">\n          <XMApp xml:id=\"p1.m1.1.1\">\n            <XMTok meaning=\"element-of\" name=\"in\" role=\"RELOP\">∈</XMTok>\n            <XMApp xml:id=\"p1.m1.1.1.2\">\n              <XMTok name=\"underline\" role=\"UNDERACCENT\" stretchy=\"true\">¯</XMTok>\n              <XMTok font=\"italic\" name=\"psi\" role=\"UNKNOWN\">ψ</XMTok>\n            </XMApp>\n            <XMTok font=\"italic\" role=\"UNKNOWN\">P</XMTok>\n          </XMApp>\n        </XMath>\n      </Math>"
  );
  // without a `$` of its own the argument keeps the formula reading: an underlined variable
  assert_eq!(
    hf3_element(tex, "Math", "\\underline{x}"),
    "<Math mode=\"inline\" tex=\"\\underline{x}\" text=\"underline@(x)\" xml:id=\"p1.m3\">\n        <XMath xml:id=\"p1.m3.1\">\n          <XMApp xml:id=\"p1.m3.1.1\">\n            <XMTok name=\"underline\" role=\"UNDERACCENT\" stretchy=\"true\">¯</XMTok>\n            <XMTok font=\"italic\" role=\"UNKNOWN\">x</XMTok>\n          </XMApp>\n        </XMath>\n      </Math>"
  );
}

/// 63t: `\TextOrMath` defers its choice to a `\protected` second stage (latex.ltx:10243-10248), so at an alignment
/// cell's start it chooses in the cell's math (2311.14468; Perl chooses the text branch there, KPE #543). Repro
/// expansion-primitives/textormath_in_align_cell.
#[test]
fn textormath_in_align_cell() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/textormath_in_align_cell.tex"
  );
  assert_eq!(
    hf3_element(tex, "p", "Text"),
    "<p xml:id=\"p1.1\">Text <Math mode=\"inline\" tex=\"L_{c}\" text=\"L _ c\" xml:id=\"p1.m1\">\n        <XMath xml:id=\"p1.m1.1\">\n          <XMApp xml:id=\"p1.m1.1.1\">\n            <XMTok role=\"SUBSCRIPTOP\" scriptpos=\"post1\"/>\n            <XMTok font=\"italic\" role=\"UNKNOWN\">L</XMTok>\n            <XMTok font=\"italic\" fontsize=\"70%\" role=\"UNKNOWN\">c</XMTok>\n          </XMApp>\n        </XMath>\n      </Math> here.</p>"
  );
  assert_eq!(
    hf3_element(tex, "Math", "L_{c}=x"),
    "<Math tex=\"\\displaystyle L_{c}=x_{i}\" text=\"L _ c = x _ i\" xml:id=\"S0.E1.m2\">\n            <XMath xml:id=\"S0.E1.m2.1\">\n              <XMApp xml:id=\"S0.E1.m2.1.1\">\n                <XMTok meaning=\"equals\" role=\"RELOP\">=</XMTok>\n                <XMApp xml:id=\"S0.E1.m2.1.1.2\">\n                  <XMTok role=\"SUBSCRIPTOP\" scriptpos=\"post1\"/>\n                  <XMTok font=\"italic\" role=\"UNKNOWN\">L</XMTok>\n                  <XMTok font=\"italic\" fontsize=\"70%\" role=\"UNKNOWN\">c</XMTok>\n                </XMApp>\n                <XMApp xml:id=\"S0.E1.m2.1.1.3\">\n                  <XMTok role=\"SUBSCRIPTOP\" scriptpos=\"post1\"/>\n                  <XMTok font=\"italic\" role=\"UNKNOWN\">x</XMTok>\n                  <XMTok font=\"italic\" fontsize=\"70%\" role=\"UNKNOWN\">i</XMTok>\n                </XMApp>\n              </XMApp>\n            </XMath>\n          </Math>"
  );
}

/// 63t: a contact's HTML link is made of its whole text, so an address inside an element of its own (the font
/// wrapper of a `\small` after the title block, RED sectioning-frontmatter/frontmatter_digested_in_body_font; 2609.14733,
/// 2609.03698) still links to itself (Perl's stylesheet: `text()`, the link `mailto:` alone). Repro
/// sectioning-frontmatter/contact_link_from_whole_text.
#[test]
fn contact_link_from_whole_text() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/contact_link_from_whole_text.tex"
  );
  let (log, _xml) = latexml::util::test::convert_with(tex, Some("ar5iv.sty"));
  assert_eq!(super::perfect_kernel_batch46::error_count(&log), 0, "{log}");
  assert_eq!(
    super::perfect_kernel_batch46::warning_count(&log),
    0,
    "{log}"
  );
  let html = latexml::api::convert_to_html(tex).expect("HTML");
  let at = html.find("<a href=\"mailto:").expect("the email link");
  assert_eq!(
    &html[at..at + html[at..].find("</a>").unwrap() + 4],
    "<a href=\"mailto:ab@x.org\"><span class=\"ltx_text\" style=\"font-size:90%;\">ab@x.org</span></a>",
    "{html}"
  );
  // an address that is the contact's own text links to that text alone, not the marks after it; a contact that is
  // already a link (`\href`) is not wrapped in another
  let html = latexml::api::convert_to_html(
    r"\documentclass{revtex4-2}
\usepackage{hyperref}
\begin{document}
\title{T}
\author{A. B}
\email{a@b.org \textsuperscript{*}}
\homepage{http://x.org/ \textit{(lab)}}
\author{C. D}
\email{\href{mailto:c@d.org}{c@d.org}}
\maketitle
Body.
\end{document}",
  )
  .expect("HTML");
  for anchor in [
    "<a href=\"mailto:a@b.org\">",
    "<a href=\"http://x.org/\">",
    "<span class=\"ltx_contact_name\">Email:\u{a0}</span><a href=\"mailto:c@d.org\" title=\"\" class=\"ltx_ref ltx_href\">c@d.org</a>",
  ] {
    assert!(html.contains(anchor), "{anchor}\n{html}");
  }
}
