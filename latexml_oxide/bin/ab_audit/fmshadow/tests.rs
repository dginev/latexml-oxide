use super::*;

const HEAD: &str = r#"<document xmlns="http://dlmf.nist.gov/LaTeXML"><title>T</title>"#;

fn creator(name: &str, affiliations: &[&str], rule: &str) -> TCreator {
  let mut c = TCreator {
    name: name.into(),
    affiliations: affiliations.iter().map(|a| a.to_string()).collect(),
    ..TCreator::default()
  };
  for a in affiliations {
    c.rules
      .insert(format!("affiliation\t{}", norm(a)), rule.into());
  }
  c
}

/// Convert `tex` under the shadow switch: its XML and its dump.
fn convert(tex: &str) -> (String, Value) {
  let (log, xml, json) = latexml::util::test::convert_with_setup_then(
    tex,
    Some("ar5iv.sty"),
    || latexml_engine::base_utilities::set_fm_shadow_override(Some(true)),
    |_| latexml_engine::base_utilities::fm_shadow_json(),
  );
  assert_eq!(
    (
      latexml::util::test::error_count(&log),
      latexml::util::test::warning_count(&log)
    ),
    (0, 0),
    "{log}"
  );
  (xml, serde_json::from_str(&json).expect("json"))
}

/// The comparator on two conversions: a marked block the shadow resolves as the XML has it agrees; the design's
/// `dag` probe disagrees, one classified row per moved symbol line (ruling 2: today's XML has both lines on Cat Cole,
/// the shadow each on its carriers) — and resolving the dump's events again gives the dump's own resolution.
#[test]
fn comparator_rows_on_an_agreeing_and_a_disagreeing_conversion() {
  let agreeing = "\\documentclass{article}\n\\begin{document}\n\\title{T}\n\\author{Ann Able$^{1}$ \\and Bob Baker$^{2}$\\\\ $^1$Univ A\\\\ $^2$Univ B}\n\\maketitle\nBody.\n\\end{document}\n";
  let (xml, dump) = convert(agreeing);
  let shadow = shadow_tuples(&json_resolved(&dump["resolved"]));
  assert_eq!(shadow, shadow_tuples(&fm_resolve(&json_inputs(&dump))));
  assert_eq!(
    compare(&shadow, &xml_tuples(xml.as_bytes()).expect("xml")),
    vec![]
  );
  let disagreeing = "\\documentclass{article}\n\\begin{document}\n\\title{T}\n\\author{Ann Able$^{\\dag}$ \\and Bob Baker$^{\\ddag}$ \\and Cat Cole$^{\\dag}$\\\\ \\dag Univ A\\\\ \\ddag Univ B}\n\\maketitle\nBody.\n\\end{document}\n";
  let (xml, dump) = convert(disagreeing);
  let shadow = shadow_tuples(&json_resolved(&dump["resolved"]));
  assert_eq!(shadow, shadow_tuples(&fm_resolve(&json_inputs(&dump))));
  let rows = compare(&shadow, &xml_tuples(xml.as_bytes()).expect("xml"));
  assert!(
    !rows.is_empty() && rows.iter().all(|d| d.kind == "affiliation" && d.classified),
    "{rows:?}"
  );
  assert!(
    rows.iter().any(
      |d| d.rule == "ruling2:symbol-line" && d.detail.starts_with("Ann Able: shadow-only univa")
    ),
    "{rows:?}"
  );
}

#[test]
fn names_are_compared_without_their_marks() {
  let xml = xml_tuples(
    format!(
      r#"{HEAD}<creator role="author"><personname>Ann Able<sup>1</sup></personname><contact role="affiliation"><sup>1</sup>Univ A</contact></creator></document>"#
    )
    .as_bytes(),
  )
  .expect("xml");
  let shadow = Tuples {
    creators: vec![creator("Ann Able", &["Univ A"], "ruling1:request")],
    ..Tuples::default()
  };
  assert_eq!(compare(&shadow, &xml), vec![]);
}

#[test]
fn a_symbol_line_on_its_carriers_is_a_classified_disagreement() {
  // (the dag probe: today's XML puts both symbol lines on the last name; ruling 2 on each carrier)
  let xml = xml_tuples(
    format!(
      r#"{HEAD}<creator><personname>Ann Able<sup>†</sup></personname></creator><creator><personname>Cat Cole<sup>†</sup></personname><contact role="affiliation">†Univ A</contact></creator></document>"#
    )
    .as_bytes(),
  )
  .expect("xml");
  let shadow = Tuples {
    creators: vec![
      creator("Ann Able", &["Univ A"], "ruling2:symbol-line"),
      creator("Cat Cole", &["Univ A"], "ruling2:symbol-line"),
    ],
    ..Tuples::default()
  };
  let d = compare(&shadow, &xml);
  assert_eq!(d.len(), 1, "{d:?}");
  assert_eq!(
    (d[0].kind, d[0].classified, d[0].rule.as_str()),
    ("affiliation", true, "ruling2:symbol-line")
  );
}

#[test]
fn a_name_split_by_a_reproducing_rule_is_unclassified() {
  let xml = xml_tuples(
    format!(r#"{HEAD}<creator><personname>Ann Able Bob Baker</personname></creator></document>"#)
      .as_bytes(),
  )
  .expect("xml");
  let mut a = creator("Ann Able", &[], "");
  a.rules.insert("name".into(), "line:names".into());
  let mut b = creator("Bob Baker", &[], "");
  // (no separator: a split no XML-wrong class explains)
  b.rules.insert("name".into(), "line:names".into());
  let d = compare(
    &Tuples {
      creators: vec![a, b],
      ..Tuples::default()
    },
    &xml,
  );
  assert_eq!(d.len(), 1, "{d:?}");
  assert_eq!(
    (d[0].kind, d[0].classified, d[0].rule.as_str()),
    ("name-split", false, "line:names")
  );
}

/// The block's items are compared (OD #159's nameless creator) and `rule_of` finds them there; a joined XML item is
/// explained only by the shadow's pieces joined, the word "and" between them allowed.
#[test]
fn block_items_and_joined_pieces() {
  let mut shadow = Tuples {
    creators: vec![creator(
      "Ann Able",
      &["Univ A", "Univ B"],
      "ruling2:symbol-line",
    )],
    ..Tuples::default()
  };
  shadow.block.push(("email".into(), "office@x.org".into()));
  shadow.block_rules.insert(
    "email\tofficexorg".into(),
    "owner:block ruling3:one-per-print".into(),
  );
  assert_eq!(
    rule_of(&shadow, "email", "officexorg").as_deref(),
    Some("moved owner:block ruling3:one-per-print")
  );
  assert_eq!(
    rule_of(&shadow, "affiliation", "univaandunivb").as_deref(),
    Some("overlaps ruling2:symbol-line")
  );
  assert_eq!(rule_of(&shadow, "affiliation", "univaunivc"), None);
  let xml = Tuples {
    creators: vec![creator("Ann Able", &["Univ A", "Univ B"], "")],
    block: vec![("email".into(), "office@x.org".into())],
    ..Tuples::default()
  };
  assert_eq!(compare(&shadow, &xml), vec![]);
  let xml = Tuples {
    creators: vec![creator("Ann Able", &["Univ A", "Univ B"], "")],
    ..Tuples::default()
  };
  let d = compare(&shadow, &xml);
  assert_eq!(
    (d.len(), d[0].kind, d[0].classified),
    (1, "block", false),
    "{d:?}"
  );
}

/// FM S2a.2 r3 normalisation: a name's marks the XML holds as text (`R. Vasile 1`, 0807.4407) or as a floating
/// superscript formula (`Dadhich${}^{a,\;b}$`, 1307.6166; `\,^{3}`, 0804.4669) are no text of the name.
#[test]
fn marks_as_text_or_formulas_are_no_name_text() {
  let xml = xml_tuples(
    format!(
      r#"{HEAD}<creator><personname>R. Vasile 1</personname></creator><creator><personname>Naresh Dadhich<Math tex="{{}}^{{a,\;b}}" text="x"><XMath><XMTok>a</XMTok></XMath></Math></personname></creator><creator><personname>Roberta Zambrini<Math tex="\,^{{3}}" text="x"><XMath><XMTok>3</XMTok></XMath></Math></personname></creator></document>"#
    )
    .as_bytes(),
  )
  .expect("xml");
  let mut a = creator("R. Vasile", &[], "");
  a.marks = vec!["1".into()];
  let shadow = Tuples {
    creators: vec![
      a,
      creator("Naresh Dadhich", &[], ""),
      creator("Roberta Zambrini", &[], ""),
    ],
    ..Tuples::default()
  };
  assert_eq!(compare(&shadow, &xml), vec![]);
}

/// FM S2a.2 r3 classes: names parted only by a mark or a note merged into one personname (1609.02129); a url whose text is an
/// e-mail address read as an e-mail (r2 item 8, 1701.01829); a title's note on the first author (1603.01892); the
/// addresses of a names line in the personname (2405.06319) — each disagreement classified, by its class or its tag.
#[test]
fn r3_classes_classify_their_disagreements() {
  let named = |name: &str, rule: &str| {
    let mut c = creator(name, &[], "");
    c.rules.insert("name".into(), rule.into());
    c
  };
  fn marked(mut c: TCreator, mark: &str) -> TCreator {
    c.marks = vec![mark.into()];
    c
  }
  let one = |body: &str| xml_tuples(format!("{HEAD}{body}</document>").as_bytes()).expect("xml");
  let shadow = Tuples {
    creators: vec![
      marked(named("C. Nisoli", "line:names"), "2"),
      named("C. Reichhardt", "line:names"),
    ],
    ..Tuples::default()
  };
  let d = compare(
    &shadow,
    &one("<creator><personname>C. Nisoli C. Reichhardt</personname></creator>"),
  );
  assert_eq!(
    (d.len(), d[0].class),
    (
      1,
      Some("names parted only by a mark or a note merged into one personname")
    ),
    "{d:?}"
  );
  let mut ann = named("Ann Able", "line:names");
  ann.contacts.push(("email".into(), "ann@x.org".into()));
  ann
    .rules
    .insert("email\tannxorg".into(), "api:annotate:last".into());
  let shadow = Tuples {
    creators: vec![ann.clone()],
    ..Tuples::default()
  };
  let d = compare(
    &shadow,
    &one(
      r#"<creator><personname>Ann Able</personname><contact role="url">ann@x.org</contact></creator>"#,
    ),
  );
  assert!(d.len() == 2 && d.iter().all(|d| d.classified), "{d:?}");
  let mut shadow = Tuples {
    creators: vec![named("Ann Able", "line:names")],
    ..Tuples::default()
  };
  shadow.block.push(("note".into(), "Supported.".into()));
  shadow.block_rules.insert(
    "note\tsupported".into(),
    "api:owner:block api:title-note".into(),
  );
  let d = compare(
    &shadow,
    &one(r#"<creator><personname>Ann Able</personname><note>Supported.</note></creator>"#),
  );
  assert!(
    d.len() == 2
      && d.iter().all(|d| d.class
        == Some("a note made in the title (before any author) given to the first author")),
    "{d:?}"
  );
  let shadow = Tuples {
    creators: vec![ann],
    ..Tuples::default()
  };
  let d = compare(
    &shadow,
    &one("<creator><personname>Ann Able (ann@x.org)</personname></creator>"),
  );
  assert!(
    !d.is_empty()
      && d.iter().all(|d| {
        d.class == Some("e-mail addresses printed in the names line read as creators or into the name before them")
      }),
    "{d:?}"
  );
}

/// A name disagreement is classified by an XML-wrong class only when the class explains every name: wide-space names
/// merged into one personname (2312.06207) — but not when a name is also missing.
#[test]
fn xml_wrong_name_classes_explain_every_name() {
  let mut a = creator("Ann Able", &[], "");
  a.rules.insert("name".into(), "line:names".into());
  let mut b = creator("Bob Baker", &[], "");
  b.rules.insert("name".into(), "line:names sep:gap".into());
  let shadow = Tuples {
    creators: vec![a, b],
    ..Tuples::default()
  };
  let merged = xml_tuples(
    format!(r#"{HEAD}<creator><personname>Ann Able  Bob Baker</personname></creator></document>"#)
      .as_bytes(),
  )
  .expect("xml");
  let d = compare(&shadow, &merged);
  assert_eq!(
    (d.len(), d[0].classified, d[0].class),
    (1, true, Some(XML_WRONG[0].name)),
    "{d:?}"
  );
  let other = xml_tuples(
    format!(r#"{HEAD}<creator><personname>Ann Able</personname></creator></document>"#).as_bytes(),
  )
  .expect("xml");
  let d = compare(&shadow, &other);
  assert_eq!((d.len(), d[0].classified), (1, false), "{d:?}");
}

/// FM S2a.2 r4 normalisation: a footnote set in the personname, its text without its `ltx:tags`, explains the shadow's
/// note of the creator (revtex's `\footnotemark`/`\footnotetext` on a name, 2503.10108).
#[test]
fn a_note_in_the_name_is_the_creators() {
  let xml = xml_tuples(
    format!(
      r#"{HEAD}<creator role="author"><personname>Hui-Yun Cao<note mark="*" role="footnote"><tags><tag>*</tag><tag role="creftype">footnote</tag><tag role="refnum">*</tag></tags>E-mail: caohy@hbnu.edu.cn</note></personname></creator></document>"#
    )
    .as_bytes(),
  )
  .expect("xml");
  assert_eq!(
    (
      xml.creators[0].name.as_str(),
      xml.creators[0].name_notes.clone()
    ),
    ("Hui-Yun Cao", vec!["E-mail: caohy@hbnu.edu.cn".to_string()])
  );
  let mut shadow = Tuples {
    creators: vec![creator("Hui-Yun Cao", &[], "")],
    ..Tuples::default()
  };
  shadow.creators[0]
    .notes
    .push("E-mail: caohy@hbnu.edu.cn".into());
  assert_eq!(compare(&shadow, &xml), vec![]);
}

/// FM S2a.2 r4: a paper with an author block the rehearsal could not see (out of positions: BESIII's 600-name
/// `\author`, 2012.14750) is `unobserved`, outside the gate's denominator as a held one.
#[test]
fn an_unobserved_block_is_outside_the_gate() {
  let root = std::env::temp_dir().join(format!("fmshadow_unobserved_{}", std::process::id()));
  let dir = root.join("2012.14750");
  fs::create_dir_all(&dir).expect("dir");
  fs::write(
    dir.join("out.xml"),
    format!(
      r#"{HEAD}<creator role="author"><personname>Ann Able</personname></creator></document>"#
    ),
  )
  .expect("xml");
  let entry = |status: &str, extra: &str| {
    format!(
      r#"{{"index":0,"kind":"author","hook":"\\lx@add@authors@append","binding":"revtex4_support.sty","status":"{status}"{extra}}}"#
    )
  };
  fs::write(
    dir.join("out.xml.fmshadow.json"),
    format!(
      r#"{{"entries":[{},{}],"resolved":{{"creators":[],"block":[],"dropped":[],"policies":[]}}}}"#,
      entry("skipped", r#","reason":"unstamped: out of positions""#),
      entry("events", r#","errors":0,"events":[]"#)
    ),
  )
  .expect("json");
  let row = compare_paper(&root, "2012.14750", false);
  fs::remove_dir_all(&root).expect("cleanup");
  assert_eq!(
    (row.status.as_str(), row.family.as_str()),
    ("unobserved", "revtex4_support.sty")
  );
}

/// FM S2a.2 r4: the XML-wrong class "legend lines welded onto the affiliation above them" — the XML's affiliation is
/// the shadow's with the symbol lines' notes after it, an e-mail legend's label words before its address (2305.13575,
/// 2012.15253, 2110.08090 …, each PDF-confirmed).
#[test]
fn welded_legends_are_an_xml_wrong_class() {
  let mut ann = creator("Ann Able", &["Univ A, City"], "ruling1:request");
  ann.notes.push("These authors contributed equally.".into());
  ann.rules.insert(
    format!("note\t{}", norm("These authors contributed equally.")),
    "ruling2:symbol-line".into(),
  );
  let mut bob = creator("Bob Baker", &["Univ B, Town"], "ruling1:request");
  bob
    .contacts
    .push(("email".into(), "bob@example.org".into()));
  bob.rules.insert(
    format!("email\t{}", norm("bob@example.org")),
    "ruling2:symbol-line".into(),
  );
  let shadow = Tuples {
    creators: vec![ann, bob],
    ..Tuples::default()
  };
  let xml = xml_tuples(
    format!(
      r#"{HEAD}<creator role="author"><personname>Ann Able</personname><contact role="affiliation">Univ A, City</contact></creator><creator role="author"><personname>Bob Baker</personname><contact role="affiliation">Univ B, Town∗These authors contributed equally.†Corresponding author: bob@example.org</contact></creator></document>"#
    )
    .as_bytes(),
  )
  .expect("xml");
  let d = compare(&shadow, &xml);
  let welded: Vec<_> = d.iter().filter(|d| d.detail.contains("xml-only")).collect();
  assert!(
    !welded.is_empty()
      && welded.iter().all(|d| {
        d.classified
          && d.class
            == Some("legend lines (a symbol and its note) welded onto the affiliation above them")
      }),
    "{d:?}"
  );
}
