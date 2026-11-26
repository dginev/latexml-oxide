//! Guards for QA14 K2 (Bruce Q&A QA14, the length census's LIST/ADJW rows): a list sets its registers as latex.ltx's
//! `\list` does (:15848-15869) — `\@list<depth>` and the setup body give the margins, by which `\linewidth` narrows
//! and `\@totalleftmargin` grows — with the standard classes' `\leftmargini`..`\leftmarginvi` and `\labelsep`
//! (article.cls:322-339) in ems of the class size; changepage's `{adjustwidth}` is such a list; a box in a list sets
//! its own line. Perl's registers are 0pt and its `\list` sets none (latex_constructs.pool.ltxml:1649, :1690-1702), so
//! a `width=\linewidth` graphic in a list or an MDPI `{adjustwidth}{-\extralength}{0cm}` figure took the text's width.
//! OXIDIZED_DESIGN_DIVERGENCES #29.
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

/// Each repro: 0 errors, 0 warnings, and each `(tag, xml:id, element)` whole, the lengths as pdflatex prints them.
/// list_narrows_the_line_width: every list-shaped environment of the kernel and article.cls at its depth. Red
/// (INT081AA): every line 345.0pt, every margin 0.0pt.
/// adjustwidth_sets_the_line_width: changepage's list, a negative margin widening the line, a list and a figure in it,
/// `{adjustwidth*}`, a graphic's width. Red (INT081AA): 345.0pt and 0.0pt throughout.
/// box_in_a_list_sets_its_own_line: a footnote, a float, a minipage (its list first-level), a `p` cell start at their
/// own edge; a trivlist (center, theorem, proof) clears the margins. Red (QA14K2 round 1, before the box resets): F and
/// G "319.99997pt", MI "22.0pt" (the second level's margin).
/// enumitem_stated_margins_set_the_line: a stated `leftmargin`/`rightmargin` key. Red (INT081AA): B "345.0pt 0.0pt 0.0pt".
/// standard_class_list_margins_follow_the_size: at 12pt the ems are cmr12's. Red (QA14K2 round 1): A "25.00003pt 5.0pt
/// 20.00003pt", at the binding's 10pt em.
/// adjustwidth_panels_share_the_line: panels in an `{adjustwidth}` are a row of its line. Red (QA14K2 round 1):
/// `panelwidth="0.349"`, `panelrows="2"`. Witnesses 2405.20768, 2408.06972.
/// label_opening_glue_nets_against_labelsep: a label's leading glue nets against `\labelsep` on every label path, a
/// trailing one stays, a `\usebox`ed register keeps its own. Red (QA14K2 eb92bd5ebc9): `<tag> x</tag>` (U+2002),
/// `<tag> •</tag>`, the typerefnum's ` k`. Witnesses 2605.07451, 1508.03599.
/// enumitem_wide_clears_the_left_margin: `wide`/`fullwidth` set `\leftmargin\z@`, keys in order. Red (QA14K2
/// eb92bd5ebc9): A-D, F "319.99997pt 25.00003pt"; G, H (a `leftmargin` before `wide`) "316.54726pt 28.45274pt".
/// note_and_cell_paragraphs_are_unspaced_in_a_list: a footnote and a `p` cell reset `\parskip`. Red (QA14K2
/// eb92bd5ebc9): "4.0pt plus 2.0pt minus 1.0pt" (the list's `\parsep`).
#[test]
fn list_registers_set_the_line_width() {
  let cases: [Case; 9] = [
    (repro!("list-structure/list_narrows_the_line_width.tex"), &[
      (
        "p",
        "p1.1",
        "<p xml:id=\"p1.1\">A 345.0pt 25.00003pt 0.0pt 0.0pt 20.00003pt 0.0pt 5.0pt</p>",
      ),
      (
        "p",
        "S0.I1.i1.p1.1",
        "<p xml:id=\"S0.I1.i1.p1.1\">B1 319.99997pt 25.00003pt 0.0pt 25.00003pt 20.00003pt 0.0pt</p>",
      ),
      (
        "p",
        "S0.I1.i1.I1.i1.p1.1",
        "<p xml:id=\"S0.I1.i1.I1.i1.p1.1\">B2 297.99997pt 22.0pt 0.0pt 47.00003pt 17.0pt 0.0pt</p>",
      ),
      (
        "p",
        "S0.I1.i1.I1.i1.I1.i1.p1.1",
        "<p xml:id=\"S0.I1.i1.I1.i1.I1.i1.p1.1\">B3 279.3pt 18.69997pt 0.0pt 65.7pt 13.69997pt 0.0pt</p>",
      ),
      (
        "p",
        "S0.I1.i1.I1.i1.I1.i1.I1.i1.p1.1",
        "<p xml:id=\"S0.I1.i1.I1.i1.I1.i1.I1.i1.p1.1\">B4 262.30002pt 16.99998pt 0.0pt 82.69998pt 11.99998pt 0.0pt</p>",
      ),
      (
        "p",
        "S0.I2.i1.p1.1",
        "<p xml:id=\"S0.I2.i1.p1.1\">E1 319.99997pt 25.00003pt 0.0pt 25.00003pt 20.00003pt 0.0pt</p>",
      ),
      (
        "p",
        "S0.I2.i1.I2.i1.p1.1",
        "<p xml:id=\"S0.I2.i1.I2.i1.p1.1\">E2 297.99997pt 22.0pt 0.0pt 47.00003pt 17.0pt 0.0pt</p>",
      ),
      (
        "p",
        "S0.I3.ix1.p1.1",
        "<p xml:id=\"S0.I3.ix1.p1.1\">D 319.99997pt 25.00003pt 0.0pt 25.00003pt 0.0pt -25.00003pt</p>",
      ),
      (
        "p",
        "p1.2.1",
        "<p xml:id=\"p1.2.1\">Q 294.99994pt 25.00003pt 25.00003pt 25.00003pt 20.00003pt 0.0pt</p>",
      ),
      (
        "p",
        "p1.3.1",
        "<p xml:id=\"p1.3.1\">N 294.99994pt 25.00003pt 25.00003pt 25.00003pt 20.00003pt 15.00002pt</p>",
      ),
      (
        "p",
        "p1.4.1",
        "<p xml:id=\"p1.4.1\">V 279.99992pt 40.00005pt 25.00003pt 40.00005pt 20.00003pt -15.00002pt</p>",
      ),
      (
        "p",
        "S0.I4.i1.p1.1",
        "<p xml:id=\"S0.I4.i1.p1.1\">L 259.64175pt 28.45274pt 56.9055pt 28.45274pt 20.00003pt 0.0pt</p>",
      ),
      (
        "p",
        "p1.6",
        "<p xml:id=\"p1.6\">Z 345.0pt 25.00003pt 0.0pt 0.0pt 20.00003pt 0.0pt</p>",
      ),
    ]),
    (
      repro!("block-model/adjustwidth_sets_the_line_width.tex"),
      &[
        (
          "p",
          "p1.1",
          "<p xml:id=\"p1.1\">A 259.64175pt 28.45274pt 56.9055pt 28.45274pt</p>",
        ),
        (
          "p",
          "p1.2",
          "<p xml:id=\"p1.2\">B 373.45274pt -28.45274pt 0.0pt -28.45274pt</p>",
        ),
        (
          "p",
          "S0.I1.i1.p1.1",
          "<p xml:id=\"S0.I1.i1.p1.1\">C 351.45274pt 22.0pt 0.0pt -6.45274pt</p>",
        ),
        (
          "p",
          "fig1.1",
          "<p class=\"ltx_figure_panel\" xml:id=\"fig1.1\">F 345.0pt -28.45274pt 0.0pt 0.0pt</p>",
        ),
        (
          "p",
          "p2.1",
          "<p xml:id=\"p2.1\">G 373.45274pt 0.0pt -28.45274pt 0.0pt</p>",
        ),
        (
          "p",
          "fig2.1",
          "<p class=\"ltx_figure_panel\" xml:id=\"fig2.1\">H 373.45274pt -28.45274pt 0.0pt -28.45274pt</p>",
        ),
        (
          "graphics",
          "g1",
          "<graphics class=\"ltx_figure_panel\" graphic=\"f.png\" options=\"width=345.0pt,keepaspectratio=true\" panelwidth=\"1\" xml:id=\"g1\"/>",
        ),
        (
          "graphics",
          "g2",
          "<graphics class=\"ltx_figure_panel\" graphic=\"h.png\" options=\"width=373.45274pt,keepaspectratio=true\" panelwidth=\"1\" xml:id=\"g2\"/>",
        ),
      ],
    ),
    (
      repro!("list-structure/box_in_a_list_sets_its_own_line.tex"),
      &[
        (
          "note",
          "footnote1",
          "<note mark=\"1\" role=\"footnote\" xml:id=\"footnote1\"><tags><tag>1</tag>\
           <tag role=\"refnum\">1</tag><tag role=\"typerefnum\">footnote 1</tag>\
           </tags>F 345.0pt 25.00003pt 0.0pt 20.00003pt</note>",
        ),
        (
          "p",
          "S0.I1.i2.fig1.1",
          "<p xml:id=\"S0.I1.i2.fig1.1\">G 345.0pt 25.00003pt 0.0pt 20.00003pt</p>",
        ),
        (
          "p",
          "S0.I1.i3.p1.1.1.1",
          "<p xml:id=\"S0.I1.i3.p1.1.1.1\">M 159.99998pt 25.00003pt 0.0pt 20.00003pt</p>",
        ),
        (
          "p",
          "S0.I1.i3.I1.i1.p1.1",
          "<p xml:id=\"S0.I1.i3.I1.i1.p1.1\">MI 134.99995pt 25.00003pt 25.00003pt 20.00003pt</p>",
        ),
        (
          "p",
          "S0.I1.i4.p1.1.1.1.1.1",
          "<p xml:id=\"S0.I1.i4.p1.1.1.1.1.1\">P 85.35826pt 25.00003pt 0.0pt 20.00003pt</p>",
        ),
        (
          "td",
          "S0.I1.i4.p1.1.1.2",
          "<td align=\"left\" xml:id=\"S0.I1.i4.p1.1.1.2\">L 319.99997pt 25.00003pt 25.00003pt 20.00003pt</td>",
        ),
        (
          "text",
          "S0.I1.i4.p1.2.1",
          "<text cssstyle=\"padding:3.0pt\" framecolor=\"#000000\" framed=\"rectangle\" xml:id=\"S0.I1.i4.p1.2.1\">X 319.99997pt 25.00003pt 25.00003pt 20.00003pt</text>",
        ),
        (
          "p",
          "S0.I1.i5.p1.1",
          "<p align=\"center\" xml:id=\"S0.I1.i5.p1.1\">C 319.99997pt 0.0pt 25.00003pt 0.0pt</p>",
        ),
        (
          "text",
          "Thmthm1.p1.1.1",
          "<text font=\"italic\" xml:id=\"Thmthm1.p1.1.1\">T 319.99997pt 0.0pt 25.00003pt 0.0pt</text>",
        ),
        (
          "text",
          "S0.I1.i5.p2.1.1",
          "<text xml:id=\"S0.I1.i5.p2.1.1\">R 319.99997pt 0.0pt 25.00003pt 0.0pt∎</text>",
        ),
      ],
    ),
    (
      repro!("list-structure/enumitem_stated_margins_set_the_line.tex"),
      &[
        (
          "p",
          "S0.I1.i1.p1.1",
          "<p xml:id=\"S0.I1.i1.p1.1\">A 345.0pt 0.0pt 0.0pt</p>",
        ),
        (
          "p",
          "S0.I2.i1.p1.1",
          "<p xml:id=\"S0.I2.i1.p1.1\">B 288.09451pt 28.45274pt 28.45274pt</p>",
        ),
        (
          "p",
          "S0.I3.i1.p1.1",
          "<p xml:id=\"S0.I3.i1.p1.1\">C 319.99997pt 25.00003pt 0.0pt</p>",
        ),
      ],
    ),
    (
      repro!("class-bindings/standard_class_list_margins_follow_the_size.tex"),
      &[
        (
          "p",
          "p1.1",
          "<p xml:id=\"p1.1\">A 29.3747pt 5.87494pt 23.49976pt</p>",
        ),
        (
          "p",
          "Ch0.S0.I1.i1.p1.1",
          "<p xml:id=\"Ch0.S0.I1.i1.p1.1\">B1 360.6253pt 29.3747pt 29.3747pt</p>",
        ),
        (
          "p",
          "Ch0.S0.I1.i1.I1.i1.p1.1",
          "<p xml:id=\"Ch0.S0.I1.i1.I1.i1.p1.1\">B2 334.77562pt 25.84969pt 55.22438pt</p>",
        ),
      ],
    ),
    (
      repro!("captions-floats/adjustwidth_panels_share_the_line.tex"),
      &[(
        "figure",
        "S0.F1",
        "<figure class=\"ltx_panels_gallery\" inlist=\"lof\" panelcols=\"3\" panelfill=\"0.9\" panelline=\"1.165\" panelrows=\"1\" placement=\"h\" xml:id=\"S0.F1\">\
           <tags><tag>Figure 1</tag><tag role=\"refnum\">1</tag><tag role=\"typerefnum\">Figure 1</tag></tags>\
           <figure align=\"center\" class=\"ltx_figure_panel\" inlist=\"lof\" panelwidth=\"0.3\" xml:id=\"S0.F1.sf1\">\
           <tags><tag><text fontsize=\"90%\" xml:id=\"S0.F1.sf1.1\">(a)</text></tag>\
           <tag role=\"refnum\">1a</tag></tags>\
           <graphics graphic=\"a.png\" options=\"width=120.57286pt,keepaspectratio=true\" xml:id=\"S0.F1.sf1.g1\"/>\
           <toccaption><tag close=\" \">a</tag>A</toccaption><caption><tag close=\" \">\
           <text fontsize=\"90%\" xml:id=\"S0.F1.sf1.2\">(a)</text></tag>\
           <text fontsize=\"90%\" xml:id=\"S0.F1.sf1.3\">A</text></caption></figure>\
           <figure align=\"center\" class=\"ltx_figure_panel\" inlist=\"lof\" panelwidth=\"0.3\" xml:id=\"S0.F1.sf2\">\
           <tags><tag><text fontsize=\"90%\" xml:id=\"S0.F1.sf2.1\">(b)</text></tag>\
           <tag role=\"refnum\">1b</tag></tags>\
           <graphics graphic=\"b.png\" options=\"width=120.57286pt,keepaspectratio=true\" xml:id=\"S0.F1.sf2.g1\"/>\
           <toccaption><tag close=\" \">b</tag>B</toccaption><caption><tag close=\" \">\
           <text fontsize=\"90%\" xml:id=\"S0.F1.sf2.2\">(b)</text></tag>\
           <text fontsize=\"90%\" xml:id=\"S0.F1.sf2.3\">B</text></caption></figure>\
           <figure align=\"center\" class=\"ltx_figure_panel\" inlist=\"lof\" panelwidth=\"0.3\" xml:id=\"S0.F1.sf3\">\
           <tags><tag><text fontsize=\"90%\" xml:id=\"S0.F1.sf3.1\">(c)</text></tag>\
           <tag role=\"refnum\">1c</tag></tags>\
           <graphics graphic=\"f.png\" options=\"width=120.57286pt,keepaspectratio=true\" xml:id=\"S0.F1.sf3.g1\"/>\
           <toccaption><tag close=\" \">c</tag>C</toccaption><caption><tag close=\" \">\
           <text fontsize=\"90%\" xml:id=\"S0.F1.sf3.2\">(c)</text></tag>\
           <text fontsize=\"90%\" xml:id=\"S0.F1.sf3.3\">C</text></caption></figure><toccaption>\
           <tag close=\" \">1</tag>Three.</toccaption><caption><tag close=\": \">Figure 1</tag>Three.</caption>\
           </figure>",
      )],
    ),
    (
      repro!("list-structure/label_opening_glue_nets_against_labelsep.tex"),
      &[
        (
          "item",
          "S0.I1.ix1",
          "<item xml:id=\"S0.I1.ix1\"><tags><tag>x</tag><tag role=\"typerefnum\">item </tag></tags><para xml:id=\"S0.I1.ix1.p1\"><p xml:id=\"S0.I1.ix1.p1.1\">a</p></para></item>",
        ),
        (
          "item",
          "S0.I2.i1",
          "<item xml:id=\"S0.I2.i1\"><tags><tag>•</tag><tag role=\"typerefnum\">1st item</tag></tags><para xml:id=\"S0.I2.i1.p1\"><p xml:id=\"S0.I2.i1.p1.1\">b</p></para></item>",
        ),
        (
          "item",
          "S0.I3.i1",
          "<item xml:id=\"S0.I3.i1\"><tags><tag>•\u{2002}</tag><tag role=\"typerefnum\">1st item</tag></tags><para xml:id=\"S0.I3.i1.p1\"><p xml:id=\"S0.I3.i1.p1.1\">c</p></para></item>",
        ),
        (
          "item",
          "S0.I4.ix1",
          "<item xml:id=\"S0.I4.ix1\"><tags><tag><text font=\"bold\" xml:id=\"S0.I4.ix1.1\">D</text></tag><tag role=\"typerefnum\">item D</tag></tags><para xml:id=\"S0.I4.ix1.p1\"><p xml:id=\"S0.I4.ix1.p1.1\">d</p></para></item>",
        ),
        (
          "item",
          "S0.I5.ix1",
          "<item xml:id=\"S0.I5.ix1\"><tags><tag>\u{2003}P</tag><tag role=\"typerefnum\">item \u{2003}P</tag></tags><para xml:id=\"S0.I5.ix1.p1\"><p xml:id=\"S0.I5.ix1.p1.1\">e</p></para></item>",
        ),
        (
          "item",
          "S0.I5.ix2",
          "<item xml:id=\"S0.I5.ix2\"><tags><tag>(f)\u{2002}</tag><tag role=\"typerefnum\">item (f)\u{2002}</tag></tags><para xml:id=\"S0.I5.ix2.p1\"><p xml:id=\"S0.I5.ix2.p1.1\">f</p></para></item>",
        ),
        (
          "item",
          "S0.I6.ix1",
          "<item xml:id=\"S0.I6.ix1\"><tags><tag><text xml:id=\"S0.I6.ix1.1\">k</text></tag><tag role=\"typerefnum\">item <text xml:id=\"S0.I6.ix1.2\">k</text></tag></tags><para xml:id=\"S0.I6.ix1.p1\"><p xml:id=\"S0.I6.ix1.p1.1\">g</p></para></item>",
        ),
        (
          "p",
          "p1.4",
          "<p xml:id=\"p1.4\">Kept <text xml:id=\"p1.4.1\">\u{2002}k</text>.</p>",
        ),
        (
          "item",
          "S0.I7.ix1",
          "<item xml:id=\"S0.I7.ix1\"><tags><tag><text font=\"bold\" xml:id=\"S0.I7.ix1.1\">Thm.</text></tag></tags><para xml:id=\"S0.I7.ix1.p1\"><p xml:id=\"S0.I7.ix1.p1.1\">h</p></para></item>",
        ),
      ],
    ),
    (
      repro!("list-structure/enumitem_wide_clears_the_left_margin.tex"),
      &[
        (
          "itemize",
          "S0.I1",
          "<itemize cssstyle=\"--ltx-enum-leftmargin:0pt\" xml:id=\"S0.I1\"><item xml:id=\"S0.I1.i1\"><tags><tag>•</tag><tag role=\"typerefnum\">1st item</tag></tags><para xml:id=\"S0.I1.i1.p1\"><p xml:id=\"S0.I1.i1.p1.1\">A 345.0pt 0.0pt</p></para></item></itemize>",
        ),
        (
          "itemize",
          "S0.I2",
          "<itemize cssstyle=\"--ltx-enum-leftmargin:0pt\" xml:id=\"S0.I2\"><item xml:id=\"S0.I2.i1\"><tags><tag>•</tag><tag role=\"typerefnum\">1st item</tag></tags><para xml:id=\"S0.I2.i1.p1\"><p xml:id=\"S0.I2.i1.p1.1\">B 345.0pt 0.0pt</p></para></item></itemize>",
        ),
        (
          "enumerate",
          "S0.I3",
          "<enumerate cssstyle=\"--ltx-enum-leftmargin:0pt\" xml:id=\"S0.I3\"><item xml:id=\"S0.I3.i1\"><tags><tag>1.</tag><tag role=\"refnum\">1</tag><tag role=\"typerefnum\">item\u{a0}1</tag></tags><para xml:id=\"S0.I3.i1.p1\"><p xml:id=\"S0.I3.i1.p1.1\">C 345.0pt 0.0pt</p></para></item></enumerate>",
        ),
        (
          "itemize",
          "S0.I4",
          "<itemize cssstyle=\"--ltx-enum-leftmargin:0pt\" xml:id=\"S0.I4\"><item xml:id=\"S0.I4.i1\"><tags><tag>•</tag><tag role=\"typerefnum\">1st item</tag></tags><para xml:id=\"S0.I4.i1.p1\"><p xml:id=\"S0.I4.i1.p1.1\">D 345.0pt 0.0pt</p></para></item></itemize>",
        ),
        (
          "itemize",
          "S0.I5",
          "<itemize cssstyle=\"--ltx-enum-leftmargin:1cm\" xml:id=\"S0.I5\"><item xml:id=\"S0.I5.i1\"><tags><tag>•</tag><tag role=\"typerefnum\">1st item</tag></tags><para xml:id=\"S0.I5.i1.p1\"><p xml:id=\"S0.I5.i1.p1.1\">E 316.54726pt 28.45274pt</p></para></item></itemize>",
        ),
        (
          "description",
          "S0.I6",
          "<description cssstyle=\"--ltx-enum-leftmargin:0pt\" xml:id=\"S0.I6\"><item xml:id=\"S0.I6.ix1\"><tags><tag><text font=\"bold\" xml:id=\"S0.I6.ix1.1\">x</text></tag><tag role=\"typerefnum\">item x</tag></tags><para xml:id=\"S0.I6.ix1.p1\"><p xml:id=\"S0.I6.ix1.p1.1\">F 345.0pt 0.0pt</p></para></item></description>",
        ),
        (
          "itemize",
          "S0.I7",
          "<itemize cssstyle=\"--ltx-enum-leftmargin:0pt\" xml:id=\"S0.I7\"><item xml:id=\"S0.I7.i1\"><tags><tag>•</tag><tag role=\"typerefnum\">1st item</tag></tags><para xml:id=\"S0.I7.i1.p1\"><p xml:id=\"S0.I7.i1.p1.1\">G 345.0pt 0.0pt</p></para></item></itemize>",
        ),
        (
          "itemize",
          "S0.I8",
          "<itemize cssstyle=\"--ltx-enum-leftmargin:0pt\" xml:id=\"S0.I8\"><item xml:id=\"S0.I8.i1\"><tags><tag>•</tag><tag role=\"typerefnum\">1st item</tag></tags><para xml:id=\"S0.I8.i1.p1\"><p xml:id=\"S0.I8.i1.p1.1\">H 345.0pt 0.0pt</p></para></item></itemize>",
        ),
      ],
    ),
    (
      repro!("list-structure/note_and_cell_paragraphs_are_unspaced_in_a_list.tex"),
      &[
        (
          "note",
          "footnote1",
          "<note mark=\"1\" role=\"footnote\" xml:id=\"footnote1\"><tags><tag>1</tag><tag role=\"refnum\">1</tag><tag role=\"typerefnum\">footnote 1</tag></tags>F 0.0pt.<itemize xml:id=\"S0.I1.i1.I1\"><item xml:id=\"S0.I1.i1.I1.i1\"><tags><tag><text font=\"bold\" xml:id=\"S0.I1.i1.I1.i1.1\">–</text></tag><tag role=\"typerefnum\">1st item</tag></tags><para xml:id=\"S0.I1.i1.I1.i1.p1\"><p xml:id=\"S0.I1.i1.I1.i1.p1.1\">N</p></para></item></itemize></note>",
        ),
        (
          "p",
          "S0.I1.i2.p1.1.1.1.1.1",
          "<p xml:id=\"S0.I1.i2.p1.1.1.1.1.1\">C 0.0pt.</p>",
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
