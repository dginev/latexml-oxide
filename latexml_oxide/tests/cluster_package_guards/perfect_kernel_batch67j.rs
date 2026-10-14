//! Red/green guards for batch 67j: native bindings that lacked their class's own user macros (run 349 "undefined"
//! clusters: espcrc `\readRCS`/`\runtitle`/`\runauthor`, svjour epjCONF `\session-title`, imsart `\endbibitem`,
//! siamltex `\slugger`, amsppt `\eightpoint`/`\tenpoint` and AMS-TeX `\goth`, tcilatex `\TCItag`).
use super::perfect_kernel_batch46::{error_count, warning_count};

fn body(xml: &str) -> &str { xml.split("<document").nth(1).unwrap_or_default() }

/// The XML with every whitespace run collapsed to one space.
fn squash(xml: &str) -> String { xml.split_whitespace().collect::<Vec<_>>().join(" ") }

/// espcrc1.sty/espcrc2.sty v1.2 (2004; espcrc1.sty:27-31, :156-159): `\readRCS $Id: … $` sets the file stamp, and
/// the running heads are stored, never typeset (hep-ex0506002, astro-ph0610654, cond-mat0610259, hep-ph0506127,
/// hep-ex0610011).
#[test]
fn espcrc_readrcs_and_running_heads() {
  let (log, xml) = latexml::util::test::convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/class-bindings/espcrc_readrcs_and_running_heads.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let doc = body(&xml);
  assert!(!doc.contains("<Math"), "{xml}");
  assert!(doc.contains("<title>Real title</title>"), "{xml}");
  assert!(!doc.contains("Short title"), "{xml}");
  assert_eq!(doc.matches("A. Author").count(), 1, "{xml}");
  assert!(
    doc.contains("<p xml:id=\"abstract1.1\">The abstract.</p>"),
    "{xml}"
  );
  assert!(doc.contains("<p xml:id=\"p1.1\">Body text.</p>"), "{xml}");
}

/// `\readRCS` splits the RCS `$Id$` keyword as espcrc1.sty:27-31 does: `\filename` the RCS file, `\fileversion` the
/// revision, `\filedate` the date.
#[test]
fn espcrc_readrcs_sets_the_file_stamp() {
  let tex = "\\documentclass{article}\n\\usepackage{espcrc2}\n\\readRCS\n\
             $Id: espcrc2.tex,v 1.2 2004/02/24 11:22:11 spepping Exp $\n\\begin{document}\n\
             [\\filename][\\fileversion][\\filedate]\n\\end{document}\n";
  let (log, xml) = latexml::util::test::convert_with(tex, Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    xml.contains("<p xml:id=\"p1.1\">[espcrc2.tex,v][1.2][2004/02/24]</p>"),
    "{xml}"
  );
}

/// svjour.cls:78-81 inputs `sv<option>.clo`; svepjCONF.clo:233-235 adds `\session-title`, a running-head value
/// (1010.2389, 1010.3788, 1202.0149).
#[test]
fn svjour_epjconf_session_title_is_stored() {
  let (log, xml) = latexml::util::test::convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/class-bindings/svjour_epjconf_session_title_is_stored.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let doc = body(&xml);
  assert!(!doc.contains("conference"), "{xml}");
  assert!(!doc.contains("-title"), "{xml}");
  assert!(doc.contains("<title>Real title</title>"), "{xml}");
  assert!(
    doc.contains("<p xml:id=\"abstract1.1\">The abstract.</p>"),
    "{xml}"
  );
  assert!(
    doc.contains("<p xml:id=\"S1.p1.1\">Body text.</p>"),
    "{xml}"
  );
}

/// imsart.sty:2514 `\let\endbibitem\relax`; :2585-2769 every structured entry drops its optional key list
/// (`\begin{barticle}[author]`), `{binproceedings}` among them (0906.4329, 1010.4313, 1202.6182).
#[test]
fn imsart_structured_bibitem_keys_and_endbibitem() {
  let (log, xml) = latexml::util::test::convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/class-bindings/imsart_structured_bibitem_keys_and_endbibitem.tex"
    ),
    Some("ar5iv.sty"),
  );
  // The one Warning: imsart.sty is not in TeX Live.
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(log.contains("Warning:missing_file:imsart"), "{log}");
  assert!(!xml.contains("[author]"), "{xml}");
  let flat = squash(&xml);
  assert!(
    flat.contains("<bibblock> Author, A. Title. Journal 1 1–2. </bibblock>"),
    "{xml}"
  );
  assert!(
    flat.contains("<bibblock> Writer, B. Talk. </bibblock>"),
    "{xml}"
  );
}

/// siamltex704.cls:189-209 (siamltex1213.cls:184-204): `\slugger` stores the title page's slug line (math0610498,
/// 1410.0201, 1410.1612).
#[test]
fn siamltex_slugger_is_stored() {
  let (log, xml) = latexml::util::test::convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/class-bindings/siamltex_slugger_is_stored.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let doc = body(&xml);
  for slug in ["sisc", "xxxx", "Society", "Vol."] {
    assert!(!doc.contains(slug), "{slug}: {xml}");
  }
  assert!(doc.contains("<title>Real title</title>"), "{xml}");
  assert!(doc.contains("<p xml:id=\"p1.1\">Body text.</p>"), "{xml}");
}

/// amsppt.sty:153-214 `\tenpoint` / `\eightpoint` (10pt / 8pt roman) and amstex.tex:1509 `\let\goth\frak`
/// (hep-th9306127, adap-org9410001, solv-int9306003; hep-th9306053, alg-geom9410018).
#[test]
fn amsppt_point_sizes_and_goth() {
  let (log, xml) = latexml::util::test::convert_with(
    include_str!("../../../tools/perfect_kernel/repros/amstex/amsppt_point_sizes_and_goth.tex"),
    None,
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    xml.contains("<p><text fontsize=\"80%\">Small print.</text></p>"),
    "{xml}"
  );
  assert!(
    xml.contains("<p>Normal print. The ideal <Math mode=\"inline\" tex=\"\\mathfrak{m}\""),
    "{xml}"
  );
  assert!(
    xml.contains("<XMTok font=\"fraktur\" role=\"UNKNOWN\">m</XMTok>"),
    "{xml}"
  );
}

/// tcilatex.tex 3.0+ (:676-683 in 3.0, :776-783 in 5.0) `\TCItag{n}` / `\TCItag*{text}` tag an eqnarray row; the
/// binding is the 2.5 copy, which had none (nlin0106009, hep-th0210191, hep-ph0210025).
#[test]
fn tcilatex_tcitag_tags_eqnarray_rows() {
  let (log, xml) = latexml::util::test::convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/package-bindings/tcilatex_tcitag_tags_eqnarray_rows.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let tags: Vec<&str> = xml
    .split("<equation ")
    .skip(1)
    .map(|eq| {
      eq.split("<tag>")
        .nth(1)
        .and_then(|t| t.split("</tag>").next())
        .unwrap_or_default()
    })
    .collect();
  assert_eq!(tags, ["(3)", "(4)", "A"], "{xml}");
}

/// imsart.sty:464-466 `\setattribute`/`\getattribute` store and read a value (1306.0113, 1706.08519); :640-645
/// `\thanksmark{m1}` in an `\affiliation` is the mark the authors' `\thanksref` link, not printed (1202.6182,
/// 1306.1438).
#[test]
fn imsart_setattribute_and_thanksmark() {
  let tex = "\\documentclass[aoas]{imsart}\n\\setattribute{journal}{name}{The Journal}\n\\begin{document}\n\
             \\begin{frontmatter}\\title{T}\n\\author{\\fnms{A.} \\snm{Author}\\thanksref{m1}}\n\
             \\affiliation{University\\thanksmark{m1}}\n\\end{frontmatter}\n\
             Name: [\\getattribute{journal}{name}].\n\\end{document}\n";
  let (log, xml) = latexml::util::test::convert_with(tex, Some("ar5iv.sty"));
  // The one Warning: imsart.sty is not in TeX Live.
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  // The mark is not printed: the affiliation is the address alone.
  assert!(
    xml.contains(" role=\"affiliation\">University</contact>"),
    "{xml}"
  );
  assert!(xml.contains(">Name: [The Journal].</p>"), "{xml}");
}

/// tcilatex.tex 4.0 :497-507 (5.0 :521-533): `\U{code}` / `\UNICODE{code}` stand for a character the export could
/// not name, a filled 0.1in square (0802.0560).
#[test]
fn tcilatex_unicode_placeholders() {
  let tex = "\\documentclass{article}\n\\input{tcilatex}\n\\begin{document}\nA \\U{00e9} B \\UNICODE{2234} C.\n\
             \\end{document}\n";
  let (log, xml) = latexml::util::test::convert_with(tex, Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let square = "<rule height=\"7.2pt\" width=\"7.2pt\"/>";
  assert!(
    xml.contains(&format!(">A {square} B {square} C.</p>")),
    "{xml}"
  );
}
