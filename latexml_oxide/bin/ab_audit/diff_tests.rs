//! Unit tests of `diff.rs`: the flagged rows on tiny inline XML and log fixtures.

use super::*;

fn input(xml: &str, log: &str) -> DocInput {
  DocInput {
    present: true,
    xml:     Some(xml.as_bytes().to_vec()),
    log:     Some(log.to_string()),
    meta:    None,
    stderr:  None,
  }
}

const OK: &str = "Status:conversion:0\n";

fn doc(body: &str) -> String {
  format!(r#"<document xmlns="http://dlmf.nist.gov/LaTeXML">{body}</document>"#)
}

#[test]
fn a_one_character_mailto_href_change_is_lost_text() {
  let a = doc(r#"<para><p><ref href="mailto:ann_able@a.edu">ann_able@a.edu</ref></p></para>"#);
  let b = doc(r#"<para><p><ref href="mailto:a">ann_able@a.edu</ref></p></para>"#);
  let row = compare("x", &input(&a, OK), &input(&b, OK));
  assert_eq!(row.tier, Some(Tier::Regression), "{row:?}");
  assert!(
    row.flags.contains(&"lost-text:ref@href".to_string()),
    "{row:?}"
  );
}

#[test]
fn a_lost_graphics_element_with_unchanged_text_is_flagged() {
  let a = doc(
    r#"<figure><graphics graphic="p1"/><graphics graphic="p2"/><caption>Panels</caption></figure>"#,
  );
  let b = doc(r#"<figure><graphics graphic="p1"/><caption>Panels</caption></figure>"#);
  let row = compare("x", &input(&a, OK), &input(&b, OK));
  assert!(
    row.flags.contains(&"lost-elements:graphics".to_string()),
    "{row:?}"
  );
  assert_eq!(row.tier, Some(Tier::Regression));
  assert!(row.cells[5].contains("graphics:-1"), "{row:?}");
}

#[test]
fn a_lost_space_between_two_words_is_flagged() {
  let a = doc(r#"<para><p>see <text font="italic">this</text> and</p></para>"#);
  let b = doc(r#"<para><p>see<text font="italic">this</text> and</p></para>"#);
  let row = compare("x", &input(&a, OK), &input(&b, OK));
  assert!(row.flags.contains(&"lost-text:p".to_string()), "{row:?}");
}

#[test]
fn a_creator_whose_contact_empties_is_a_regression() {
  let a = doc(
    r#"<creator role="author"><personname>Ann</personname><contact role="email">a@b.c</contact></creator>"#,
  );
  let b =
    doc(r#"<creator role="author"><personname>Ann</personname><contact role="email"/></creator>"#);
  let row = compare("x", &input(&a, OK), &input(&b, OK));
  assert!(
    row.flags.contains(&"contact-emptied".to_string()),
    "{row:?}"
  );
  assert_eq!(row.tier, Some(Tier::Regression));
}

#[test]
fn an_unparsable_xml_flags_on_either_side_and_in_the_audit() {
  let good = doc("<para><p>x</p></para>");
  let bad = "<document><p>x</document>";
  let row = compare("x", &input(&good, OK), &input(bad, OK));
  assert!(
    row.flags.contains(&"xml-unparsable:B".to_string()),
    "{row:?}"
  );
  assert_eq!(row.tier, Some(Tier::Regression));
  let row = compare("x", &input(bad, OK), &input(&good, OK));
  assert!(
    row.flags.contains(&"xml-unparsable:A".to_string()),
    "{row:?}"
  );
  let row = audit("x", &input(bad, OK));
  assert_eq!(row.flags, vec!["xml-unparsable".to_string()]);
  assert_eq!(row.tier, Some(Tier::Regression));
}

#[test]
fn a_problem_on_both_sides_ranks_after_what_b_made_worse() {
  let bad = "<document><p>x</document>";
  let good = doc("<para><p>x</p></para>");
  let both = compare("x", &input(bad, OK), &input(bad, OK));
  assert_eq!(both.flags, vec!["xml-unparsable:both".to_string()]);
  assert_eq!(both.tier, Some(Tier::Regression));
  let worse = compare("x", &input(&good, OK), &input(bad, OK));
  assert!(worse.severity() < both.severity());
}

#[test]
fn lost_attributes_markup_and_a_space_before_math_are_review_not_change() {
  // The review's repro (/tmp/rev_p0/m): a lost `name="E-mail: "`, a lost class, a lost space before Math,
  // a lost bold font and one added XMWrap. Round 1 called it `change math-changed`.
  let a = doc(
    r#"<creator role="author"><personname>Ann</personname><contact role="email" name="E-mail: ">a@b.c</contact></creator><para><p class="ltx_centering">see <Math tex="a"><XMath><XMTok>a</XMTok></XMath></Math> and <text font="bold">big</text></p></para>"#,
  );
  let b = doc(
    r#"<creator role="author"><personname>Ann</personname><contact role="email">a@b.c</contact></creator><para><p>see<Math tex="a"><XMath><XMWrap><XMTok>a</XMTok></XMWrap></XMath></Math> and <text>big</text></p></para>"#,
  );
  let row = compare("x", &input(&a, OK), &input(&b, OK));
  for flag in [
    "lost-text:contact@name",
    "space-lost:p",
    "attrs-changed:p@class",
    "attrs-changed:text@font",
  ] {
    assert!(row.flags.contains(&flag.to_string()), "{flag}: {row:?}");
  }
  // The lost `name` is one flag (text), not also an attribute change; the space is not lost words.
  assert!(
    !row
      .flags
      .contains(&"attrs-changed:contact@name".to_string()),
    "{row:?}"
  );
  assert!(!row.flags.contains(&"lost-text:p".to_string()), "{row:?}");
  assert_eq!(row.tier, Some(Tier::Regression), "{row:?}");
}

#[test]
fn a_footnote_or_index_mark_beside_a_word_loses_no_text() {
  // /tmp/rev_p0r2/f7: round 2 called both `lost-text:p`.
  for (a, b) in [
    (
      "<para><p>word more</p></para>",
      r#"<para><p>word<note role="footnote">n</note> more</p></para>"#,
    ),
    (
      "<para><p>Theorem holds</p></para>",
      "<para><p><indexmark><indexphrase>thm</indexphrase></indexmark>Theorem holds</p></para>",
    ),
  ] {
    let row = compare("x", &input(&doc(a), OK), &input(&doc(b), OK));
    assert!(
      row
        .flags
        .iter()
        .all(|f| !f.starts_with("lost-text") && !f.starts_with("space-lost")),
      "{row:?}"
    );
  }
}

#[test]
fn an_unparsable_status_is_a_regression() {
  let x = doc("<para><p>x</p></para>");
  for log in [
    "Status:conversion:x7\n",
    "Status:conversion:\n",
    "Fatal:a:b c\nStatus:conversion:300\n",
  ] {
    let row = compare("x", &input(&x, OK), &input(&x, log));
    assert!(
      row.flags.contains(&"status-unparsable:B".to_string()),
      "{log:?}: {row:?}"
    );
    assert_eq!(row.tier, Some(Tier::Regression));
  }
}

#[test]
fn in_the_audit_the_status_alone_raises_the_severity() {
  let x = doc("<para><p>x</p></para>");
  let row = audit("x", &input(&x, "Status:conversion:3\n"));
  for flag in ["fatal", "status-diag-mismatch"] {
    assert!(row.flags.contains(&flag.to_string()), "{flag}: {row:?}");
  }
  assert_eq!(row.tier, Some(Tier::Regression));
  let row = audit("x", &input(&x, "Status:conversion:2\n"));
  assert!(row.flags.contains(&"errors".to_string()), "{row:?}");
}

#[test]
fn mid_line_errors_and_fatals_are_regressions_and_warnings_review() {
  let x = doc("<para><p>x</p></para>");
  let mid = |log: &str| compare("x", &input(&x, OK), &input(&x, log));
  let r = mid("(Loading foo.sty... Error:undefined:\\x bad\nStatus:conversion:0\n");
  for flag in ["error-midline:B", "status-diag-mismatch:B"] {
    assert!(r.flags.contains(&flag.to_string()), "{flag}: {r:?}");
  }
  assert_eq!(r.tier, Some(Tier::Regression));
  // /tmp/rev_p0r3 T/midfatal and T/midwarn.
  let r = mid("(Loading foo) Fatal:timeout:wallclock\nStatus:conversion:0\n");
  assert!(r.flags.contains(&"fatal-midline:B".to_string()), "{r:?}");
  let r = mid("(Loading foo) Warning:missing:x y\nStatus:conversion:0\n");
  assert_eq!(r.flags, vec!["diag-midline:B".to_string()], "{r:?}");
  assert_eq!(r.tier, Some(Tier::Review));
  let r = mid("see Error: my own typeout\nStatus:conversion:0\n");
  assert_eq!(r.tier, Some(Tier::Same), "{r:?}");
}

#[test]
fn a_fixed_undefined_macro_is_a_change_not_a_regression() {
  // /tmp/rev_p0r2/f3 errfix.
  let a = doc(r#"<para><p>see <ERROR class="undefined">\foo</ERROR> bar</p></para>"#);
  let b = doc("<para><p>see baz bar</p></para>");
  let row = compare("x", &input(&a, OK), &input(&b, OK));
  assert!(
    row.flags.contains(&"lost-text:ERROR".to_string()),
    "{row:?}"
  );
  assert_eq!(rank("lost-text:ERROR").0, Tier::Change);
  assert_eq!(rank("attrs-changed:ERROR@class").0, Tier::Change);
  assert!(row.tier >= Some(Tier::Review), "{row:?}");
}

#[test]
fn renumbered_internal_links_and_pgf_names_are_not_changes() {
  // /tmp/rev_p0r2/f2.
  let head = r#"<document xmlns="http://dlmf.nist.gov/LaTeXML" xmlns:svg="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink">"#;
  let pair = [
    (
      r##"<para xml:id="S1.p1"><p>see <ref href="#S1.p1">here</ref></p></para>"##,
      r##"<para xml:id="S1.p2"><p>see <ref href="#S1.p2">here</ref></p></para>"##,
    ),
    (
      r##"<picture><svg:svg><svg:clipPath id="pgfcp1"><svg:path d="M 0 0"/></svg:clipPath><svg:g clip-path="url(#pgfcp1)"><svg:use xlink:href="#pgfpat1"/></svg:g></svg:svg></picture>"##,
      r##"<picture><svg:svg><svg:clipPath id="pgfcp2"><svg:path d="M 0 0"/></svg:clipPath><svg:g clip-path="url(#pgfcp2)"><svg:use xlink:href="#pgfpat2"/></svg:g></svg:svg></picture>"##,
    ),
    (
      r##"<para xml:id="p1"><p>x</p><rdf about="#p1" aboutidref="p1" property="dc:x" content="y"/></para>"##,
      r##"<para xml:id="p2"><p>x</p><rdf about="#p2" aboutidref="p2" property="dc:x" content="y"/></para>"##,
    ),
  ];
  for (a, b) in pair {
    let row = compare(
      "x",
      &input(&format!("{head}{a}</document>"), OK),
      &input(&format!("{head}{b}</document>"), OK),
    );
    assert_eq!(row.flags, vec!["ids-only".to_string()], "{a}: {row:?}");
  }
}

#[test]
fn svg_geometry_within_tolerance_is_equal_and_beyond_it_a_change() {
  let svg = |w: &str, d: &str, extra: &str| {
    doc(&format!(
      r#"<picture width="{w}" height="50"><svg:svg xmlns:svg="http://www.w3.org/2000/svg" width="{w}" height="50"><svg:path d="{d}" {extra}/></svg:svg></picture>"#
    ))
  };
  let base = svg("100", "M 0 0 L 100 50", "");
  let row = |b: &str| compare("x", &input(&base, OK), &input(b, OK));
  // Within the tolerance: equal, canonically equivalent.
  let r = row(&svg("100.04", "M 0 0 L 100.01 50", ""));
  assert_eq!(r.flags, vec!["equivalent".to_string()], "{r:?}");
  // Beyond it: a change.
  let r = row(&svg("100", "M 0 0 L 90 50", ""));
  assert_eq!(r.flags, vec!["geometry-changed".to_string()], "{r:?}");
  assert_eq!(r.tier, Some(Tier::Change));
  // /tmp/rev_p0r3 g1 (a canvas shrunk to a dot), g2 (an emptied path), g3 (a path collapsed to a point):
  // regressions.
  for b in [
    svg("0.04", "M 0 0 L 100 50", ""),
    svg("100", "", ""),
    svg("100", "M 0 0 L 0 0", ""),
  ] {
    let r = row(&b);
    assert!(r.flags.contains(&"geometry-collapsed".to_string()), "{r:?}");
    assert_eq!(r.tier, Some(Tier::Regression));
  }
  // g4, g5: small values compare relatively, so 0.01 → 0.05 and 0.04 → 0 are changes, not equal.
  let m = |v: &str| {
    doc(&format!(
      r#"<picture width="100" height="50"><svg:svg xmlns:svg="http://www.w3.org/2000/svg" width="100" height="50"><svg:g transform="matrix({v} 0 0 {v} 0 0)"/></svg:svg></picture>"#
    ))
  };
  let r = compare("x", &input(&m("0.01"), OK), &input(&m("0.05"), OK));
  assert_eq!(r.flags, vec!["geometry-changed".to_string()], "{r:?}");
  let r = compare(
    "x",
    &input(&svg("100", "M 0 0 L 100 50", r#"stroke-width="0.04""#), OK),
    &input(&svg("100", "M 0 0 L 100 50", r#"stroke-width="0""#), OK),
  );
  assert_eq!(r.flags, vec!["geometry-changed".to_string()], "{r:?}");
  assert!(near(0.01, 0.014) && !near(0.01, 0.05) && near(100.0, 100.05) && !near(100.0, 100.06));
}

#[test]
fn a_run_split_off_with_its_punctuation_is_moved_not_lost() {
  // /tmp/rev_p0r2 f3 text2verb: `foo,` in one class became `foo` (verbatim) + `,` (p).
  let a = doc(r#"<para><p>call <text font="typewriter">foo</text>, then</p></para>"#);
  let b = doc("<para><p>call <verbatim>foo</verbatim>, then</p></para>");
  let r = compare("x", &input(&a, OK), &input(&b, OK));
  assert!(r.flags.contains(&"moved-text:p".to_string()), "{r:?}");
  assert!(!r.flags.iter().any(|f| f.starts_with("lost-text")), "{r:?}");
  // realloss: the word is gone; 64b worldsee: a lost space joins two words.
  let b = doc("<para><p>call <verbatim></verbatim>, then</p></para>");
  assert!(
    compare("x", &input(&a, OK), &input(&b, OK))
      .flags
      .contains(&"lost-text:p".to_string())
  );
  let (a, b) = (
    doc("<para><p>world see</p></para>"),
    doc("<para><p>worldsee</p></para>"),
  );
  let r = compare("x", &input(&a, OK), &input(&b, OK));
  assert!(r.flags.contains(&"lost-text:p".to_string()), "{r:?}");
  assert_eq!(r.tier, Some(Tier::Regression));
}

#[test]
fn numbers_jittering_in_text_are_review_not_lost() {
  // /tmp/rev_p0r3 jit (mhchem, zx-calculus in s147 → s148).
  let m = |v: &str| {
    doc(&format!(
      r#"<para><p><Math tex="\kern {v}pt x"/></p></para>"#
    ))
  };
  let r = compare("x", &input(&m("3.80254"), OK), &input(&m("3.80252"), OK));
  assert!(
    r.flags.contains(&"text-jitter:Math@tex".to_string()),
    "{r:?}"
  );
  assert_eq!(r.tier, Some(Tier::Review), "{r:?}");
  let r = compare("x", &input(&m("3.8"), OK), &input(&m("4.8"), OK));
  assert!(r.flags.contains(&"lost-text:Math@tex".to_string()), "{r:?}");
  // mhchem / zx-calculus witnesses pair.
  let r = compare("x", &input(&m("28.04677"), OK), &input(&m("28.04684"), OK));
  assert_eq!(r.flags, vec!["text-jitter:Math@tex".to_string()], "{r:?}");
  // /tmp/rev_p0r5 J1–J4: numbered content in attributes is changed content, not rounding.
  let cases = [
    (
      r#"<para><p>see <ref href="https://arxiv.org/abs/2101.00001">paper</ref></p></para>"#,
      r#"<para><p>see <ref href="https://arxiv.org/abs/2102.00001">paper</ref></p></para>"#,
      "lost-text:ref@href",
    ),
    (
      r#"<bibliography><biblist><bibitem key="Smith2019"><bibblock>S.</bibblock></bibitem></biblist></bibliography>"#,
      r#"<bibliography><biblist><bibitem key="Smith2020"><bibblock>S.</bibblock></bibitem></biblist></bibliography>"#,
      "lost-text:bibitem@key",
    ),
    (
      r#"<para><p>x <graphics graphic="fig1000.png"/></p></para>"#,
      r#"<para><p>x <graphics graphic="fig1001.png"/></p></para>"#,
      "lost-text:graphics@graphic",
    ),
    (
      r#"<para><p><Math tex="2019"/></p></para>"#,
      r#"<para><p><Math tex="2020"/></p></para>"#,
      "lost-text:Math@tex",
    ),
  ];
  for (a, b, flag) in cases {
    let r = compare("x", &input(&doc(a), OK), &input(&doc(b), OK));
    assert!(r.flags.contains(&flag.to_string()), "{flag}: {r:?}");
    assert!(
      !r.flags.iter().any(|f| f.starts_with("text-jitter")),
      "{flag}: {r:?}"
    );
    assert_eq!(r.tier, Some(Tier::Regression), "{flag}");
  }
}

#[test]
fn a_dimension_typeset_from_nowhere_is_a_leak_and_a_bare_number_a_look() {
  let gained = |text: &str| {
    let a = doc("<para><p>Chapter heading, here.</p></para>");
    let b = doc(&format!(
      "<para><p>{text} Chapter heading, here.</p></para>"
    ));
    compare("x", &input(&a, OK), &input(&b, OK))
  };
  // fncychap `24pt 28pt`, latexcheat-esmx `-36.83386pt`, a dotted value `0.70.2`, and /tmp/rev_p0r4 t7
  // `24pt,`, t10 `(24pt)`, t13 `\fontdimen 24pt`: regressions.
  for leaked in [
    "24pt 28pt",
    "-36.83386pt",
    "0.70.2",
    "24pt,",
    "(24pt)",
    "\\fontdimen 24pt",
  ] {
    let r = gained(leaked);
    assert!(
      r.flags.contains(&"leaked-dimension:p".to_string()),
      "{leaked}: {r:?}"
    );
    assert_eq!(r.tier, Some(Tier::Regression), "{leaked}");
  }
  // A bare number (t11 `10`, `2003.`): a look, not a regression.
  for number in ["10", "2003.", "0.5"] {
    let r = gained(number);
    assert!(
      r.flags.contains(&"gained-number:p".to_string()),
      "{number}: {r:?}"
    );
    assert_eq!(r.tier, Some(Tier::Review), "{number}");
  }
  // A dimension amid recovered prose (floatrow-rus, s147 → s148) is reviewed; one in code is the author's.
  let r = gained("Recovered whole sentences with a width of 4pt and many more words here");
  assert!(r.flags.contains(&"gained-number:p".to_string()), "{r:?}");
  let r = compare(
    "x",
    &input(&doc("<verbatim>x</verbatim>"), OK),
    &input(&doc("<verbatim>x 15pt</verbatim>"), OK),
  );
  assert!(
    r.flags
      .iter()
      .all(|f| !f.starts_with("leaked") && !f.starts_with("gained-number")),
    "{r:?}"
  );
  // /tmp/rev_p0r5 L2: the `3` moved into a `ref` is not a lost number, so `24pt` is a leak; L7: `2019,`
  // split into `2019` + `,` is not lost either; L4 `heading,24pt` and L5 `24pt\relax` hold a dimension.
  let pair = |a: &str, b: &str| compare("x", &input(&doc(a), OK), &input(&doc(b), OK));
  for (a, b) in [
    (
      "<para><p>see Section 3 here. Chapter heading</p></para>",
      r#"<para><p>see Section <ref labelref="x">3</ref> here. 24pt Chapter heading</p></para>"#,
    ),
    (
      "<para><p>In 2019, we did it.</p></para>",
      "<para><p>In 2019 , we did it.</p></para><para><p>24pt 28pt</p></para>",
    ),
    (
      "<para><p>Chapter heading, here.</p></para>",
      "<para><p>Chapter heading,24pt here.</p></para>",
    ),
    (
      "<para><p>Chapter heading here.</p></para>",
      "<para><p>Chapter 24pt\\relax heading here.</p></para>",
    ),
  ] {
    let r = pair(a, b);
    assert!(
      r.flags.contains(&"leaked-dimension:p".to_string()),
      "{b}: {r:?}"
    );
  }
  // A number lost for good beside a gained dimension (a changed value) is still a number to look at.
  let r = pair(
    "<table><tabular><tr><td>3 x</td></tr></tabular></table>",
    "<table><tabular><tr><td>4 x 24pt</td></tr></tabular></table>",
  );
  assert!(r.flags.contains(&"gained-number:td".to_string()), "{r:?}");
  assert_eq!(token_number_kind("heading,24pt"), Some(true));
  assert_eq!(token_number_kind("24pt\\relax"), Some(true));
  assert_eq!(number_kind("(24pt),"), Some(true));
  assert_eq!(number_kind("2003."), Some(false));
  assert_eq!(number_kind("24ptx"), None);
  assert_eq!(number_kind("pt"), None);
}

#[test]
fn generated_numbers_and_table_values_are_not_leaks() {
  // /tmp/rev_p0r4 t5: a fixed `\ref` now printing `3`; t6: a table cell gaining `0.5`; an equation tag.
  let r = compare(
    "x",
    &input(
      &doc(r#"<para><p>see <ref labelref="LABEL:x"/> here</p></para>"#),
      OK,
    ),
    &input(
      &doc(r#"<para><p>see <ref labelref="LABEL:x">3</ref> here</p></para>"#),
      OK,
    ),
  );
  assert!(
    r.flags
      .iter()
      .all(|f| !f.starts_with("leaked") && !f.starts_with("gained-number")),
    "{r:?}"
  );
  assert_eq!(r.tier, Some(Tier::Review));
  let r = compare(
    "x",
    &input(
      &doc("<table><tabular><tr><td>a</td><td></td></tr></tabular></table>"),
      OK,
    ),
    &input(
      &doc("<table><tabular><tr><td>a</td><td>0.5</td></tr></tabular></table>"),
      OK,
    ),
  );
  assert!(r.flags.contains(&"gained-number:td".to_string()), "{r:?}");
  assert_eq!(r.tier, Some(Tier::Review));
  let r = compare(
    "x",
    &input(&doc(r#"<equation><tags><tag></tag></tags></equation>"#), OK),
    &input(
      &doc(r#"<equation><tags><tag>(2)</tag></tags></equation>"#),
      OK,
    ),
  );
  assert!(
    r.flags
      .iter()
      .all(|f| !f.starts_with("leaked") && !f.starts_with("gained-number")),
    "{r:?}"
  );
}

#[test]
fn a_changed_prose_number_is_not_jitter() {
  // /tmp/rev_p0r4 j2/j3, t4/t4b: values with fewer than 3 fractional digits are content.
  for (x, y) in [
    ("0.30 and 1.25", "0.31 and 1.27"),
    ("2.5cm", "2.54cm"),
    ("12.34", "12.38"),
    ("0.001", "0.005"),
  ] {
    let a = doc(&format!("<para><p>Value {x} here</p></para>"));
    let b = doc(&format!("<para><p>Value {y} here</p></para>"));
    let r = compare("x", &input(&a, OK), &input(&b, OK));
    assert!(r.flags.contains(&"lost-text:p".to_string()), "{x}: {r:?}");
    assert!(
      !r.flags
        .iter()
        .any(|f| f.starts_with("text-jitter") || f.starts_with("leaked")),
      "{x}: {r:?}"
    );
  }
  // A computed value with 3+ fractional digits on both sides is still rounding.
  let a = doc("<para><p>Value 3.80254pt here</p></para>");
  let b = doc("<para><p>Value 3.80252pt here</p></para>");
  assert_eq!(compare("x", &input(&a, OK), &input(&b, OK)).flags, vec![
    "text-jitter:p".to_string()
  ]);
  assert_eq!(numbers_with_digits("3.80254pt 12.34 2019").1, vec![5, 2, 0]);
}

#[test]
fn mid_line_counts_compare_like_errors() {
  let x = doc("<para><p>x</p></para>");
  let mid = "(Loading foo) Fatal:timeout:wallclock\nStatus:conversion:0\n";
  // /tmp/rev_p0r4 A4/m: fixed by B.
  let r = compare("x", &input(&x, mid), &input(&x, OK));
  assert!(
    r.flags.contains(&"fatal-midline-fixed".to_string()),
    "{r:?}"
  );
  assert!(
    !r.flags.iter().any(|f| f.starts_with("fatal-midline:")),
    "{r:?}"
  );
  // More in B: a regression; the same on both sides: `:both`.
  let two = "(a) Fatal:x:y z\n(b) Fatal:x:y z\nStatus:conversion:0\n";
  let r = compare("x", &input(&x, mid), &input(&x, two));
  assert!(r.flags.contains(&"fatal-midline+".to_string()), "{r:?}");
  assert_eq!(r.tier, Some(Tier::Regression));
  let r = compare("x", &input(&x, mid), &input(&x, mid));
  assert!(r.flags.contains(&"fatal-midline:both".to_string()), "{r:?}");
  // /tmp/rev_p0r4 A5/B5 e: a Fatal appended after an anchored Error.
  let r = compare(
    "x",
    &input(&x, "Error:undefined:\\a x\nStatus:conversion:2\n"),
    &input(
      &x,
      "Error:undefined:\\a x (Loading b.sty Fatal:too_many_errors:100 boom\nStatus:conversion:2\n",
    ),
  );
  assert!(r.flags.contains(&"fatal-midline:B".to_string()), "{r:?}");
  assert_eq!(r.tier, Some(Tier::Regression));
}

#[test]
fn one_boundary_split_in_two_still_shows_a_lost_space() {
  // /tmp/rev_p0r3 s1e: `see <Math/> and` → `see<Math/><Math/> and`.
  let a = doc(r#"<para><p>see <Math tex="a"/> and</p></para>"#);
  let b = doc(r#"<para><p>see<Math tex="a"/><Math tex="b"/> and</p></para>"#);
  let r = compare("x", &input(&a, OK), &input(&b, OK));
  assert!(r.flags.contains(&"space-lost:p".to_string()), "{r:?}");
}

#[test]
fn an_oom_of_the_base_alone_is_fixed() {
  let x = doc("<para><p>x</p></para>");
  let oom = DocInput {
    meta: Some(RunMeta {
      rc: Some(137),
      timed_out: true,
      ..RunMeta::default()
    }),
    ..input(&x, "Fatal:oom:alloc_failed big\nStatus:conversion:3\n")
  };
  let r = compare("x", &oom, &input(&x, OK));
  assert!(r.flags.contains(&"oom-fixed".to_string()), "{r:?}");
  assert!(!r.flags.iter().any(|f| f.starts_with("oom:")), "{r:?}");
}

#[test]
fn a_missing_log_reads_fatals_from_stderr_and_137_with_oom_is_oom() {
  let x = doc("<para><p>x</p></para>");
  let killed = DocInput {
    present: true,
    xml:     None,
    log:     None,
    meta:    Some(RunMeta {
      rc: Some(137),
      timed_out: true,
      ..RunMeta::default()
    }),
    stderr:  Some("\x1b[31mFatal:\x1b[0moom:alloc_failed allocation failed\n".to_string()),
  };
  let row = compare("x", &input(&x, OK), &killed);
  for flag in ["oom:B", "fatal-in-stderr:B", "log-missing:B"] {
    assert!(row.flags.contains(&flag.to_string()), "{flag}: {row:?}");
  }
  assert!(!row.flags.contains(&"timeout:B".to_string()), "{row:?}");
  assert!(
    row.details.contains("stderr B: Fatal:oom:alloc_failed"),
    "{}",
    row.details
  );
}

#[test]
fn a_fatal_that_exits_zero_is_a_contract_violation() {
  let x = doc("<para><p>x</p></para>");
  let side = |rc: i32| DocInput {
    meta: Some(RunMeta {
      rc: Some(rc),
      ..RunMeta::default()
    }),
    ..input(&x, "Fatal:a:b c\nStatus:conversion:3\n")
  };
  assert!(
    compare("x", &side(1), &side(0))
      .flags
      .contains(&"contract-violation:B".to_string())
  );
}

#[test]
fn a_both_sides_mismatch_keeps_its_review_tier() {
  assert_eq!(rank("status-diag-mismatch:both").0, Tier::Review);
  assert_eq!(rank("xml-unparsable:both").0, Tier::Regression);
}

#[test]
fn a_math_only_change_is_still_xml_only_review() {
  let a = doc(r#"<para><p>x <Math tex="a"><XMath><XMTok>a</XMTok></XMath></Math></p></para>"#);
  let b = doc(
    r#"<para><p>x <Math tex="a"><XMath><XMWrap><XMTok>a</XMTok></XMWrap></XMath></Math></p></para>"#,
  );
  let row = compare("x", &input(&a, OK), &input(&b, OK));
  assert!(row.flags.contains(&"math-changed".to_string()), "{row:?}");
  assert!(row.flags.contains(&"xml-only".to_string()), "{row:?}");
  assert_eq!(row.tier, Some(Tier::Review));
}

#[test]
fn a_lost_bold_run_is_review() {
  let a = doc(r#"<para><p>x <text font="bold">y</text></p></para>"#);
  let b = doc(r#"<para><p>x y</p></para>"#);
  let row = compare("x", &input(&a, OK), &input(&b, OK));
  assert!(row.flags.contains(&"wrappers-lost".to_string()), "{row:?}");
  assert_eq!(row.tier, Some(Tier::Review));
}

#[test]
fn an_unexplained_exit_code_is_a_crash() {
  let x = doc("<para><p>x</p></para>");
  let with_rc = |rc: i32, log: &str| DocInput {
    meta: Some(RunMeta {
      rc: Some(rc),
      ..RunMeta::default()
    }),
    ..input(&x, log)
  };
  let row = compare("x", &with_rc(0, OK), &with_rc(139, OK));
  assert!(row.flags.contains(&"crash:B".to_string()), "{row:?}");
  assert_eq!(row.tier, Some(Tier::Regression));
  // A Fatal exits 1 with status 3: a Fatal, not a crash.
  let fatal = "Fatal:x:y z\nStatus:conversion:3\n";
  let row = compare("x", &with_rc(1, fatal), &with_rc(1, fatal));
  assert!(row.flags.iter().all(|f| !f.starts_with("crash")), "{row:?}");
}

#[test]
fn a_status_without_its_diagnostics_is_flagged_on_its_side() {
  let x = doc("<para><p>x</p></para>");
  let row = compare("x", &input(&x, OK), &input(&x, "Status:conversion:2\n"));
  assert!(
    row.flags.contains(&"status-diag-mismatch:B".to_string()),
    "{row:?}"
  );
  assert_eq!(row.cells.last().map(String::as_str), Some("ok>mismatch"));
  let row = audit("x", &input(&x, "Error:a:b c\nStatus:conversion:0\n"));
  assert!(
    row.flags.contains(&"status-diag-mismatch".to_string()),
    "{row:?}"
  );
}

#[test]
fn an_ansi_coloured_error_makes_errors_worse() {
  let x = doc("<para><p>x</p></para>");
  let row = compare(
    "x",
    &input(&x, OK),
    &input(
      &x,
      "\x1b[31mError:\x1b[0mundefined:\\foo bad\nStatus:conversion:2\n",
    ),
  );
  for flag in ["errors+", "status-worse"] {
    assert!(row.flags.contains(&flag.to_string()), "{flag}: {row:?}");
  }
  assert_eq!(row.cells[1], "0>1");
}

#[test]
fn missing_pieces_never_pass() {
  let x = doc("<para><p>x</p></para>");
  let missing = DocInput {
    present: true,
    xml:     None,
    log:     None,
    meta:    None,
    stderr:  None,
  };
  let row = compare("x", &input(&x, OK), &missing);
  for flag in ["xml-missing:B", "log-missing:B"] {
    assert!(row.flags.contains(&flag.to_string()), "{flag}: {row:?}");
  }
  let row = compare("x", &input(&x, OK), &input(&x, "no status line\n"));
  assert!(row.flags.contains(&"no-status:B".to_string()), "{row:?}");
  assert_eq!(row.tier, Some(Tier::Regression));
  let absent = DocInput::default();
  assert!(
    compare("x", &absent, &input(&x, OK))
      .flags
      .contains(&"doc-missing:A".to_string())
  );
}

#[test]
fn identical_and_id_renumbered_documents_are_not_flagged_as_changes() {
  let a = doc(r#"<para xml:id="p1"><p>x</p></para>"#);
  let b = doc(r#"<para xml:id="p2"><p>x</p></para>"#);
  assert_eq!(
    compare("x", &input(&a, OK), &input(&a, OK)).tier,
    Some(Tier::Same)
  );
  let row = compare("x", &input(&a, OK), &input(&b, OK));
  assert_eq!(row.flags, vec!["ids-only".to_string()]);
  assert_eq!(row.tier, Some(Tier::Change));
}

#[test]
fn an_attribute_change_is_review_and_a_pure_reorder_is_xml_only() {
  let a = doc(r#"<para><p>x <text font="bold">y</text></p></para>"#);
  let b = doc(r#"<para><p>x <text font="italic">y</text></p></para>"#);
  let row = compare("x", &input(&a, OK), &input(&b, OK));
  assert_eq!(row.flags, vec!["attrs-changed:text@font".to_string()]);
  assert_eq!(row.tier, Some(Tier::Review));
  // Two attributes swapped between siblings: every multiset agrees, the canonical lines do not.
  let c = doc(r#"<para><p><text font="bold">y</text><text font="italic">z</text></p></para>"#);
  let d = doc(r#"<para><p><text font="italic">y</text><text font="bold">z</text></p></para>"#);
  let row = compare("x", &input(&c, OK), &input(&d, OK));
  assert_eq!(row.flags, vec!["xml-only".to_string()]);
  assert_eq!(row.tier, Some(Tier::Review));
}
