//! Red/green guards for batch 77v (quality batch 4): package bindings that lost authored text or mislabelled it —
//! thm-restate's `restate=` key, todonotes' option list, KOMA's bibliography preamble, sn-jnl's table rules, theorem
//! counters over a shared one and `\newcounter` over an existing `\the<ctr>`, multline's `\tag`/`\notag`, a text-mode
//! `{IEEEeqnarrayboxm}`, amsppt's non-`\long` `\footnote`, combine's class options and endnotes' `\theenmark`. Each
//! guard asserts the diagnostics by count and the authored content in its whole element.
use latexml::util::test::{convert_with, xml_element};

use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch57::{assert_elements, assert_elements_with},
};

const AR5IV: &str = "ar5iv.sty";

/// thm-restate.sty:198-215: a theorem's `restate=<name>` key stores it for `\<name>` as `{restatable}` does; the
/// key is no note ("(restate=other)" was the title, `\mainthm`/`\other` undefined: 2 errors). Witnesses 2108.11527,
/// 2602.09462. Repro package-bindings/thmtools_restate_key.
#[test]
fn thmtools_restate_key() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/package-bindings/thmtools_restate_key.tex");
  assert_elements(tex, AR5IV, (0, 0), &[
    (
      "theorem",
      "Thmtheorem1",
      r#"<theorem class="ltx_theorem_theorem" inlist="thm theorem:theorem" labels="LABEL:restatable:mainthm" xml:id="Thmtheorem1"><tags><tag>Theorem 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Theorem 1</tag></tags><title class="ltx_runin"><tag><text font="bold" xml:id="Thmtheorem1.1">Theorem 1</text></tag><text font="bold" xml:id="Thmtheorem1.2"> </text>(Main)<text font="bold" xml:id="Thmtheorem1.3">.</text></title><para xml:id="Thmtheorem1.p1"><p xml:id="Thmtheorem1.p1.1"><text font="italic" xml:id="Thmtheorem1.p1.1.1">A.</text></p></para></theorem>"#,
    ),
    (
      "theorem",
      "Thmtheorem2",
      r#"<theorem class="ltx_theorem_theorem" inlist="thm theorem:theorem" labels="LABEL:restatable:other" xml:id="Thmtheorem2"><tags><tag>Theorem 2</tag><tag role="refnum">2</tag><tag role="typerefnum">Theorem 2</tag></tags><title class="ltx_runin"><tag><text font="bold" xml:id="Thmtheorem2.1">Theorem 2</text></tag><text font="bold" xml:id="Thmtheorem2.2">.</text></title><para xml:id="Thmtheorem2.p1"><p xml:id="Thmtheorem2.p1.1"><text font="italic" xml:id="Thmtheorem2.p1.1.1">B.</text></p></para></theorem>"#,
    ),
    (
      "p",
      "p1.1",
      r#"<p xml:id="p1.1">Again: See <ref labelref="LABEL:restatable:mainthm"/> and See <ref labelref="LABEL:restatable:other"/></p>"#,
    ),
  ]);
}

/// todonotes.sty:561 `\todo[<options>]{<text>}`: the options are keys, never the note's mark — the note is numbered
/// as any (it was `mark="mine"`, the tags "mine" and "ToDo mine"). 19 papers' `\todo[inline]`; 2407.14906.
/// Repro package-bindings/todonotes_style_key.
#[test]
fn todonotes_style_key() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/package-bindings/todonotes_style_key.tex");
  assert_elements(tex, AR5IV, (0, 0), &[(
    "p",
    "p1.1",
    r#"<p xml:id="p1.1">Text<note inlist="todo" mark="1" role="todo" xml:id="todo1"><tags><tag>1</tag><tag role="refnum">1</tag><tag role="typerefnum">ToDo 1</tag></tags>Fix this.</note> more.</p>"#,
  )]);
}

/// scrartcl.cls:6752-6763: `\setbibpreamble{<text>}` is typeset between the bibliography's heading and its list (it
/// was dropped without a diagnostic). Repro class-bindings/koma_setbibpreamble_is_printed.
#[test]
fn koma_setbibpreamble_is_printed() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/koma_setbibpreamble_is_printed.tex"
  );
  assert_elements(tex, AR5IV, (0, 0), &[(
    "bibliography",
    "bib",
    r#"<bibliography inlist="toc" xml:id="bib"><title>References</title><para class="ltx_noindent" xml:id="bib.p1"><p xml:id="bib.p1.1">Pre text.</p></para><biblist><bibitem key="a" xml:id="bib.bib1"><tags><tag>[1]</tag><tag role="refnum">1</tag></tags><bibblock> A. Author, Title.</bibblock></bibitem></biblist></bibliography>"#,
  )]);
}

/// sn-jnl's table rules without the paper's own booktabs: the class copies that define `\toprule`/`\midrule`/
/// `\botrule` themselves (2605.00003 :1280-1321, 2006.16202) and the class as distributed, which loads booktabs, both
/// rule the rows (sn_jnl_cls.rs since 67z). Repro class-bindings/sn_jnl_table_rules_without_booktabs.
#[test]
fn sn_jnl_table_rules_without_booktabs() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/sn_jnl_table_rules_without_booktabs.tex"
  );
  assert_elements(tex, AR5IV, (0, 0), &[(
    "tabular",
    "S0.T1.2",
    r#"<tabular vattach="middle" xml:id="S0.T1.2"><tbody><tr xml:id="S0.T1.2.1"><td align="left" border="tt" xml:id="S0.T1.2.1.1">A</td><td align="left" border="tt" xml:id="S0.T1.2.1.2">B</td></tr><tr xml:id="S0.T1.2.2"><td align="left" border="b t" xml:id="S0.T1.2.2.1">C</td><td align="left" border="b t" xml:id="S0.T1.2.2.2">D</td></tr></tbody></tabular>"#,
  )]);
}

/// latex.ltx `\@xnthm`: `\newtheorem{proposition}{Proposition}[section]` after a `[theorem]`-sharing one steps its own
/// counter (it printed "Proposition 1.0", refnum 1.2). Witness 2407.21129. Repro
/// macro-state/newtheorem_over_shared_counter_theorem.
#[test]
fn newtheorem_over_shared_counter_theorem() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/macro-state/newtheorem_over_shared_counter_theorem.tex"
  );
  assert_elements(tex, AR5IV, (0, 0), &[(
    "theorem",
    "S1.Thmproposition1",
    r#"<theorem class="ltx_theorem_proposition" inlist="thm theorem:proposition" xml:id="S1.Thmproposition1"><tags><tag>Proposition 1.1</tag><tag role="refnum">1.1</tag><tag role="typerefnum">Proposition 1.1</tag></tags><title class="ltx_runin"><tag><text font="bold" xml:id="S1.Thmproposition1.1">Proposition 1.1</text></tag></title><para xml:id="S1.Thmproposition1.p1"><p xml:id="S1.Thmproposition1.p1.1"><text font="italic" xml:id="S1.Thmproposition1.p1.1.1">P.</text></p></para></theorem>"#,
  )]);
}

/// arxbj's `\newtheorem{rem}{Remark}` over elsart_support's `\newtheorem{rem}[thm]{Remark}`: "Remark 1", "Remark 2"
/// (both were "Remark 0"). Witnesses 1205.2947, 1307.5990. Repro class-bindings/elsart_support_theorem_tag_zero.
#[test]
fn elsart_support_theorem_tag_zero() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/elsart_support_theorem_tag_zero.tex"
  );
  assert_elements(tex, AR5IV, (0, 0), &[
    (
      "theorem",
      "Thmrem1",
      r#"<theorem class="ltx_theorem_rem" inlist="thm theorem:rem" xml:id="Thmrem1"><tags><tag>Remark 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Remark 1</tag></tags><title class="ltx_runin"><tag><text font="bold" xml:id="Thmrem1.1">Remark 1</text></tag></title><para xml:id="Thmrem1.p1"><p xml:id="Thmrem1.p1.1"><text font="italic" xml:id="Thmrem1.p1.1.1">A.</text></p></para></theorem>"#,
    ),
    (
      "theorem",
      "Thmrem2",
      r#"<theorem class="ltx_theorem_rem" inlist="thm theorem:rem" xml:id="Thmrem2"><tags><tag>Remark 2</tag><tag role="refnum">2</tag><tag role="typerefnum">Remark 2</tag></tags><title class="ltx_runin"><tag><text font="bold" xml:id="Thmrem2.1">Remark 2</text></tag></title><para xml:id="Thmrem2.p1"><p xml:id="Thmrem2.p1.1"><text font="italic" xml:id="Thmrem2.p1.1.1">B.</text></p></para></theorem>"#,
    ),
  ]);
}

/// latex.ltx `\@definecounter` keeps an existing `\the<ctr>`, with its warning: llncs's `\let\c@case\relax
/// \newcounter{case}` over its alias keeps `\thecase` = `\thetheorem` (pdflatex "Lemma 1, Case 1, Case 1, Lemma 2,
/// Case 2, Lemma 3"; the cases printed their own counter). Witness 2303.08726. Repro
/// class-bindings/llncs_alias_newcounter.
#[test]
fn llncs_alias_newcounter() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/class-bindings/llncs_alias_newcounter.tex");
  let xml = assert_elements_with(
    tex,
    AR5IV,
    (0, 1),
    &["Command `\\thecase' already defined -- not changed"],
    &[],
  );
  let tags: Vec<String> = ["case1", "case2", "Thmtheorem2", "case1a", "Thmtheorem3"]
    .iter()
    .map(|id| {
      let theorem =
        xml_element(&xml, "theorem", &[&format!("xml:id=\"{id}\"")]).unwrap_or_default();
      xml_element(&theorem, "tags", &[]).unwrap_or_default()
    })
    .collect();
  assert_eq!(tags, [
    r#"<tags><tag>Case 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Case 1</tag></tags>"#,
    r#"<tags><tag>Case 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Case 1</tag></tags>"#,
    r#"<tags><tag>Lemma 2</tag><tag role="refnum">2</tag><tag role="typerefnum">Lemma 2</tag></tags>"#,
    r#"<tags><tag>Case 2</tag><tag role="refnum">2</tag><tag role="typerefnum">Case 2</tag></tags>"#,
    r#"<tags><tag>Lemma 3</tag><tag role="refnum">3</tag><tag role="typerefnum">Lemma 3</tag></tags>"#,
  ]);
}

/// The `<tags>` of each `<equation>` in document order, `""` for an equation without.
fn equation_tags(xml: &str) -> Vec<String> {
  xml
    .match_indices("<equation ")
    .map(|(at, _)| {
      let equation = xml_element(&xml[at..], "equation", &[]).unwrap_or_default();
      let head = &equation[..equation.find("<Math").unwrap_or(equation.len())];
      xml_element(head, "tags", &[]).unwrap_or_default()
    })
    .collect()
}

/// amsmath's multline numbered as `{equation}`: `\tag{29}` replaces the number, `\notag` drops it (all three were
/// numbered (1), (2), (3)). Witness hep-th0210191. Repro alignment/multline_notag_then_tag.
#[test]
fn multline_notag_then_tag() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/alignment/multline_notag_then_tag.tex");
  let (stderr, xml) = convert_with(tex, Some(AR5IV));
  assert_eq!(
    (error_count(&stderr), warning_count(&stderr)),
    (0, 0),
    "{stderr}"
  );
  assert_eq!(equation_tags(&xml), [
    r#"<tags><tag>(29)</tag><tag role="refnum">29</tag></tags>"#,
    r#"<tags><tag>(30)</tag><tag role="refnum">30</tag></tags>"#,
    "",
  ]);
}

/// A tagged multline never steps the counter: `\tag*{M\theequation}` is "M0", `\tag{Q}` "(Q)" (they were (1) and
/// (2)). Repro alignment-bindings/multline_tag_replaces_the_number.
#[test]
fn multline_tag_replaces_the_number() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/alignment-bindings/multline_tag_replaces_the_number.tex"
  );
  let (stderr, xml) = convert_with(tex, Some(AR5IV));
  assert_eq!(
    (error_count(&stderr), warning_count(&stderr)),
    (0, 0),
    "{stderr}"
  );
  assert_eq!(equation_tags(&xml), [
    r#"<tags><tag>M0</tag><tag role="refnum">M0</tag></tags>"#,
    r#"<tags><tag>(Q)</tag><tag role="refnum">Q</tag></tags>"#,
  ]);
}

/// A text-mode `{IEEEeqnarrayboxm}` is inline math of its own, so "and more." stays text after it (it was an
/// `XMText` inside the box's Math). Repro alignment-bindings/ieeeeqnarrayboxm_in_text_keeps_its_math.
#[test]
fn ieeeeqnarrayboxm_in_text_keeps_its_math() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/alignment-bindings/ieeeeqnarrayboxm_in_text_keeps_its_math.tex"
  );
  assert_elements(tex, AR5IV, (0, 0), &[(
    "p",
    "p1.1",
    r#"<p xml:id="p1.1">Text <Math mode="inline" tex="\begin{IEEEeqnarraybox}{c}\displaystyle z\end{IEEEeqnarraybox}" text="Array[[z]]" xml:id="p1.m1"><XMath xml:id="p1.m1.1"><XMArray role="ARRAY" vattach="middle" xml:id="p1.m1.1.1"><XMRow xml:id="p1.m1.1.1.1"><XMCell align="center" xml:id="p1.m1.1.1.1.1"><XMTok font="italic" role="UNKNOWN">z</XMTok></XMCell></XMRow></XMArray></XMath></Math> and more.</p>"#,
  )]);
}

/// amsppt.sty:810-818: `\footnote`'s text is no `\long` argument — a runaway one stops at the paragraph's end, 1
/// error ("Paragraph ended before \footnote was complete"), the note dropped and both later paragraphs kept outside
/// any note (it took them, 2 errors). Repro amstex/amsppt_footnote_argument_runs_past_par.
#[test]
fn amsppt_footnote_argument_runs_past_par() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/amstex/amsppt_footnote_argument_runs_past_par.tex"
  );
  let xml = assert_elements_with(
    tex,
    AR5IV,
    (1, 0),
    &["Paragraph ended before \\footnote was complete"],
    &[
      (
        "para",
        "id1",
        r#"<para xml:id="id1"><p xml:id="id1.1">Text</p></para>"#,
      ),
      (
        "para",
        "id2",
        r#"<para xml:id="id2"><p xml:id="id2.1">Later paragraph.</p></para>"#,
      ),
      (
        "para",
        "id3",
        r#"<para xml:id="id3"><p xml:id="id3.1">Last paragraph.</p></para>"#,
      ),
    ],
  );
  assert!(!xml.contains("<note"), "{xml}");
}

/// combine.cls:85-88, :105-111: `[book]` loads book, so `\chapter` is the class's (it read the chapterless
/// stand-in, with its Warning). Witness 2310.14794 (an article combine). Repro
/// class-bindings/combine_book_option_loads_book.
#[test]
fn combine_book_option_loads_book() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/combine_book_option_loads_book.tex"
  );
  // (the autoref names end in a no-break space, `~` here)
  let chapter = r#"<chapter inlist="toc" xml:id="Ch1"><tags><tag>Chapter 1</tag><tag role="autoref">chapter~1<text xml:id="Ch1.1"/></tag><tag role="refnum">1</tag><tag role="typerefnum">Chapter 1</tag></tags><title><tag close=" ">Chapter 1</tag>Start</title><toctitle><tag close=" ">1</tag>Start</toctitle><section inlist="toc" xml:id="Ch1.S1"><tags><tag>1.1</tag><tag role="autoref">section~1.1<text xml:id="Ch1.S1.1"/></tag><tag role="refnum">1.1</tag><tag role="typerefnum">§1.1</tag></tags><title><tag close=" ">1.1</tag>Part</title><para xml:id="Ch1.S1.p1"><p xml:id="Ch1.S1.p1.1">Text.</p></para></section></chapter>"#.replace('~', "\u{a0}");
  assert_elements(tex, AR5IV, (0, 0), &[("chapter", "Ch1", &chapter)]);
}

/// endnotes.sty:271-282: `\theenmark` is the mark of the note last made, not the counter where it is read ("The last
/// mark is 17." both times). Witness cms-noteref-demo. Repro package-bindings/endnotes_theenmark_is_the_note_mark.
#[test]
fn endnotes_theenmark_is_the_note_mark() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/package-bindings/endnotes_theenmark_is_the_note_mark.tex"
  );
  assert_elements(tex, AR5IV, (0, 0), &[
    (
      "p",
      "p1.1",
      r#"<p xml:id="p1.1">One<note inlist="ent" mark="1" role="endnote" xml:id="endnote1"><tags><tag><text xml:id="endnote1.1"><sup xml:id="endnote1.1.1">1</sup></text></tag><tag role="refnum">1</tag><tag role="typerefnum">endnote 1</tag></tags>A.</note> two<note inlist="ent" mark="2" role="endnote" xml:id="endnote2"><tags><tag><text xml:id="endnote2.1"><sup xml:id="endnote2.1.1">2</sup></text></tag><tag role="refnum">2</tag><tag role="typerefnum">endnote 2</tag></tags>B.</note>The last mark is 2.</p>"#,
    ),
    (
      "p",
      "p2.1",
      r#"<p xml:id="p2.1">Given<note inlist="ent" mark="7" role="endnote" xml:id="endnotex1"><tags><tag><text xml:id="endnotex1.1"><sup xml:id="endnotex1.1.1">7</sup></text></tag><tag role="refnum">7</tag><tag role="typerefnum">endnote 7</tag></tags>C.</note>The given mark is 7.</p>"#,
    ),
  ]);
}
