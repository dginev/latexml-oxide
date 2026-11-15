//! Guards for QA14A: a `\parbox` sets `\hsize` and `\linewidth` to its width (latex.ltx:16253 `\@iiiparbox`, :16280
//! `\@parboxrestore`), `\textwidth` and `\columnwidth` staying the enclosing ones; a minipage sets all four (:16312-16313
//! `\@iiiminipage`). Perl's `\parbox` set all four (latex_constructs.pool.ltxml:4746, KNOWN_PERL_ERRORS #1273), so a
//! `\parbox{0.44\textwidth}{\includegraphics[width=0.44\textwidth]{…}}` drew the graphic at 0.44 of the box. Witnesses
//! 1412.0314, 1711.09469.
use latexml::util::test::{assert_element, convert_with};

use super::perfect_kernel_batch46::{error_count, warning_count};

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!("../../../tools/perfect_kernel/repros/", $path))
  };
}

/// A repro and its `(tag, xml:id, element)` rows.
type Case = (
  &'static str,
  &'static [(&'static str, &'static str, &'static str)],
);

/// Each repro: 0 errors, 0 warnings, and each `(tag, xml:id, element)` whole.
/// parbox_keeps_the_page_text_width: every box prints `\textwidth \columnwidth \linewidth \hsize` as pdflatex does —
/// a parbox (A, its graphic 151.8pt), a minipage (B, 66.79pt), `\fbox{\parbox}` (C), a minipage in a parbox (D), a
/// parbox in a minipage (E), a parbox in a `p` cell (F), `\parbox[t][3cm][s]` (G). Red (75531ffc9d6): A, C-G print
/// the box's width four times, A's graphic 66.79274pt.
/// a_parbox_panel_reads_the_page_widths: the panel grid rule reads a parbox's graphics again against the page's
/// `\columnwidth` (`BoxSets::Line`): read against the box's they do not give their widths again, and the float loses
/// its `panelline` (red with the parbox read as a minipage, `BoxSets::All`).
/// a_subcaptionbox_keeps_the_box_hsize: a sub-float sets no `\hsize` (only `\@xfloat` does, latex.ltx:17482), so a
/// `\subcaptionbox` in a parbox or a `p` cell reads the box's. Red (QA14A r1): (a)'s graphic 345.0pt and (b) "H 345.0pt",
/// the parbox no longer setting `\columnwidth`; (c) "H 345.0pt" on 75531ffc9d6 too. Witness 2010.06608 S4.F8.
/// a_subfloat_keeps_the_box_hsize, a_subfigure_keeps_the_box_hsize: subfig's `\subfloat` and subfigure.sty's
/// `\subfigure` (`before_untyped_float`) likewise (review revQA14A p9/p10). Red (QA14A r2, 0b8415564db): (a)'s graphic
/// 172.5pt, (b) "H 345.0pt"; (c) "H 345.0pt" on 75531ffc9d6 too.
#[test]
fn parbox_sets_only_the_line_width() {
  let cases: [Case; 5] = [
    (
      repro!("boxes-groups/parbox_keeps_the_page_text_width.tex"),
      &[
        (
          "inline-block",
          "p1.1.1",
          "<inline-block class=\"ltx_parbox\" vattach=\"middle\" width=\"151.8pt\" xml:id=\"p1.1.1\">\
           <graphics graphic=\"a.png\" options=\"width=151.80084pt,keepaspectratio=true\" xml:id=\"g1\"/>\
           <p xml:id=\"p1.1.1.1\">A 345.0pt 345.0pt 151.80084pt 151.80084pt</p></inline-block>",
        ),
        (
          "inline-block",
          "p2.1.1",
          "<inline-block class=\"ltx_minipage\" vattach=\"middle\" width=\"151.8pt\" xml:id=\"p2.1.1\">\
           <graphics graphic=\"b.png\" options=\"width=66.79274pt,keepaspectratio=true\" xml:id=\"g2\"/>\
           <p xml:id=\"p2.1.1.1\">B 151.80084pt 151.80084pt 151.80084pt 151.80084pt</p></inline-block>",
        ),
        (
          "p",
          "p3.1.1.1",
          "<p xml:id=\"p3.1.1.1\">C 345.0pt 345.0pt 172.5pt 172.5pt</p>",
        ),
        (
          "p",
          "p4.1.1.1.1.1",
          "<p xml:id=\"p4.1.1.1.1.1\">D 172.5pt 172.5pt 172.5pt 172.5pt</p>",
        ),
        (
          "p",
          "p5.1.1.1.1.1",
          "<p xml:id=\"p5.1.1.1.1.1\">E 172.5pt 172.5pt 86.25pt 86.25pt</p>",
        ),
        (
          "p",
          "p6.1.1.1.1.1.1.1",
          "<p xml:id=\"p6.1.1.1.1.1.1.1\">F 345.0pt 345.0pt 172.5pt 172.5pt</p>",
        ),
        (
          "p",
          "p7.1.1.1",
          "<p xml:id=\"p7.1.1.1\">G 345.0pt 345.0pt 103.50105pt 103.50105pt</p>",
        ),
      ],
    ),
    (
      repro!("captions-floats/a_parbox_panel_reads_the_page_widths.tex"),
      &[
        (
          "figure",
          "S0.F1",
          "<figure class=\"ltx_panels_gallery\" inlist=\"lof\" panelcols=\"2\" panelfill=\"0.96\" panelline=\"1\" \
           panelrows=\"1\" xml:id=\"S0.F1\"><tags><tag>Fig. 1</tag><tag role=\"refnum\">1</tag>\
           <tag role=\"typerefnum\">Fig. 1</tag></tags>\
           <block class=\"ltx_figure_panel ltx_parbox\" panelwidth=\"0.48\" vattach=\"middle\" width=\"165.6pt\" \
           xml:id=\"S0.F1.1\"><graphics graphic=\"a.png\" options=\"width=155.24895pt,keepaspectratio=true\" \
           xml:id=\"S0.F1.g1\"/></block>\
           <block class=\"ltx_figure_panel ltx_parbox\" panelwidth=\"0.48\" vattach=\"middle\" width=\"165.6pt\" \
           xml:id=\"S0.F1.2\"><graphics graphic=\"b.png\" options=\"width=155.24895pt,keepaspectratio=true\" \
           xml:id=\"S0.F1.g2\"/></block>\
           <toccaption><tag close=\" \">1</tag>Two.</toccaption><caption><tag close=\". \">Fig. 1</tag>Two.</caption>\
           </figure>",
        ),
        (
          "figure",
          "S0.F2",
          "<figure class=\"ltx_panels_gallery\" inlist=\"lof\" panelcols=\"3\" panelfill=\"0.9\" panelline=\"2.048\" \
           panelrows=\"1\" xml:id=\"S0.F2\"><tags><tag>Fig. 2</tag><tag role=\"refnum\">2</tag>\
           <tag role=\"typerefnum\">Fig. 2</tag></tags>\
           <block class=\"ltx_figure_panel ltx_parbox\" panelwidth=\"0.3\" vattach=\"middle\" width=\"103.5pt\" \
           xml:id=\"S0.F2.1\"><graphics graphic=\"a.png\" options=\"width=103.50105pt,keepaspectratio=true\" \
           xml:id=\"S0.F2.g1\"/></block>\
           <block class=\"ltx_figure_panel ltx_parbox\" panelwidth=\"0.3\" vattach=\"middle\" width=\"103.5pt\" \
           xml:id=\"S0.F2.2\"><graphics graphic=\"b.png\" options=\"width=103.50105pt,keepaspectratio=true\" \
           xml:id=\"S0.F2.g2\"/></block>\
           <block class=\"ltx_figure_panel ltx_parbox\" panelwidth=\"0.3\" vattach=\"middle\" width=\"103.5pt\" \
           xml:id=\"S0.F2.3\"><graphics graphic=\"c.png\" options=\"width=103.50105pt,keepaspectratio=true\" \
           xml:id=\"S0.F2.g3\"/></block>\
           <toccaption><tag close=\" \">2</tag>Three.</toccaption><caption><tag close=\". \">Fig. 2</tag>Three.</caption>\
           </figure>",
        ),
      ],
    ),
    (
      repro!("captions-floats/a_subcaptionbox_keeps_the_box_hsize.tex"),
      &[
        (
          "figure",
          "S0.F1",
          "<figure class=\"ltx_panels_gallery\" inlist=\"lof\" panelcols=\"2\" panelfill=\"0.77\" panelline=\"1\" \
           panelrows=\"1\" xml:id=\"S0.F1\"><tags><tag>Figure 1</tag><tag role=\"refnum\">1</tag>\
           <tag role=\"typerefnum\">Figure 1</tag></tags>\
           <figure class=\"ltx_figure_panel ltx_parbox\" inlist=\"lof\" panelwidth=\"0.23\" vattach=\"bottom\" \
           width=\"79.3pt\" xml:id=\"S0.F1.sf1\"><tags><tag><text fontsize=\"90%\" xml:id=\"S0.F1.sf1.1\">(a)</text>\
           </tag><tag role=\"refnum\">1a</tag></tags>\
           <graphics graphic=\"a.png\" options=\"width=79.34853pt,keepaspectratio=true\" xml:id=\"S0.F1.sf1.g1\"/>\
           <toccaption><tag close=\" \">a</tag>a</toccaption><caption><tag close=\" \">\
           <text fontsize=\"90%\" xml:id=\"S0.F1.sf1.2\">(a)</text></tag>\
           <text fontsize=\"90%\" xml:id=\"S0.F1.sf1.3\">a</text></caption></figure>\
           <figure class=\"ltx_figure_panel ltx_parbox\" inlist=\"lof\" panelwidth=\"0.54\" vattach=\"bottom\" \
           width=\"186.3pt\" xml:id=\"S0.F1.sf2\"><tags><tag><text fontsize=\"90%\" xml:id=\"S0.F1.sf2.1\">(b)</text>\
           </tag><tag role=\"refnum\">1b</tag></tags>\
           <p xml:id=\"S0.F1.sf2.2\">H 186.29768pt L 186.29768pt</p>\
           <toccaption><tag close=\" \">b</tag>b</toccaption><caption><tag close=\" \">\
           <text fontsize=\"90%\" xml:id=\"S0.F1.sf2.3\">(b)</text></tag>\
           <text fontsize=\"90%\" xml:id=\"S0.F1.sf2.4\">b</text></caption></figure>\
           <toccaption><tag close=\" \">1</tag>Boxes.</toccaption><caption><tag close=\": \">Figure 1</tag>Boxes.</caption>\
           </figure>",
        ),
        (
          "p",
          "S0.F2.sf1.2",
          "<p xml:id=\"S0.F2.sf1.2\">H 103.50105pt L 103.50105pt</p>",
        ),
        (
          "p",
          "S0.F2.sf2.2",
          "<p xml:id=\"S0.F2.sf2.2\">H 345.0pt L 345.0pt</p>",
        ),
      ],
    ),
    (
      repro!("captions-floats/a_subfloat_keeps_the_box_hsize.tex"),
      &[
        (
          "figure",
          "S0.F1.sf1",
          "<figure align=\"center\" class=\"ltx_figure_panel\" xml:id=\"S0.F1.sf1\"><tags><tag>(a)</tag>\
           <tag role=\"refnum\">1a</tag></tags>\
           <graphics graphic=\"a.png\" options=\"width=82.79926pt,keepaspectratio=true\" xml:id=\"S0.F1.sf1.g1\"/>\
           <toccaption><tag close=\" \">a</tag>a</toccaption><caption><tag close=\" \">(a)</tag>a</caption></figure>",
        ),
        (
          "p",
          "S0.F1.sf2.1",
          "<p xml:id=\"S0.F1.sf2.1\">H 165.59853pt L 165.59853pt</p>",
        ),
        (
          "p",
          "S0.F1.sf3.1",
          "<p xml:id=\"S0.F1.sf3.1\">H 103.50105pt L 103.50105pt</p>",
        ),
        (
          "p",
          "S0.F2.sf1.1",
          "<p xml:id=\"S0.F2.sf1.1\">H 345.0pt L 345.0pt</p>",
        ),
      ],
    ),
    (
      repro!("captions-floats/a_subfigure_keeps_the_box_hsize.tex"),
      &[
        (
          "figure",
          "S0.F1.sf1",
          "<figure align=\"center\" class=\"ltx_figure_panel\" xml:id=\"S0.F1.sf1\"><tags><tag>\
           <text fontsize=\"80%\" xml:id=\"S0.F1.sf1.1\">(a)</text></tag><tag role=\"refnum\">1(a)</tag></tags>\
           <graphics graphic=\"a.png\" options=\"width=82.79926pt,keepaspectratio=true\" xml:id=\"S0.F1.sf1.g1\"/>\
           <toccaption><tag close=\" \">(a)</tag>a</toccaption><caption><tag close=\" \">\
           <text fontsize=\"80%\" xml:id=\"S0.F1.sf1.2\">(a)</text></tag>\
           <text fontsize=\"80%\" xml:id=\"S0.F1.sf1.3\">a</text></caption></figure>",
        ),
        (
          "p",
          "S0.F1.sf2.2",
          "<p xml:id=\"S0.F1.sf2.2\">H 165.59853pt L 165.59853pt</p>",
        ),
        (
          "p",
          "S0.F1.sf3.2",
          "<p xml:id=\"S0.F1.sf3.2\">H 103.50105pt L 103.50105pt</p>",
        ),
        (
          "p",
          "S0.F2.sf1.2",
          "<p xml:id=\"S0.F2.sf1.2\">H 345.0pt L 345.0pt</p>",
        ),
      ],
    ),
  ];
  for (tex, expected) in cases {
    let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
    assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
    for (tag, id, element) in expected {
      assert_element(&xml, tag, &[&format!("xml:id=\"{id}\"")], element);
    }
  }
}
