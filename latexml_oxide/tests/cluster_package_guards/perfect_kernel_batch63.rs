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
      "<creator role=\"author\"><personname>Carlo Condo</personname></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Pascal Giard, <emph font=\"italic\" xml:id=\"id1\">Member, IEEE</emph></personname></creator>",
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
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Marina Krekhova</personname></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Günter Lattermann</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><text font=\"slanted\" xml:id=\"id2\">Makromolekulare Chemie I, Univ. Bayreuth</text></contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Ingo Rehberg</personname></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Reinhard Richter</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><text font=\"slanted\" xml:id=\"id3\">Experimentalphysik V, Univ. Bayreuth</text></contact></creator>",
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
/// empty author taking a copy of the block's affiliation. Repro
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
/// not a creator of its own (review r8). The line sits under both names in the PDF; that only the last name holds
/// it is the OD #459 residual (a names line's affiliations go to its last name), not the target. Repro
/// sectioning-frontmatter/author_emulateapj_affiliation_line_altaffilmark_stays_with_author.
#[test]
fn author_emulateapj_affiliation_line_altaffilmark_stays_with_author() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_emulateapj_affiliation_line_altaffilmark_stays_with_author.tex"
    ),
    &[
      "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Alternate Affiliation:\u{a0}\" role=\"altaffiliation\">Fellow</contact></creator>",
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
      "<creator role=\"author\"><personname>G.N.Abramov</personname></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>M.N.Achasov</personname></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>A.G.Bogdanchikov</personname></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Yu.M.Shatunov</personname></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>A.V.Vasiljev</personname></creator>",
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
      "<creator role=\"author\"><personname>Ivan Petrov</personname></creator>",
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
