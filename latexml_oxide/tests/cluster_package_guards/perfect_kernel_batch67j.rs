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
  // In math only the size changes: the math families are resized, the letters stay math italic (`\textfont\@ne
  // \eighti`; 1306.2523, math0210423), in a display and in inline math in `\eightpoint` text.
  let xml = squash(&xml);
  assert!(
    xml.contains(
      "<XMApp> <XMTok fontsize=\"80%\" meaning=\"equals\" role=\"RELOP\">=</XMTok> \
       <XMApp> <XMTok fontsize=\"80%\" meaning=\"plus\" role=\"ADDOP\">+</XMTok> \
       <XMTok font=\"italic\" fontsize=\"80%\" role=\"UNKNOWN\">x</XMTok> \
       <XMTok font=\"italic\" fontsize=\"80%\" role=\"UNKNOWN\">y</XMTok> </XMApp> \
       <XMTok font=\"italic\" fontsize=\"80%\" role=\"UNKNOWN\">z</XMTok> </XMApp>"
    ),
    "{xml}"
  );
  assert!(
    xml.contains(
      "<XMTok meaning=\"plus\" role=\"ADDOP\">+</XMTok> <XMTok font=\"italic\" role=\"UNKNOWN\">a</XMTok> \
       <XMTok font=\"italic\" role=\"UNKNOWN\">b</XMTok>"
    ),
    "{xml}"
  );
  assert!(
    !xml.contains("mathvariant") && !xml.contains("font=\"upright\""),
    "{xml}"
  );
}

/// tcilatex.tex 3.0+ (:676-683 in 3.0, :776-783 in 5.0) `\TCItag{n}` / `\TCItag*{text}` tag an eqnarray row; the
/// binding is the 2.5 copy, which had none (nlin0106009, hep-th0210191, hep-ph0210025). A row whose only content is
/// its tag (Scientific Word's `… \nonumber \\ && \TCItag{27}`, inner and last; nlin0106009's 11 rows) numbers the
/// equation above it, where Perl prunes it with its tag (KNOWN_PERL_ERRORS #682, OXIDIZED_DESIGN_DIVERGENCES #591).
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
  assert_eq!(tags, ["(3)", "(4)", "A", "(27)", "(28)"], "{xml}");
  let xml = squash(&xml);
  for (id, n, math) in [("S0.Ex5", 27, "u=v,"), ("S0.Ex7", 28, "w=t,")] {
    let equation = format!(
      "<equation xml:id=\"{id}\"> <tags> <tag>({n})</tag> <tag role=\"refnum\">{n}</tag> </tags> <MathFork> \
       <Math tex=\"\\displaystyle {math}\""
    );
    assert!(xml.contains(&equation), "{equation}\n{xml}");
  }
  assert!(!xml.contains("_tagged"), "{xml}");
}

/// amsmath's own `\tag` on a row that holds nothing else (`\notag \\ \tag{28}` in `align`, an inner row; `\notag \\
/// \tag{29}` in `gather`, the last): pdflatex prints the tag on that row, where Perl drops the row with its tag
/// (KNOWN_PERL_ERRORS #682, OXIDIZED_DESIGN_DIVERGENCES #591; nlin0106009). An auto-numbered row keeps its number.
#[test]
fn ams_tag_only_row_keeps_number() {
  let (log, xml) = latexml::util::test::convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/alignment/ams_tag_only_row_keeps_number.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let xml = squash(&xml);
  for (id, n) in [("S0.Ex2", 28), ("S0.Ex4", 29)] {
    let equation = format!(
      "<equation xml:id=\"{id}\"> <tags> <tag>({n})</tag> <tag role=\"refnum\">{n}</tag> </tags> </equation>"
    );
    assert!(xml.contains(&equation), "{equation}\n{xml}");
  }
  assert!(
    xml.contains(
      "<equation xml:id=\"S0.E1\"> <tags> <tag>(1)</tag> <tag role=\"refnum\">1</tag> </tags>"
    ),
    "{xml}"
  );
  assert_eq!(xml.matches("<equation ").count(), 5, "{xml}");
  assert!(!xml.contains("_tagged"), "{xml}");
}

/// A trailing `\\` leaves an auto-numbered empty last row: still pruned (Perl Alignment.pm normalize_prune_rows), in
/// `eqnarray` (its number spent, as LaTeX's) and `align`; only a row its author tagged is kept.
#[test]
fn trailing_row_without_tag_is_still_pruned() {
  let tex = "\\documentclass{article}\n\\usepackage{amsmath}\n\\begin{document}\n\
             \\begin{eqnarray}\na &=& b \\\\\n\\end{eqnarray}\n\
             \\begin{align}\nc &= d \\\\\n\\end{align}\n\\end{document}\n";
  let (log, xml) = latexml::util::test::convert_with(tex, Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let xml = squash(&xml);
  assert_eq!(xml.matches("<equation ").count(), 2, "{xml}");
  assert!(
    xml.contains("<equation xml:id=\"S0.E1\"> <tags> <tag>(1)</tag>"),
    "{xml}"
  );
  assert!(
    xml.contains("<equation xml:id=\"S0.E3\"> <tags> <tag>(3)</tag>"),
    "{xml}"
  );
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
