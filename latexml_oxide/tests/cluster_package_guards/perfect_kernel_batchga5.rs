//! Red/green guards for batch GA5: decisions on a token that TeX takes by its meaning, read by meaning (GA0 finding
//! 10). One table, a row per changed code path whose output differed, each the `\let`-copy shape that was wrong (oracle
//! in the repro header). A module of its own: no table of meaning-for-spelling rows existed; later GA batches add rows
//! (GA4: a table package's state out of the kernel).
use latexml::util::test::{assert_element, convert_files_with};

use super::perfect_kernel_batch46::{error_count, warning_count};

/// One repro: its source, its exact (errors, warnings), whole elements `(tag, attributes, element)` and text the
/// output holds.
struct Row {
  name:     &'static str,
  tex:      &'static str,
  counts:   (usize, usize),
  elements: &'static [(&'static str, &'static str, &'static str)],
  holds:    &'static [&'static str],
}

const ROWS: &[Row] = &[
  // tex_math.rs `eqno_stops_at`: `\let\dollar=$` ends the tag, as Perl's `defined_as`
  Row {
    name:     "eqno_tag_ends_at_a_let_dollar",
    tex:      include_str!(
      "../../../tools/perfect_kernel/repros/kernel-alignment/eqno_tag_ends_at_a_let_dollar.tex"
    ),
    counts:   (0, 0),
    elements: &[(
      "p",
      "xml:id=\"p1.1\"",
      r#"<p xml:id="p1.1">Text after.</p>"#,
    )],
    holds:    &[r#"tex="e=f""#, r#"<XMTok meaning="3" role="NUMBER""#],
  },
  // tex_tables.rs `before_cell_unlist`/`after_cell_unlist`, `is_fill`: `\let\myhfil\hfil` in a math template
  Row {
    name:     "halign_let_hfil_in_math_template",
    tex:      include_str!(
      "../../../tools/perfect_kernel/repros/kernel-alignment/halign_let_hfil_in_math_template.tex"
    ),
    counts:   (0, 0),
    elements: &[(
      "td",
      "xml:id=\"p1.1.1.1\"",
      r#"<td align="center" class="ltx_nopad_l ltx_nopad_r" xml:id="p1.1.1.1"><Math mode="inline" tex="x" text="x" xml:id="p1.m1"><XMath xml:id="p1.m1.1"><XMTok font="italic" role="UNKNOWN">x</XMTok></XMath></Math></td>"#,
    )],
    holds:    &[],
  },
  // tex_tables.rs `template_after_fill_defeated`: the after template's last fill by the same reading (`is_fill`) as
  // `has_after_fill`, so content after a `\myhfil` defeats it as after `\hfil` (pinned, the same on GA5base)
  Row {
    name:     "halign_let_hfil_before_content_after",
    tex:      include_str!(
      "../../../tools/perfect_kernel/repros/kernel-alignment/halign_let_hfil_before_content_after.tex"
    ),
    counts:   (0, 0),
    elements: &[(
      "td",
      "xml:id=\"p1.1.1.1\"",
      r#"<td align="right" class="ltx_nopad_l ltx_nopad_r" xml:id="p1.1.1.1">a  <Math mode="inline" tex="|" text="|" xml:id="p1.m1"><XMath xml:id="p1.m1.1"><XMTok role="VERTBAR" stretchy="false">|</XMTok></XMath></Math></td>"#,
    )],
    holds:    &[],
  },
  // GA4 (GA0 findings 2, 4): longtable's `\kill` and caption row through the alignment's `RowDisposition`, pinned
  // before the move (2010.09763, 2201.11330)
  Row {
    name:     "longtable_kill_row_and_caption_note",
    tex:      include_str!(
      "../../../tools/perfect_kernel/repros/alignment-bindings/longtable_kill_row_and_caption_note.tex"
    ),
    counts:   (0, 0),
    elements: &[(
      "table",
      "xml:id=\"S0.T1\"",
      r#"<table inlist="lot" xml:id="S0.T1"><tags><tag>Table 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Table 1</tag></tags><caption><tag close=": ">Table 1</tag>The caption<note mark="1" role="footnote" xml:id="footnote1"><tags><tag>1</tag><tag role="refnum">1</tag><tag role="typerefnum">footnote 1</tag></tags>A note.</note></caption><toccaption><tag close=" ">1</tag>The caption</toccaption><tabular xml:id="S0.T1.1"><thead><tr xml:id="S0.T1.1.1"><td align="left" thead="column" xml:id="S0.T1.1.1.1">H1</td><td align="left" thead="column" xml:id="S0.T1.1.1.2">H2</td></tr></thead><tbody><tr xml:id="S0.T1.1.2"><td align="left" xml:id="S0.T1.1.2.1">a</td><td align="left" xml:id="S0.T1.1.2.2">b</td></tr><tr xml:id="S0.T1.1.3"><td align="left" xml:id="S0.T1.1.3.1">c</td><td align="left" xml:id="S0.T1.1.3.2">d</td></tr></tbody></tabular></table>"#,
    )],
    holds:    &["After TEXT."],
  },
  // GA4b (revGA4 p09): a nested tabular after a longtable caption goes with the caption's row (RED, GA4C post-merge),
  // the loss counted by one Warning per table (0 on GA4); the captions and the text after the tables kept
  Row {
    name:     "longtable_caption_then_a_nested_tabular_red",
    tex:      include_str!(
      "../../../tools/perfect_kernel/repros/alignment-bindings/longtable_caption_then_a_nested_tabular_red.tex"
    ),
    counts:   (0, 2),
    elements: &[],
    holds:    &[
      r#"<caption><tag close=": ">Table 1</tag>Cap</caption>"#,
      r#"<caption><tag close=": ">Table 2</tag>Two</caption>"#,
      "After TEXT.",
    ],
  },
  // GA4b: the caption's cell is a `\multicolumn`, so the column template's own text ("." of `r@{.}`, 1111.5166) is no
  // loss; the "7" after the third caption is (1903.09068's shape), one Warning
  Row {
    name:     "longtable_caption_row_template_text_not_counted",
    tex:      include_str!(
      "../../../tools/perfect_kernel/repros/alignment-bindings/longtable_caption_row_template_text_not_counted.tex"
    ),
    counts:   (0, 1),
    elements: &[(
      "table",
      "xml:id=\"S0.T1\"",
      r#"<table inlist="lot" labels="LABEL:t:a" xml:id="S0.T1"><tags><tag>Table 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Table 1</tag></tags><caption><tag close=": ">Table 1</tag>Dot</caption><toccaption><tag close=" ">1</tag>Dot</toccaption><tabular xml:id="S0.T1.1"><tr xml:id="S0.T1.1.1"><td align="right" class="ltx_nopad_r" xml:id="S0.T1.1.1.1">1.</td><td align="left" class="ltx_nopad_l" xml:id="S0.T1.1.1.2">5</td></tr></tabular></table>"#,
    )],
    holds:    &[
      r#"<caption><tag close=": ">Table 3</tag>Five</caption>"#,
      "After TEXT.",
    ],
  },
  // tabularx's binding lets `\verb` to its `\TX@verb` in the environment, the kernel's after it (2404.06128,
  // 2309.04295)
  Row {
    name:     "tabularx_verb_starred_in_a_cell",
    tex:      include_str!(
      "../../../tools/perfect_kernel/repros/alignment/tabularx_verb_starred_in_a_cell.tex"
    ),
    counts:   (0, 0),
    elements: &[
      (
        "tabular",
        "xml:id=\"p1.1\"",
        r#"<tabular vattach="middle" xml:id="p1.1"><tbody><tr xml:id="p1.1.1"><td align="left" xml:id="p1.1.1.1">A</td><td align="left" xml:id="p1.1.1.2"><inline-block vattach="top" xml:id="p1.1.1.2.1"><p xml:id="p1.1.1.2.1.1"><verbatim font="typewriter" xml:id="p1.1.1.2.1.1.1">x+y</verbatim><text xml:id="p1.1.1.2.1.1.2"> and <verbatim font="typewriter" xml:id="p1.1.1.2.1.1.2.1">a␣b</verbatim></text></p></inline-block></td></tr></tbody></tabular>"#,
      ),
      (
        "p",
        "xml:id=\"p1.2\"",
        r#"<p xml:id="p1.2">After <verbatim font="typewriter" xml:id="p1.2.1">q r</verbatim> TEXT.</p>"#,
      ),
    ],
    holds:    &[],
  },
];

/// Every row converted once, in-process: exact counts, the whole elements.
#[test]
fn batchga5_meaning_rows() {
  for row in ROWS {
    let (log, xml) = convert_files_with(row.tex, &[], Some("ar5iv.sty"));
    let name = row.name;
    assert_eq!(error_count(&log), row.counts.0, "{name}: errors\n{log}");
    assert_eq!(warning_count(&log), row.counts.1, "{name}: warnings\n{log}");
    for (tag, attrs, element) in row.elements {
      assert_element(&xml, tag, &[attrs], element);
    }
    for text in row.holds {
      assert!(xml.contains(text), "{name}: no {text:?}\n{xml}");
    }
  }
}
