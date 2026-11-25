//! Footnote series beside the kernel's footnotes: manyfoot's declared series (`\DeclareNewFootnote`, `\newfootnote`),
//! nccfoots' given marks and bigfoot's series, each a kernel note marked as pdflatex prints it (`manyfoot_sty.rs`,
//! `nccfoots_sty.rs`, `bigfoot_sty.rs`). Its own table, not the Error boundary's: a row whose package is not in the
//! TeX Live at hand is skipped visibly. Repros `tools/perfect_kernel/repros/package-bindings/`.
use latexml::util::test::{assert_element, convert_files_with, kpse_has, rng_error_count};

use super::perfect_kernel_batch46::{error_count, warning_count};

/// A repro: its name, the package it needs, its source, (errors, warnings), and its paragraph `p1.1` whole.
type Row = (
  &'static str,
  &'static str,
  &'static str,
  (usize, usize),
  &'static str,
);

const ROWS: &[Row] = &[
  // Each style's `\footnote<X>` (arabic, `[roman]` with `[<n>]`, `[fnsymbol]`, `[gobble]`), `\footnotetext<X>` and
  // `\Footnotetext<X>`, and a series note in a minipage keeping the series' count while the kernel's is lettered;
  // INT081P: the texts inline after plain-text marks, and manyfoot.sty:390's `\endminipage` Warning.
  (
    "manyfoot_series_notes",
    "manyfoot.sty",
    include_str!("../../../tools/perfect_kernel/repros/package-bindings/manyfoot_series_notes.tex"),
    (0, 0),
    r#"<p xml:id="p1.1">Alpha<note mark="1" role="footnote" xml:id="footnote1"><tags>
          <tag>1</tag>
          <tag role="refnum">1</tag>
          <tag role="typerefnum">footnote 1</tag>
        </tags>Main note.</note> beta<note mark="1" role="footnote" xml:id="footnotex1"><tags>
          <tag>1</tag>
          <tag role="refnum">1</tag>
          <tag role="typerefnum">footnote 1</tag>
        </tags>First B.</note> gamma<note mark="2" role="footnote" xml:id="footnotex2"><tags>
          <tag>2</tag>
          <tag role="refnum">2</tag>
          <tag role="typerefnum">footnote 2</tag>
        </tags>Second B.</note>
delta<note mark="i" role="footnote" xml:id="footnotex3"><tags>
          <tag>i</tag>
          <tag role="refnum">i</tag>
          <tag role="typerefnum">footnote i</tag>
        </tags>Roman one.</note> epsilon<note mark="iv" role="footnote" xml:id="footnotex4"><tags>
          <tag>iv</tag>
          <tag role="refnum">iv</tag>
          <tag role="typerefnum">footnote iv</tag>
        </tags>Roman four.</note> zeta<note mark="*" role="footnote" xml:id="footnotex5"><tags>
          <tag>*</tag>
          <tag role="refnum">*</tag>
          <tag role="typerefnum">footnote *</tag>
        </tags>Symbol one.</note>
eta<note role="footnote" xml:id="footnotex6"><tags>
          <tag role="typerefnum">footnote </tag>
        </tags>Unmarked G.</note> theta3 iota<note mark="3" role="footnotetext" xml:id="footnotex7">Text of B3.</note>
kappa<note mark="x" role="footnotetext" xml:id="footnotex8">Given x.</note>
<inline-block class="ltx_minipage" vattach="middle" width="113.8pt" xml:id="p1.1.1">
        <p xml:id="p1.1.1.1">Inside<note mark="a" role="footnote" xml:id="footnotex9"><tags>
              <tag>a</tag>
              <tag role="refnum">a</tag>
              <tag role="typerefnum">footnote a</tag>
            </tags>Minipage note.</note> and<note mark="4" role="footnote" xml:id="footnotex10"><tags>
              <tag>4</tag>
              <tag role="refnum">4</tag>
              <tag role="typerefnum">footnote 4</tag>
            </tags>B in minipage.</note></p>
      </inline-block></p>"#,
  ),
  // The `para` style's text sets no mark unless given (manyfoot.sty:68); a `plain` series' empty mark keeps the last
  // `\@thefnmark`, which a `\footnote<X>` sets (witness 2003.11645's `[para*]` and `\Footnotetextauthor{}{…}`).
  (
    "manyfoot_para_series",
    "manyfoot.sty",
    include_str!("../../../tools/perfect_kernel/repros/package-bindings/manyfoot_para_series.tex"),
    (0, 0),
    r#"<p xml:id="p1.1">Alpha<note mark="1" role="footnote" xml:id="footnotex1"><tags>
          <tag>1</tag>
          <tag role="refnum">1</tag>
          <tag role="typerefnum">footnote 1</tag>
        </tags>Para A one.</note> beta<note mark="2" role="footnote" xml:id="footnotex2"><tags>
          <tag>2</tag>
          <tag role="refnum">2</tag>
          <tag role="typerefnum">footnote 2</tag>
        </tags>Para A two.</note>
gamma<note mark="2" role="footnotetext" xml:id="footnotex3">Author text, last mark.</note> delta<note mark="y" role="footnotetext" xml:id="footnotex4">Marked y.</note>
epsilon<note role="footnotetext" xml:id="footnotex5">Para text, no mark.</note></p>"#,
  ),
  // bigfoot.sty:383-384 redefines the series' text makers (its boxes, and perpage's sorted `FN@totalid`, which
  // warned "\c@pchk@FN@totalid is not a register"); INT081P set the text as a `<block>` after the paragraph.
  (
    "bigfoot_series_note",
    "bigfoot.sty",
    include_str!("../../../tools/perfect_kernel/repros/package-bindings/bigfoot_series_note.tex"),
    (0, 0),
    r#"<p xml:id="p1.1">Alpha<note mark="1" role="footnote" xml:id="footnote1"><tags>
          <tag>1</tag>
          <tag role="refnum">1</tag>
          <tag role="typerefnum">footnote 1</tag>
        </tags>Main note.</note> beta<note mark="1" role="footnote" xml:id="footnotex1"><tags>
          <tag>1</tag>
          <tag role="refnum">1</tag>
          <tag role="typerefnum">footnote 1</tag>
        </tags>Series B note.</note> gamma<note mark="z" role="footnotetext" xml:id="footnotex2">Given z.</note></p>"#,
  ),
  // nccfoots' given marks: INT081P marked both texts "0", the footnote counter's.
  (
    "nccfoots_given_marks",
    "nccfoots.sty",
    include_str!("../../../tools/perfect_kernel/repros/package-bindings/nccfoots_given_marks.tex"),
    (0, 0),
    r#"<p xml:id="p1.1">Alpha*<note mark="*" role="footnotetext" xml:id="footnotex1">Starred note.</note> beta7 gamma<note mark="7" role="footnotetext" xml:id="footnotex2">Text of seven.</note></p>"#,
  ),
  // manyfoot's begin-document code keeps what it defines (`\MFL@fudgefactor`, `\MFL@paraskip`,
  // `\defaultfootnoterule`), which a package's own para text maker reads (dgruyter.sty:2379); MANYFOOT: 2 errors.
  (
    "manyfoot_para_text_reads_hook_definitions",
    "manyfoot.sty",
    include_str!(
      "../../../tools/perfect_kernel/repros/package-bindings/manyfoot_para_text_reads_hook_definitions.tex"
    ),
    (0, 0),
    r#"<p xml:id="p1.1">Alpha<note mark="1" role="footnote" xml:id="footnotex1"><tags>
          <tag>1</tag>
          <tag role="refnum">1</tag>
          <tag role="typerefnum">footnote 1</tag>
        </tags>Para A note.</note> beta<text xml:id="p1.1.1"><text font="bold" xml:id="p1.1.1.1">1</text> Para A text.</text> gamma<text xml:id="p1.1.2"><text font="bold" xml:id="p1.1.2.1">z</text> Given z.</text></p>"#,
  ),
  // A series' text is not the text of the main `\footnotemark` of the same number (`_fnseries`); MANYFOOT: the
  // main mark held SERIES-A-TEXT, MAIN-TEXT left unpaired.
  (
    "manyfoot_series_text_is_not_a_main_mark_text",
    "manyfoot.sty",
    include_str!(
      "../../../tools/perfect_kernel/repros/package-bindings/manyfoot_series_text_is_not_a_main_mark_text.tex"
    ),
    (0, 0),
    r#"<p xml:id="p1.1">Main<note mark="1" role="footnote" xml:id="footnote1"><tags>
          <tag>1</tag>
          <tag role="refnum">1</tag>
          <tag role="typerefnum">footnote 1</tag>
        </tags>MAIN-TEXT</note> and series1 text.
<note mark="1" role="footnotetext" xml:id="footnotex1">SERIES-A-TEXT</note></p>"#,
  ),
  // Two series interleaved with main marks, texts and notes; MANYFOOT: the main mark 1 held A-ONE-TEXT.
  (
    "manyfoot_interleaved_series_and_main_texts",
    "manyfoot.sty",
    include_str!(
      "../../../tools/perfect_kernel/repros/package-bindings/manyfoot_interleaved_series_and_main_texts.tex"
    ),
    (0, 0),
    r#"<p xml:id="p1.1">Main<note mark="1" role="footnote" xml:id="footnote1"><tags>
          <tag>1</tag>
          <tag role="refnum">1</tag>
          <tag role="typerefnum">footnote 1</tag>
        </tags>MAIN-ONE-TEXT</note> one1 twoi three.
<note mark="i" role="footnotetext" xml:id="footnotex1">B-ONE-TEXT</note>
<note mark="1" role="footnotetext" xml:id="footnotex2">A-ONE-TEXT</note>

Again<note mark="2" role="footnote" xml:id="footnote2"><tags>
          <tag>2</tag>
          <tag role="refnum">2</tag>
          <tag role="typerefnum">footnote 2</tag>
        </tags>MAIN-TWO.</note> four<note mark="2" role="footnote" xml:id="footnotex4"><tags>
          <tag>2</tag>
          <tag role="refnum">2</tag>
          <tag role="typerefnum">footnote 2</tag>
        </tags>A-TWO.</note> fiveii six<note mark="3" role="footnote" xml:id="footnote3"><tags>
          <tag>3</tag>
          <tag role="refnum">3</tag>
          <tag role="typerefnum">footnote 3</tag>
        </tags>MAIN-THREE-TEXT</note>.

<note mark="ii" role="footnotetext" xml:id="footnotex6">B-TWO-TEXT</note></p>"#,
  ),
];

/// Every row converted once, in-process: the exact counts, a schema-valid document and the paragraph whole.
#[test]
fn footnote_series_rows() {
  for (name, package, tex, (errors, warnings), paragraph) in ROWS {
    if !kpse_has(package) {
      eprintln!("SKIP {name}: {package} is not in this TeX Live");
      continue;
    }
    let (log, xml) = convert_files_with(tex, &[], Some("ar5iv.sty"));
    assert_eq!(error_count(&log), *errors, "{name}:\n{log}");
    assert_eq!(warning_count(&log), *warnings, "{name}:\n{log}");
    if let Some(lines) = rng_error_count(&xml) {
      assert_eq!(lines, 0, "{name}: schema:\n{xml}");
    }
    assert_element(&xml, "p", &["xml:id=\"p1.1\""], paragraph);
  }
}
