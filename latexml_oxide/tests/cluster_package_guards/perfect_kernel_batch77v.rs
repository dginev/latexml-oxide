//! Red/green guards for batch 77v (quality batch 4): package bindings that lost authored text or mislabelled it —
//! thm-restate's `restate=` key, todonotes' option list, KOMA's bibliography preamble, sn-jnl's table rules, theorem
//! counters over a shared one and `\newcounter` over an existing `\the<ctr>`, multline's `\tag`/`\notag`, a text-mode
//! `{IEEEeqnarrayboxm}`, amsppt's non-`\long` `\footnote`, combine's class options and endnotes' `\theenmark`; a
//! second `\newtheorem` refused, a binding's theorem guess yielding to the document's, tac's `\def`-style `\newtheorem`
//! and ntheorem's `\renewtheorem` of an undefined name. Each guard asserts the diagnostics by count and the authored
//! content in its whole element.
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

/// amsppt.sty:807-815: `\footnote`'s text is no `\long` argument — a runaway one stops at the paragraph's end, 1
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

/// A `\tag` in `{multline*}` is the equation's tag, as in `{equation*}`: "(Z)", the untagged one none, the counter not
/// stepped (it was dropped). Repro alignment-bindings/multline_star_tag_is_kept.
#[test]
fn multline_star_tag_is_kept() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/alignment-bindings/multline_star_tag_is_kept.tex"
  );
  let (stderr, xml) = convert_with(tex, Some(AR5IV));
  assert_eq!(
    (error_count(&stderr), warning_count(&stderr)),
    (0, 0),
    "{stderr}"
  );
  assert_eq!(equation_tags(&xml), [
    r#"<tags><tag>(Z)</tag><tag role="refnum">Z</tag></tags>"#,
    "",
    r#"<tags><tag>(1)</tag><tag role="refnum">1</tag></tags>"#,
  ]);
}

/// latex.ltx `\@nthm` refuses a theorem the document declared already ("Command \prop already defined", an Info as
/// every "already defined. Or name" message): the first, sharing theorem's counter, is kept — "Theorem 1, Prop 2,
/// Theorem 3, Prop 4" (it was replaced: "Prop 1", "Theorem 2", "Prop 2"). Repro
/// macro-state/newtheorem_twice_keeps_the_first.
#[test]
fn newtheorem_twice_keeps_the_first() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/macro-state/newtheorem_twice_keeps_the_first.tex"
  );
  let (stderr, xml) = convert_with(tex, Some(AR5IV));
  assert_eq!(
    (error_count(&stderr), warning_count(&stderr)),
    (0, 0),
    "{stderr}"
  );
  assert_eq!(
    stderr
      .lines()
      .filter(|l| l.starts_with("Info:") && l.contains("Command \\prop already defined"))
      .count(),
    1,
    "{stderr}"
  );
  let tags: Vec<String> = ["Thmtheorem1", "Thmtheorem2", "Thmtheorem3", "Thmtheorem4"]
    .iter()
    .map(|id| {
      let theorem =
        xml_element(&xml, "theorem", &[&format!("xml:id=\"{id}\"")]).unwrap_or_default();
      xml_element(&theorem, "tags", &[]).unwrap_or_default()
    })
    .collect();
  assert_eq!(tags, [
    r#"<tags><tag>Theorem 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Theorem 1</tag></tags>"#,
    r#"<tags><tag>Prop 2</tag><tag role="refnum">2</tag><tag role="typerefnum">Prop 2</tag></tags>"#,
    r#"<tags><tag>Theorem 3</tag><tag role="refnum">3</tag><tag role="typerefnum">Theorem 3</tag></tags>"#,
    r#"<tags><tag>Prop 4</tag><tag role="refnum">4</tag><tag role="typerefnum">Prop 4</tag></tags>"#,
  ]);
}

/// amsppt.sty:807-815, :798-806: a footnote in a formula reverts in its `tex=` as the author's `\footnote{…}` /
/// `\footnotetext{…}` (it was the binding's `\lx@amsppt@footnote`). Repro
/// amstex/amsppt_footnote_in_math_reverts_as_footnote.
#[test]
fn amsppt_footnote_in_math_reverts_as_footnote() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/amstex/amsppt_footnote_in_math_reverts_as_footnote.tex"
  );
  let (stderr, xml) = convert_with(tex, Some(AR5IV));
  assert_eq!(
    (error_count(&stderr), warning_count(&stderr)),
    (0, 0),
    "{stderr}"
  );
  let texs: Vec<&str> = xml
    .match_indices("<Math ")
    .filter_map(|(at, _)| {
      let head = &xml[at..at + xml[at..].find('>').unwrap_or(0)];
      let start = head.find(" tex=\"")? + 6;
      Some(&head[start..start + head[start..].find('"')?])
    })
    .collect();
  assert_eq!(texs, [
    r"a=b\text{\footnote{In display.}}",
    r"c=d\text{\footnotetext{Its text.}}",
  ]);
  assert!(!xml.contains("lx@amsppt"), "{xml}");
}

/// endnotes.sty:278-283: `\endnote[4]` sets the counter to 4 in a group and marks the note `\theendnote`, so a `\roman`
/// one is "iv" at the note and as `\theenmark` (both were "4"); the next note steps on from 1, "ii". Repro
/// package-bindings/endnotes_given_number_is_formatted.
#[test]
fn endnotes_given_number_is_formatted() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/package-bindings/endnotes_given_number_is_formatted.tex"
  );
  assert_elements(tex, AR5IV, (0, 0), &[(
    "p",
    "p1.1",
    r#"<p xml:id="p1.1">One<note inlist="ent" mark="i" role="endnote" xml:id="endnote1"><tags><tag><text xml:id="endnote1.1"><sup xml:id="endnote1.1.1">i</sup></text></tag><tag role="refnum">i</tag><tag role="typerefnum">endnote i</tag></tags>A.</note> two<note inlist="ent" mark="iv" role="endnote" xml:id="endnotex1"><tags><tag><text xml:id="endnotex1.1"><sup xml:id="endnotex1.1.1">iv</sup></text></tag><tag role="refnum">iv</tag><tag role="typerefnum">endnote iv</tag></tags>B.</note> mark: iv. Three<note inlist="ent" mark="ii" role="endnote" xml:id="endnote2"><tags><tag><text xml:id="endnote2.1"><sup xml:id="endnote2.1.1">ii</sup></text></tag><tag role="refnum">ii</tag><tag role="typerefnum">endnote ii</tag></tags>C.</note></p>"#,
  )]);
}

/// combine.cls:31-50: `colclass=` is read before `\ProcessOptions`, so `[11pt,colclass=amsart]` passes 11pt to
/// amsart (it went to article). Repro class-bindings/combine_colclass_gets_the_class_options.
#[test]
fn combine_colclass_gets_the_class_options() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/combine_colclass_gets_the_class_options.tex"
  );
  assert_elements(tex, AR5IV, (0, 0), &[(
    "p",
    "p1.1",
    r#"<p xml:id="p1.1">amsart takes 11pt: yes. article: no.</p>"#,
  )]);
}

/// combine.cls:31-50, :85-88: `[colclass=report,book]` typesets in book, the class option run after `colclass=` was
/// read (it loaded report). Repro class-bindings/combine_class_option_after_colclass.
#[test]
fn combine_class_option_after_colclass() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/combine_class_option_after_colclass.tex"
  );
  assert_elements(tex, AR5IV, (0, 0), &[(
    "p",
    "p1.1",
    r#"<p xml:id="p1.1">Class: book. Book: yes.</p>"#,
  )]);
}

/// Every `<tags>` element of the document's theorems, in document order.
fn theorem_tags(xml: &str) -> Vec<String> {
  xml
    .match_indices("<theorem ")
    .map(|(at, _)| xml_element(&xml[at..], "tags", &[]).unwrap_or_default())
    .collect()
}

/// The number of LaTeX's "Command \<name> already defined" messages (an Info, `is_typesetting_only_message`).
fn already_defined(stderr: &str, name: &str) -> usize {
  let needle = format!("Command \\{name} already defined");
  stderr.lines().filter(|l| l.contains(&needle)).count()
}

/// A fallback class's theorem guess (OmniBus `theorem_preload`, run by the first undeclared `\begin{proof}` with the
/// document's origin) is no declaration of the document's: it declares through `\lx@guess@newtheorem`, which never
/// claims the name, so the author's later `\newtheorem{note}{Notation}` / `\newtheorem{claim}{Claim}[section]` replace
/// the guess ("Note 1.1", "Claim 1" and two "already defined" messages under r2). Repro
/// macro-state/newtheorem_guess_omnibus_yields.
#[test]
fn omnibus_guess_yields_to_document_newtheorem() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/macro-state/newtheorem_guess_omnibus_yields.tex"
  );
  let (stderr, xml) = convert_with(tex, Some(AR5IV));
  // The one warning: no binding for the unknown class (OmniBus is used).
  assert_eq!(
    (error_count(&stderr), warning_count(&stderr)),
    (0, 1),
    "{stderr}"
  );
  // The author's {note} claims the name after the guess: the second `\newtheorem{note}{Again}` is refused (pdflatex
  // "Command \note already defined"), the guessed {claim} is replaced without one.
  assert_eq!(
    (
      already_defined(&stderr, "note"),
      already_defined(&stderr, "claim")
    ),
    (1, 0),
    "{stderr}"
  );
  assert_eq!(theorem_tags(&xml), [
    r#"<tags><tag>Notation 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Notation 1</tag></tags>"#,
    r#"<tags><tag>Notation 2</tag><tag role="refnum">2</tag><tag role="typerefnum">Notation 2</tag></tags>"#,
    r#"<tags><tag>Claim 1.1</tag><tag role="refnum">1.1</tag><tag role="typerefnum">Claim 1.1</tag></tags>"#,
  ]);
}

/// The neurips binding's `\AtBeginDocument` theorem set is a guess (the shipped neurips_2024.sty declares none): the
/// document's `\newtheorem{lemma}{Lemma}`, `\newtheorem{remark}{Observation}` replace it, as pdflatex prints ("Lemma
/// 1.1", "Remark 1.2" under r2). Repro package-bindings/neurips_guess_theorem_yields.
#[test]
fn neurips_guess_yields_to_document_newtheorem() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/package-bindings/neurips_guess_theorem_yields.tex"
  );
  let (stderr, xml) = convert_with(tex, Some(AR5IV));
  assert_eq!(
    (error_count(&stderr), warning_count(&stderr)),
    (0, 0),
    "{stderr}"
  );
  assert_eq!(
    (
      already_defined(&stderr, "lemma"),
      already_defined(&stderr, "remark")
    ),
    (0, 0),
    "{stderr}"
  );
  assert_eq!(theorem_tags(&xml), [
    r#"<tags><tag>Lemma 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Lemma 1</tag></tags>"#,
    r#"<tags><tag>Observation 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Observation 1</tag></tags>"#,
  ]);
}

/// tac.cls's `\newtheorem`/`\newtheoremrm` are `\def`s that define `\<name>` outright (0708.1921's tac.cls:409-418):
/// a second declaration of `thm` replaces the first, never refused ("Theorem 2", "Theorem 3" and two "already
/// defined" messages under r2; pdflatex "0.2. Proposition.", "0.3. Remark."). Repro
/// class-bindings/tac_newtheorem_redeclares.
#[test]
fn tac_newtheorem_redeclares() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/tac_newtheorem_redeclares.tex"
  );
  let (stderr, xml) = convert_with(tex, Some(AR5IV));
  assert_eq!(
    (error_count(&stderr), warning_count(&stderr)),
    (0, 0),
    "{stderr}"
  );
  assert_eq!(already_defined(&stderr, "thm"), 0, "{stderr}");
  let titles: Vec<String> = theorem_tags(&xml)
    .iter()
    .map(|tags| xml_element(tags, "tag", &[]).unwrap_or_default())
    .collect();
  assert_eq!(titles, [
    "<tag>Theorem 1</tag>",
    "<tag>Proposition 2</tag>",
    "<tag>Remark 3</tag>"
  ]);
}

/// ntheorem.sty:808-814 `\@renewtheorem`: `\renewtheorem` of a name no theorem has raises "Theorem keyword Rem
/// undefined", then declares it (silent under r2). Repro package-bindings/ntheorem_renewtheorem_undefined.
#[test]
fn ntheorem_renewtheorem_of_undefined_errs() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/package-bindings/ntheorem_renewtheorem_undefined.tex"
  );
  let (stderr, xml) = convert_with(tex, Some(AR5IV));
  assert_eq!(
    (error_count(&stderr), warning_count(&stderr)),
    (1, 0),
    "{stderr}"
  );
  assert_eq!(
    stderr
      .lines()
      .filter(|l| l.starts_with("Error:") && l.contains("Theorem keyword Rem undefined"))
      .count(),
    1,
    "{stderr}"
  );
  assert_eq!(theorem_tags(&xml), [
    r#"<tags><tag>Remark 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Remark 1</tag></tags>"#,
  ]);
}

/// llncs.cls declares {theorem}; LaTeX refuses the document's `\newtheorem{theorem}{Theorem}[section]` and prints
/// "Theorem 1.", but a binding's declaration yields to the document's (its set is in general more than its class's),
/// so ours is "Theorem 1.1": OXIDIZED_DESIGN_DIVERGENCES #783. The document's second `\newtheorem{mylemma}` is
/// refused, as in LaTeX. Pins the divergence (green before and after 77v r3). Repro
/// class-bindings/llncs_document_newtheorem_over_class.
#[test]
fn llncs_document_newtheorem_replaces_the_class() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/llncs_document_newtheorem_over_class.tex"
  );
  let (stderr, xml) = convert_with(tex, Some(AR5IV));
  assert_eq!(
    (error_count(&stderr), warning_count(&stderr)),
    (0, 0),
    "{stderr}"
  );
  assert_eq!(
    (
      already_defined(&stderr, "theorem"),
      already_defined(&stderr, "mylemma")
    ),
    (0, 1),
    "{stderr}"
  );
  assert_eq!(theorem_tags(&xml), [
    r#"<tags><tag>Theorem 1.1</tag><tag role="refnum">1.1</tag><tag role="typerefnum">Theorem 1.1</tag></tags>"#,
    r#"<tags><tag>Lemma 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Lemma 1</tag></tags>"#,
  ]);
}
