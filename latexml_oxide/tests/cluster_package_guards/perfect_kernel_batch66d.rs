//! Red/green guards for batch 66d: run 340's Fatal origins, round 2 (`~/data/pk_agents/main/run340/fatal/
//! fatal_rootcauses.md` R1, R3, R4, R5, R6, R10, R13). Repros `tools/perfect_kernel/repros/*`.
use super::perfect_kernel_batch46::{error_count, warning_count};

fn convert_counted(tex: &str) -> (String, String) {
  latexml::util::test::convert_with(tex, Some("ar5iv.sty"))
}

/// R1. The caption-label hoist closes its `\ifx.#5.` test before the caption is digested (latex.ltx's `\@firstoftwo`
/// idiom): a stray `\fi` in a caption (listings' `\lst@ifdisplaystyle…\fi` with `@` not a letter, 2012.08626,
/// 2210.17505) took the wrapper's own `\fi`, and the `\label` scan read to the end of the file (Fatal `Mouth:EoF`).
/// pdflatex reports the stray `\fi` as "Extra \fi", here "Didn't expect \fi".
#[test]
fn caption_hoist_stray_fi_keeps_its_caption() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/captions-floats/caption_hoist_stray_fi.tex"
  ));
  assert!(!log.contains("Fatal:"), "{log}");
  assert_eq!(log.matches("Error:unexpected:fi ").count(), 2, "{log}");
  assert_eq!(error_count(&log), 2, "{log}");
  assert!(
    xml.contains(r#"<figure inlist="lof" labels="LABEL:f:a" xml:id="S0.F1">"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<caption><tag close=": ">Figure 1</tag>A stray  here.</caption>"#),
    "{xml}"
  );
}

/// R1. An `\if` without its `\fi` in a caption (0807.3034's `\def\arcs{\ifmmode … \else …}`) stays open, as in TeX,
/// rather than taking the hoist's `\else`.
#[test]
fn caption_hoist_unbalanced_if_keeps_its_caption() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/captions-floats/caption_hoist_unbalanced_if.tex"
  ));
  assert_eq!(error_count(&log), 0, "{log}");
  assert!(
    xml.contains(r#"<figure inlist="lof" labels="LABEL:f:a" xml:id="S0.F1">"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<caption><tag close=": ">Figure 1</tag>The 3<sup "#),
    "{xml}"
  );
  assert!(xml.contains("</sup> fiber.</caption>"), "{xml}");
}

/// R3. AMS-TeX's `\Bbb` and `\bold` are font switches of their own (amstex.tex:1263-1266, 1358-1361): the idiom
/// `\def\mathbb{\Bbb}` looped through the amsfonts binding's alias of `\mathbb` (1112.1423, 1108.6264, 1012.3760,
/// 1201.0265).
#[test]
fn amstex_bbb_and_bold_are_font_switches() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/amstex/amstex_mathbb_is_Bbb.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  for tok in [
    r#"<XMTok font="blackboard" role="UNKNOWN">R</XMTok>"#,
    r#"<XMTok font="blackboard" role="UNKNOWN">Z</XMTok>"#,
    r#"<XMTok font="bold" role="UNKNOWN">B</XMTok>"#,
  ] {
    assert!(xml.contains(tok), "{tok}\n{xml}");
  }
}

/// R3. amsfonts.sty:115-116 maps `\bold` to `\mathbf` (Perl's binding: `\mathbb`), so a paper's
/// `\renewcommand{\mathbb}{\bold}` is bold, not a loop (1611.03179).
#[test]
fn amsfonts_bold_is_mathbf() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/amstex/amsfonts_bold_is_mathbf.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    xml.contains(r#"<XMTok font="bold" role="UNKNOWN">Q</XMTok>"#),
    "{xml}"
  );
}

/// R4. TL amsppt1.tex's AMS-TeX 1.x shims (`\long\def\abstract#1{\ab@#1\endabstract}`) are taken: the LaTeX kernel's
/// `\abstract` lock is not AMS-TeX's, and amsppt.sty's state the file reads (`\add@missing`, `\envir@end`,
/// `\addresscount@`, `\ifmonograph@`, `\thetranslator@`) exists (alg-geom9505028, alg-geom9606011, 2310.08131,
/// q-alg9505010, q-alg9507011: Fatal `Document:Malformed`).
#[test]
fn amsppt1_abstract_shim_is_kept() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/amstex/amsppt1_abstract_braced.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(xml.contains("<title>A title</title>"), "{xml}");
  assert!(
    xml.contains(r#"<p xml:id="abstract1.1">We shall investigate flips.</p>"#),
    "{xml}"
  );
  assert!(xml.contains(r#"<p xml:id="p1.1">Body text.</p>"#), "{xml}");
}

/// R6. `{filecontents}` ends at `\end{<env>}` anywhere in a line (latex.ltx:19047-19066), its text before written as
/// the last line and its text after ignored, each with LaTeX's warning (1909.03404, 2311.02007, 2503.03764: the rest
/// of the document was captured, Fatal `Document:Malformed`).
#[test]
fn filecontents_ends_mid_line() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/macro-state/filecontents_end_midline.tex"
  ));
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 2, "{log}");
  assert!(
    log.contains(
      "Writing text `\\end{tikzpicture}' before \\end{filecontents*} as last line of a.tikz"
    ),
    "{log}"
  );
  assert!(
    log.contains("Ignoring text `{b.tex}' after \\end{filecontents}"),
    "{log}"
  );
  assert!(
    xml.contains(r#"<p xml:id="p1.1">Body. Hello from b.</p>"#),
    "{xml}"
  );
}

/// R10. The kernel's `\shorttitle`/`\shortauthor` aliases are stubs a document's `\newcommand` replaces (latex.ltx has
/// neither): the refused definition left `\title[\shorttitle]` looping through `\@shorttitle` (1105.4905, 0909.2464).
#[test]
fn shorttitle_is_the_documents_own() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/shorttitle_newcommand_by_document.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(xml.contains("<title>A long title</title>"), "{xml}");
  assert!(xml.contains("<toctitle>Short title</toctitle>"), "{xml}");
}

/// R13. `\lbrack` is a macro, as plain.tex:509 and latex.ltx:562 define it: tikz-cd's `\pgfutil@ifnextchar[` no longer
/// takes a first cell opening with `\lbrack` for an options bracket (2212.11079: Fatal `Timeout:PushbackLimit`).
#[test]
fn lbrack_is_a_macro() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/macro-state/tikzcd_lbrack_first_cell.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(
    xml
      .matches(r#"class="ltx_tikzmatrix_col ltx_nopad_l ltx_nopad_r""#)
      .count(),
    2,
    "{xml}"
  );
  // The first cell keeps its brackets around `A , B`, before the second cell's `C`.
  let mut at = 0;
  for tok in [
    r#"<XMTok role="OPEN" stretchy="false">[</XMTok>"#,
    r#">A</XMTok>"#,
    r#"<XMTok role="PUNCT">,</XMTok>"#,
    r#">B</XMTok>"#,
    r#"<XMTok role="CLOSE" stretchy="false">]</XMTok>"#,
    r#">C</XMTok>"#,
  ] {
    at += xml[at..]
      .find(tok)
      .unwrap_or_else(|| panic!("{tok} after byte {at}\n{xml}"))
      + tok.len();
  }
}

/// amsppt.sty:820-836 `\topcaption`/`\botcaption#1#2\endcaption`: the label in the caption font, `.\enspace`, the text,
/// a paragraph of the (no-op) insert; undefined, 2 errors in q-alg9505010, q-alg9507011.
#[test]
fn amsppt_captions_are_paragraphs() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/amstex/amsppt_botcaption.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  for para in [
    "<p xml:id=\"p2.1\"><text font=\"smallcaps\" xml:id=\"p2.1.1\">Figure 1</text>.\u{2002}Variables organized in a pattern</p>",
    "<p xml:id=\"p3.1\"><text font=\"smallcaps\" xml:id=\"p3.1.1\">Table 2</text>.\u{2002}Values</p>",
    // No label: no separator; the caption's own font change ends with it.
    "<p xml:id=\"p4.1\"><text font=\"italic\" xml:id=\"p4.1.1\">Unlabelled</text> plate</p>",
    "<p xml:id=\"p5.1\">More text.</p>",
  ] {
    assert!(xml.contains(para), "{para}\n{xml}");
  }
}
