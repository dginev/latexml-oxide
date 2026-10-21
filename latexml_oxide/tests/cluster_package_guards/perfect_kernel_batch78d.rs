//! Red/green guards for batch 78d (quality batch 7): REVTeX's section numbers, IEEEtran's `\appendix[<title>]`, the
//! width registers of subcaption's sub-floats, mdpi's caption settings, and a parameter text's closing `#{`. Each guard
//! converts its repro under `tools/perfect_kernel/repros/`, asserts the diagnostics by count and the authored content in
//! whole elements, against pdflatex. Witnesses 2509.19464 (subfigure widths), 2610.26441 (tkz-euclide's `#{`).
use latexml::util::test::{assert_element, convert_with};

use super::perfect_kernel_batch46::{error_count, warning_count};

const PRELOAD: &str = "ar5iv.sty";

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!("../../../tools/perfect_kernel/repros/", $path))
  };
}

/// Convert `tex`: 0 errors, 0 warnings, and each `(tag, xml:id, element)` as given.
fn assert_clean(tex: &str, expected: &[(&str, &str, &str)]) -> String {
  let (log, xml) = convert_with(tex, Some(PRELOAD));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  for (tag, id, element) in expected {
    assert_element(&xml, tag, &[&format!("xml:id=\"{id}\"")], element);
  }
  xml
}

/// The text of every `<tag>` (the heading form) and `<tag role="refnum">` in document order, as `form|r:refnum`.
fn heading_numbers(xml: &str) -> String {
  let re = regex::Regex::new(r#"<tag( role="refnum")?>(.*?)</tag>"#).unwrap();
  let markup = regex::Regex::new(r"<[^>]+>").unwrap();
  re.captures_iter(xml)
    .map(|c| {
      let text = markup.replace_all(&c[2], "");
      if c.get(1).is_some() {
        format!("r:{text}")
      } else {
        text.to_string()
      }
    })
    .collect::<Vec<_>>()
    .join("|")
    .replace('\u{2009}', "_")
}

/// REVTeX numbers its sections "I", "A", "1", "a" and chains a `\ref` with thin spaces through `\p@<level>`
/// (revtex4-2.cls:6528-6551 `\secnums@rtx`), numbers paragraphs (`secnumdepth` 4, :6677), prints a heading's number
/// as `\the<level>.` (aps4-2.rtx:288), and in the appendix numbers the subsections `\arabic` and the equations
/// "A1" (:7372-7387). Red (int45b; Perl revtex4_support.sty.ltxml:111 defines `\thesection` only): "I", "I.1",
/// "I.1.1", the paragraph unnumbered, appendix "A.1", equation "(2)". The variants' forms are each pdflatex's (probes
/// with revtex4-2, revtex4-1, revtex4): `rmp` periods (apsrmp4-2.rtx:124-148), `secnumarabic` "I.1" in 4.2 and "1.1"
/// in 4.0, `aapm` "I.A" (aapm4-2.rtx:290-314), `eqsecnum` "(1.1)", `\appendix*` "(A.1)". The PRL journal unnumbers
/// every heading (`\c@secnumdepth=-\maxdimen`, aps4-2.rtx:587, aps4-1.rtx:564, revtex4's aps.rtx:517), its appendix
/// equations counting on, "(2)" (pdflatex; 1508.03506's PDF "(16)"); red (78d r1, rev78d finding 2): every number
/// above and "(A1)". The journal is the first named in 4.1/4.2 (`[prb,prl]` numbered) and the last declared in 4.0
/// (`[prb,prl]` PRL); AIP's `cp`, whose `secnumdepth` 0 (aip4-2.rtx:1181) never runs, stays numbered. Repro
/// sectioning-frontmatter/revtex_sections_number_as_the_class.
#[test]
fn revtex_sections_number_as_the_class() {
  let tex = repro!("sectioning-frontmatter/revtex_sections_number_as_the_class.tex");
  let xml = assert_clean(tex, &[
    (
      "section",
      "S1",
      "<section inlist=\"toc\" labels=\"LABEL:s1\" xml:id=\"S1\"><tags><tag>I.</tag>\
 <tag role=\"autoref\">section\u{00A0}I<text xml:id=\"S1.1\"/></tag><tag role=\"refnum\">I</tag>\
 <tag role=\"typerefnum\">§I</tag></tags><title><tag close=\" \">I.</tag>Intro</title><toctitle>\
 <tag close=\" \">I</tag>Intro</toctitle><para xml:id=\"S1.p1\">\
 <equation labels=\"LABEL:e1\" xml:id=\"S1.E1\"><tags><tag>(1)</tag>\
 <tag role=\"autoref\">Equation\u{00A0}1<text xml:id=\"S1.E1.1\"/></tag><tag role=\"refnum\">1</tag>\
 </tags><Math mode=\"display\" tex=\"a=b\" text=\"a = b\" xml:id=\"S1.E1.m1\">\
 <XMath xml:id=\"S1.E1.m1.1\"><XMApp xml:id=\"S1.E1.m1.1.1\">\
 <XMTok meaning=\"equals\" role=\"RELOP\">=</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\">a</XMTok>\
 <XMTok font=\"italic\" role=\"UNKNOWN\">b</XMTok></XMApp></XMath></Math></equation></para>\
 <subsection inlist=\"toc\" labels=\"LABEL:s11\" xml:id=\"S1.SS1\"><tags><tag>A.</tag>\
 <tag role=\"autoref\">subsection\u{00A0}I\u{2009}A<text xml:id=\"S1.SS1.1\"/></tag>\
 <tag role=\"refnum\">I\u{2009}A</tag><tag role=\"typerefnum\">§I\u{2009}A</tag></tags><title>\
 <tag close=\" \">A.</tag>Sub</title><toctitle><tag close=\" \">A</tag>Sub</toctitle>\
 <subsubsection inlist=\"toc\" labels=\"LABEL:s111\" xml:id=\"S1.SS1.SSS1\"><tags><tag>1.</tag>\
 <tag role=\"autoref\">subsubsection\u{00A0}I\u{2009}A\u{2009}1<text xml:id=\"S1.SS1.SSS1.1\"/></tag>\
 <tag role=\"refnum\">I\u{2009}A\u{2009}1</tag><tag role=\"typerefnum\">§I\u{2009}A\u{2009}1</tag></tags>\
 <title><tag close=\" \">1.</tag>Subsub</title><toctitle><tag close=\" \">1</tag>Subsub</toctitle>\
 <paragraph inlist=\"toc\" labels=\"LABEL:p1\" xml:id=\"S1.SS1.SSS1.P1\"><tags><tag>a.</tag>\
 <tag role=\"autoref\">paragraph\u{00A0}I\u{2009}A\u{2009}1\u{2009}a<text xml:id=\"S1.SS1.SSS1.P1.1\"/>\
 </tag><tag role=\"refnum\">I\u{2009}A\u{2009}1\u{2009}a</tag>\
 <tag role=\"typerefnum\">¶I\u{2009}A\u{2009}1\u{2009}a</tag></tags><title>\
 <tag close=\" \">a.</tag>Para</title><toctitle><tag close=\" \">a</tag>Para</toctitle>\
 <para xml:id=\"S1.SS1.SSS1.P1.p1\">\
 <p xml:id=\"S1.SS1.SSS1.P1.p1.1\">See <ref labelref=\"LABEL:s1\"/>, <ref labelref=\"LABEL:s11\"/>, <ref labelref=\"LABEL:s111\"/>, <ref labelref=\"LABEL:p1\"/>, <ref labelref=\"LABEL:e1\"/>, <ref labelref=\"LABEL:e2\"/>, <ref labelref=\"LABEL:a1\"/>, <ref labelref=\"LABEL:a11\"/>.</p>\
 </para></paragraph></subsubsection></subsection></section>",
    ),
    (
      "appendix",
      "A1",
      "<appendix inlist=\"toc\" labels=\"LABEL:a1\" xml:id=\"A1\"><tags><tag>Appendix A</tag>\
  <tag role=\"autoref\">Appendix\u{00A0}A<text xml:id=\"A1.1\"/></tag><tag role=\"refnum\">A</tag>\
  <tag role=\"typerefnum\">Appendix A</tag></tags><title><tag close=\" \">Appendix A</tag>App</title>\
  <toctitle><tag close=\" \">A</tag>App</toctitle><para xml:id=\"A1.p1\">\
  <equation labels=\"LABEL:e2\" xml:id=\"A1.E1\"><tags><tag>(A1)</tag>\
  <tag role=\"autoref\">Equation\u{00A0}A1<text xml:id=\"A1.E1.1\"/></tag><tag role=\"refnum\">A1</tag>\
  </tags><Math mode=\"display\" tex=\"c=d\" text=\"c = d\" xml:id=\"A1.E1.m1\">\
  <XMath xml:id=\"A1.E1.m1.1\"><XMApp xml:id=\"A1.E1.m1.1.1\">\
  <XMTok meaning=\"equals\" role=\"RELOP\">=</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\">c</XMTok>\
  <XMTok font=\"italic\" role=\"UNKNOWN\">d</XMTok></XMApp></XMath></Math></equation></para>\
  <subsection inlist=\"toc\" labels=\"LABEL:a11\" xml:id=\"A1.SS1\"><tags><tag>1.</tag>\
  <tag role=\"autoref\">subsection\u{00A0}A\u{2009}1<text xml:id=\"A1.SS1.1\"/></tag>\
  <tag role=\"refnum\">A\u{2009}1</tag><tag role=\"typerefnum\">§A\u{2009}1</tag></tags><title>\
  <tag close=\" \">1.</tag>ASub</title><toctitle><tag close=\" \">1</tag>ASub</toctitle></subsection>\
  </appendix>",
    ),
  ]);
  assert_eq!(
    heading_numbers(&xml),
    "I.|r:I|(1)|r:1|A.|r:I_A|1.|r:I_A_1|a.|r:I_A_1_a|Appendix A|r:A|(A1)|r:A1|1.|r:A_1",
    "aps"
  );
  for (class, options, star, expected) in [
    (
      "revtex4-2",
      "aps,rmp",
      "",
      "I.|r:I|(1)|r:1|A.|r:I.A|1.|r:I.A.1|a.|r:I.A.1.a|Appendix A|r:A|(A1)|r:A1|1.|r:A.1",
    ),
    (
      "revtex4-2",
      "aps,secnumarabic",
      "",
      "I.|r:I|(1)|r:1|I.1.|r:I.1|I.1.1.|r:I.1.1|I.1.1.1.|r:I.1.1.1|Appendix A|r:A|(A1)|r:A1|1.|r:1",
    ),
    (
      "revtex4-2",
      "aapm",
      "",
      "I.|r:I|(1)|r:1|I.A.|r:I.A|I.A.1.|r:I.A.1|I.A.1.a.|r:I.A.1.a|Appendix A|r:A|(A1)|r:A1|1.|r:1",
    ),
    (
      "revtex4-2",
      "aps,eqsecnum",
      "",
      "I.|r:I|(1.1)|r:1.1|A.|r:I_A|1.|r:I_A_1|a.|r:I_A_1_a|Appendix A|r:A|(A1)|r:A1|1.|r:A_1",
    ),
    (
      "revtex4-2",
      "aps",
      "*",
      "I.|r:I|(1)|r:1|A.|r:I_A|1.|r:I_A_1|a.|r:I_A_1_a|Appendix|(A.1)|r:A.1|1.|r:_1",
    ),
    (
      "revtex4-1",
      "aps",
      "",
      "I.|r:I|(1)|r:1|A.|r:I_A|1.|r:I_A_1|a.|r:I_A_1_a|Appendix A|r:A|(A1)|r:A1|1.|r:A_1",
    ),
    (
      "revtex4",
      "aps,secnumarabic",
      "",
      "1.|r:1|(1)|r:1|1.1.|r:1.1|1.1.1.|r:1.1.1|a.|r:1_1.1_1.1.1_a|Appendix A|r:A|(A1)|r:A1|1.|r:1",
    ),
    (
      "revtex4",
      "aps,rmp",
      "",
      "I.|r:I|(1)|r:1|A.|r:I.A|1.|r:I.A.1|a.|r:I.A.1.a|Appendix A|r:A|(A1)|r:A1|1.|r:A.1",
    ),
    ("revtex4-2", "aps,prl", "", "(1)|r:1|(2)|r:2"),
    ("revtex4-1", "aps,prl", "", "(1)|r:1|(2)|r:2"),
    ("revtex4", "aps,prl", "", "(1)|r:1|(2)|r:2"),
    ("revtex4", "prb,prl", "", "(1)|r:1|(2)|r:2"),
    (
      "revtex4-2",
      "prb,prl",
      "",
      "I.|r:I|(1)|r:1|A.|r:I_A|1.|r:I_A_1|a.|r:I_A_1_a|Appendix A|r:A|(A1)|r:A1|1.|r:A_1",
    ),
    (
      "revtex4-2",
      "aip,cp",
      "",
      "I.|r:I|(1)|r:1|A.|r:I_A|1.|r:I_A_1|a.|r:I_A_1_a|Appendix A|r:A|(A1)|r:A1|1.|r:A_1",
    ),
  ] {
    let variant = tex
      .replace(
        "\\documentclass[aps,prb]{revtex4-2}",
        &format!("\\documentclass[{options}]{{{class}}}"),
      )
      .replace("\\appendix\n", &format!("\\appendix{star}\n"));
    let (log, xml) = convert_with(&variant, Some(PRELOAD));
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "{class}[{options}]{star}: {log}"
    );
    assert_eq!(heading_numbers(&xml), expected, "{class}[{options}]{star}");
  }
  // PRL's appendix: unnumbered, its equation the document's second.
  let variant = tex.replace(
    "\\documentclass[aps,prb]{revtex4-2}",
    "\\documentclass[aps,prl]{revtex4-2}",
  );
  let (log, xml) = convert_with(&variant, Some(PRELOAD));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "appendix",
    &["xml:id=\"Ax1\""],
    "<appendix inlist=\"toc\" labels=\"LABEL:a1\" xml:id=\"Ax1\"><title>App</title><para xml:id=\"Ax1.p1\">\
 <equation labels=\"LABEL:e2\" xml:id=\"Ax1.E2\"><tags><tag>(2)</tag>\
 <tag role=\"autoref\">Equation\u{00A0}2<text xml:id=\"Ax1.E2.1\"/></tag><tag role=\"refnum\">2</tag></tags>\
 <Math mode=\"display\" tex=\"c=d\" text=\"c = d\" xml:id=\"Ax1.E2.m1\"><XMath xml:id=\"Ax1.E2.m1.1\">\
 <XMApp xml:id=\"Ax1.E2.m1.1.1\"><XMTok meaning=\"equals\" role=\"RELOP\">=</XMTok>\
 <XMTok font=\"italic\" role=\"UNKNOWN\">c</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\">d</XMTok></XMApp>\
 </XMath></Math></equation></para><subsection inlist=\"toc\" labels=\"LABEL:a11\" xml:id=\"Ax1.SSx1\">\
 <title>ASub</title></subsection></appendix>",
  );
}

/// IEEEtran's `\appendix[<title>]` is one appendix (IEEEtran.cls:5719-5745): `\ref` "A", the heading "Appendix" with
/// the title (pdflatex "APPENDIX" "PROOF"), its subsection "A." with `\ref` "A". Red (int45b; rev78c2 probe i2): "[Proof]"
/// a paragraph of the last subsection, no appendix, the subsection "B." with refnum "B". Without the argument the class
/// prints "Appendix" alone and throws a following `\section`'s argument away; here that title is the appendix's
/// (OXIDIZED_DESIGN_DIVERGENCES #793), past a blank line, a comment line or a `\label` (red on 78d r1, rev78d finding 1:
/// an empty appendix "A" and the titled one "B"; witnesses 1903.01287, 1103.2068, 2002.03484, 2410.15995, 2401.07123).
/// A `\section*` after it follows the "Appendix" heading (pdflatex "APPENDIX" "STAR"); a later `\section` is
/// "Appendix B" (r1 "Appendix", finding 4); `\appendices` after `\appendix` numbers from "A" again (pdflatex "APPENDIX",
/// "APPENDIX A", "APPENDIX B"; r1 "Appendix" refnums B, C, finding 3). Repro
/// sectioning-frontmatter/ieeetran_appendix_with_a_title_is_one_appendix.
#[test]
fn ieeetran_appendix_with_a_title_is_one_appendix() {
  let tex = repro!("sectioning-frontmatter/ieeetran_appendix_with_a_title_is_one_appendix.tex");
  assert_clean(tex, &[(
    "appendix",
    "A1",
    "<appendix inlist=\"toc\" labels=\"LABEL:a1\" xml:id=\"A1\"><tags><tag>Appendix</tag>\
 <tag role=\"refnum\">A</tag><tag role=\"typerefnum\">Appendix A</tag></tags><title>\
 <tag close=\" \">Appendix</tag>Proof</title><toctitle><tag close=\" \">A</tag>Proof</toctitle>\
 <subsection inlist=\"toc\" labels=\"LABEL:a11\" xml:id=\"A1.SS1\"><tags><tag>A.</tag>\
 <tag role=\"refnum\">A</tag><tag role=\"typerefnum\">§A</tag></tags><title><tag close=\" \">A.</tag>\
 <text font=\"italic\" xml:id=\"A1.SS1.1\">Lemma</text></title><toctitle>\
 <tag close=\" \">A</tag>Lemma</toctitle><para xml:id=\"A1.SS1.p1\"><p xml:id=\"A1.SS1.p1.1\">Text.</p>\
 <equation labels=\"LABEL:e1\" xml:id=\"A1.E1\"><tags><tag>(1)</tag><tag role=\"refnum\">1</tag></tags>\
 <Math mode=\"display\" tex=\"x=y\" text=\"x = y\" xml:id=\"A1.E1.m1\"><XMath xml:id=\"A1.E1.m1.1\">\
 <XMApp xml:id=\"A1.E1.m1.1.1\"><XMTok meaning=\"equals\" role=\"RELOP\">=</XMTok>\
 <XMTok font=\"italic\" role=\"UNKNOWN\">x</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\">y</XMTok>\
 </XMApp></XMath></Math></equation></para></subsection></appendix>",
  )]);
  // `\appendix` then `\section{…}`: one appendix, tagged "Appendix", its title kept.
  let variant = tex.replace("\\appendix[Proof]\n", "\\appendix\n\\section{Proof of X}\n");
  let (log, xml) = convert_with(&variant, Some(PRELOAD));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(
    heading_numbers(&xml),
    "I.|r:I|A.|r:I-A|Appendix|r:A|A.|r:A|(1)|r:1",
    "\\appendix\\section"
  );
  // `\appendix` alone: the "Appendix" heading pdflatex prints, the subsection inside it.
  let variant = tex.replace("\\appendix[Proof]\n", "\\appendix\n");
  let (log, xml) = convert_with(&variant, Some(PRELOAD));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(
    heading_numbers(&xml),
    "I.|r:I|A.|r:I-A|Appendix|r:A|A.|r:A|(1)|r:1",
    "\\appendix"
  );
  // Each appendix as `id[labels] tag|title`, and the heading numbers, for `\appendix` followed by …
  let one = "I.|r:I|A.|r:I-A|Appendix|r:A|A.|r:A|(1)|r:1";
  let more = "\\section{More}\\label{a2}\nText.\n\\end{document}";
  for (name, replacement, rest, numbers, appendices) in [
    (
      "a blank line",
      "\\appendix\n\n\\section{Proof of X}\n",
      "",
      one,
      "A1[LABEL:a1] Appendix|Proof of X",
    ),
    (
      "a comment line",
      "\\appendix\n% a note\n\n\\section{Proof of X}\n",
      "",
      one,
      "A1[LABEL:a1] Appendix|Proof of X",
    ),
    (
      "a \\label",
      "\\appendix\n\\label{a0}\n\n\\section{Proof of X}\n",
      "",
      one,
      "A1[LABEL:a0 LABEL:a1] Appendix|Proof of X",
    ),
    (
      // (rev78d2 finding 1: 2002.07641's `\appendix{}`, an empty group before the `\section`)
      "an empty group",
      "\\appendix{}\n\n\\section{Proof of X}\n",
      "",
      one,
      "A1[LABEL:a1] Appendix|Proof of X",
    ),
    (
      "a \\section*",
      "\\appendix\n\n\\section*{Star}\n",
      "",
      one,
      "A1[] Appendix|; Ax1[LABEL:a1] |Star",
    ),
    (
      "a later \\section",
      "\\appendix\n\\section{Proof of X}\n",
      more,
      "I.|r:I|A.|r:I-A|Appendix|r:A|A.|r:A|(1)|r:1|Appendix B|r:B",
      "A1[LABEL:a1] Appendix|Proof of X; A2[LABEL:a2] Appendix B|More",
    ),
    (
      "\\appendices",
      "\\appendix\n\\appendices\n\\section{First}\n",
      more,
      "I.|r:I|A.|r:I-A|Appendix|r:A|Appendix A|r:A|A.|r:A|(1)|r:1|Appendix B|r:B",
      "A1[] Appendix|; A1a[LABEL:a1] Appendix A|First; A2[LABEL:a2] Appendix B|More",
    ),
  ] {
    let mut variant = tex.replace("\\appendix[Proof]\n", replacement);
    if !rest.is_empty() {
      variant = variant.replace("\\end{document}", rest);
    }
    let (log, xml) = convert_with(&variant, Some(PRELOAD));
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "{name}: {log}"
    );
    assert_eq!(heading_numbers(&xml), numbers, "{name}");
    assert_eq!(appendix_headings(&xml), appendices, "{name}");
  }
}

/// Each `<appendix>` as `id[labels] tag|title`: its heading tag and title text.
fn appendix_headings(xml: &str) -> String {
  let re =
    regex::Regex::new(r#"(?s)<appendix\b([^>]*)>\s*(?:<tags>.*?</tags>\s*)?<title>(.*?)</title>"#)
      .unwrap();
  let attr = |attrs: &str, name: &str| {
    regex::Regex::new(&format!(r#"{name}="([^"]*)""#))
      .unwrap()
      .captures(attrs)
      .map(|c| c[1].to_string())
      .unwrap_or_default()
  };
  let tag = regex::Regex::new(r#"^<tag close="[^"]*">(.*?)</tag>"#).unwrap();
  let markup = regex::Regex::new(r"<[^>]+>").unwrap();
  re.captures_iter(xml)
    .map(|c| {
      let title = &c[2];
      let (form, rest) = match tag.captures(title) {
        Some(t) => (t[1].to_string(), title[t[0].len()..].to_string()),
        None => (String::new(), title.to_string()),
      };
      format!(
        "{}[{}] {}|{}",
        attr(&c[1], "xml:id"),
        attr(&c[1], "labels"),
        markup.replace_all(&form, ""),
        markup.replace_all(&rest, "")
      )
    })
    .collect::<Vec<_>>()
    .join("; ")
}

/// A sub-float is subcaption's minipage (subcaption.sty:70-92): its width argument sets `\hsize`, `\textwidth`,
/// `\columnwidth` and `\linewidth` inside it (latex.ltx:16312-16313, `\@parboxrestore`), so a `width=\linewidth` image
/// is the panel's width. pdflatex "96.59958pt" ×4, the table's "155.24895pt" (`\textwidth` 345pt). Red (int45b; Perl's
/// sub-floats set only `\hsize`): all four 345.0pt. A `\subcaptionbox[<width>]` is a `\parbox` (caption.sty:473):
/// `\hsize` and `\linewidth` its width, `\textwidth` and `\columnwidth` the outer ones, pdflatex "68.99895pt
/// 68.99895pt 345.0pt 345.0pt" (red on 78d r1, all four 68.99895pt; rev78d finding 5). Witness 2509.19464 A1.F6. Repro
/// captions-floats/subfigure_sets_the_line_width.
#[test]
fn subfigure_sets_the_line_width() {
  assert_clean(
    repro!("captions-floats/subfigure_sets_the_line_width.tex"),
    &[
      (
        "figure",
        "S0.F1",
        "<figure inlist=\"lof\" xml:id=\"S0.F1\"><tags><tag>Figure 1</tag><tag role=\"refnum\">1</tag>\
 <tag role=\"typerefnum\">Figure 1</tag></tags>\
 <figure class=\"ltx_figure_panel\" inlist=\"lof\" placement=\"t\" xml:id=\"S0.F1.sf1\"><tags><tag>\
 <text fontsize=\"90%\" xml:id=\"S0.F1.sf1.1\">(a)</text></tag><tag role=\"refnum\">1a</tag></tags>\
 <p xml:id=\"S0.F1.sf1.2\">96.59958pt 96.59958pt 96.59958pt 96.59958pt</p><toccaption>\
 <tag close=\" \">a</tag>A</toccaption><caption><tag close=\" \">\
 <text fontsize=\"90%\" xml:id=\"S0.F1.sf1.3\">(a)</text></tag>\
 <text fontsize=\"90%\" xml:id=\"S0.F1.sf1.4\">A</text></caption></figure>\
 <p class=\"ltx_figure_panel ltx_minipage\" vattach=\"top\" width=\"69.0pt\" xml:id=\"S0.F1.1\">68.99895pt</p>\
 <toccaption><tag close=\" \">1</tag>Main</toccaption><caption>\
 <tag close=\": \">Figure 1</tag>Main</caption></figure>",
      ),
      (
        "table",
        "S0.T1",
        "<table inlist=\"lot\" xml:id=\"S0.T1\"><tags><tag>Table 1</tag><tag role=\"refnum\">1</tag>\
  <tag role=\"typerefnum\">Table 1</tag></tags><table inlist=\"lot\" xml:id=\"S0.T1.st1\"><tags><tag>\
  <text fontsize=\"90%\" xml:id=\"S0.T1.st1.1\">(a)</text></tag><tag role=\"refnum\">1a</tag></tags>\
  <p xml:id=\"S0.T1.st1.2\">155.24895pt</p><toccaption><tag close=\" \">a</tag>T</toccaption><caption>\
  <tag close=\" \"><text fontsize=\"90%\" xml:id=\"S0.T1.st1.3\">(a)</text></tag>\
  <text fontsize=\"90%\" xml:id=\"S0.T1.st1.4\">T</text></caption></table><toccaption>\
  <tag close=\" \">1</tag>Tab</toccaption><caption><tag close=\": \">Table 1</tag>Tab</caption></table>",
      ),
      (
        "figure",
        "S0.F2",
        "<figure inlist=\"lof\" xml:id=\"S0.F2\"><tags><tag>Figure 2</tag><tag role=\"refnum\">2</tag>\
  <tag role=\"typerefnum\">Figure 2</tag></tags><figure inlist=\"lof\" xml:id=\"S0.F2.sf1\"><tags><tag>\
  <text fontsize=\"90%\" xml:id=\"S0.F2.sf1.1\">(a)</text></tag><tag role=\"refnum\">2a</tag></tags>\
  <p xml:id=\"S0.F2.sf1.2\">68.99895pt 68.99895pt 345.0pt 345.0pt</p><toccaption><tag close=\" \">a</tag>B</toccaption>\
  <caption><tag close=\" \"><text fontsize=\"90%\" xml:id=\"S0.F2.sf1.3\">(a)</text></tag>\
  <text fontsize=\"90%\" xml:id=\"S0.F2.sf1.4\">B</text></caption></figure><toccaption>\
  <tag close=\" \">2</tag>Box</toccaption><caption><tag close=\": \">Figure 2</tag>Box</caption></figure>",
      ),
    ],
  );
}

/// mdpi.cls:1362-1373 sets the class's captions after it loads caption: a bold small label ended by a period, small
/// text — pdflatex (2302.03012's copy) "Figure 1. Fig cap.", "Table 1. Tab cap.". Without a shipped copy the binding
/// stands in for the class. Red (int45b): "Figure 1: " at the normal size. Repro
/// class-bindings/mdpi_captions_end_the_label_with_a_period.
#[test]
fn mdpi_captions_end_the_label_with_a_period() {
  assert_clean(
    repro!("class-bindings/mdpi_captions_end_the_label_with_a_period.tex"),
    &[
      (
        "figure",
        "S1.F1",
        "<figure inlist=\"lof\" xml:id=\"S1.F1\"><tags><tag>Figure 1</tag>\
 <tag role=\"autoref\">Figure\u{00A0}1<text xml:id=\"S1.F1.1\"/></tag><tag role=\"creftype\">fig.</tag>\
 <tag role=\"creftypecap\">Figure</tag><tag role=\"creftypeplural\">figs.</tag>\
 <tag role=\"creftypepluralcap\">Figures</tag><tag role=\"refnum\">1</tag>\
 <tag role=\"typerefnum\">Figure 1</tag></tags><p align=\"center\" xml:id=\"S1.F1.2\">X</p>\
 <toccaption class=\"ltx_centering\"><tag close=\" \">1</tag>Fig cap.</toccaption>\
 <caption class=\"ltx_centering\"><tag close=\". \">\
 <text font=\"bold\" fontsize=\"90%\" xml:id=\"S1.F1.3\">Figure 1</text></tag>\
 <text fontsize=\"90%\" xml:id=\"S1.F1.4\">Fig cap.</text></caption></figure>",
      ),
      (
        "table",
        "S1.T1",
        "<table inlist=\"lot\" labels=\"LABEL:lastpage\" xml:id=\"S1.T1\"><tags><tag>Table 1</tag>\
  <tag role=\"autoref\">Table\u{00A0}1<text xml:id=\"S1.T1.1\"/></tag><tag role=\"creftype\">table</tag>\
  <tag role=\"creftypecap\">Table</tag><tag role=\"creftypeplural\">tables</tag>\
  <tag role=\"creftypepluralcap\">Tables</tag><tag role=\"refnum\">1</tag>\
  <tag role=\"typerefnum\">Table 1</tag></tags><toccaption><tag close=\" \">1</tag>Tab cap.</toccaption>\
  <caption><tag close=\". \"><text font=\"bold\" fontsize=\"90%\" xml:id=\"S1.T1.2\">Table 1</text></tag>\
  <text fontsize=\"90%\" xml:id=\"S1.T1.3\">Tab cap.</text></caption>\
  <p align=\"center\" xml:id=\"S1.T1.4\">Y</p></table>",
      ),
    ],
  );
}

/// A parameter text's closing `#{` appends the `{` to the last delimiter and to the replacement text (tex.web §473-476):
/// `\def\pt(#1)#{…}` delimits #1 by `){`, through `\def`, `\long\def`, `\edef`, `\gdef`, `\protected\xdef`, and
/// `\meaning` shows the brace at both ends. pdflatex "A [2*cos(1),3|T] B [x|L] [x|E] [g(h)|G] [g(h)|X] [a/b(c)|W]",
/// "macro:(#1){->\def \got {#1}\usegot {", and `\def\ub#1#{…}`'s "macro:#1{->[#1]{". Red (int45b; Perl
/// TeX_Macro.pool.ltxml:108-112 the same): 3 "expected {" Errors, "[2*cos(1|,]3)T"; `\ub` (78d r1, rev78d finding 6)
/// "macro:#1->[#1]{". Witness 2610.26441 (tkz-euclide's `\def\tkz@DefPoint[#1](#2)#{`). Repro
/// expansion-primitives/def_hash_brace_extends_the_last_delimiter.
#[test]
fn hash_brace_extends_the_last_delimiter() {
  assert_clean(
    repro!("expansion-primitives/def_hash_brace_extends_the_last_delimiter.tex"),
    &[
      (
        "para",
        "p1",
        "<para xml:id=\"p1\"><p xml:id=\"p1.1\">A [2*cos(1),3—T] B [x—L] [x—E] [g(h)—G] [g(h)—X] [a/b(c)—W]</p>\
 </para>",
      ),
      (
        "para",
        "p2",
        "<para xml:id=\"p2\"><p xml:id=\"p2.1\">\
  <text font=\"typewriter\" xml:id=\"p2.1.1\">macro:(#1){-&gt;\\def \\got {#1}\\usegot {</text> \
  <text font=\"typewriter\" xml:id=\"p2.1.2\">macro:#1{-&gt;[#1]{</text></p></para>",
      ),
    ],
  );
}
