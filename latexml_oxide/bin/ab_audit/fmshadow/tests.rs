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
  assert_eq!(row.unobserved, ["unstamped: out of positions"]);
}

/// FM S2a.2 r5 (review revfms2a2_4 finding 2): an unobserved paper's observed blocks are compared all the same — the
/// names only the XML has (the unseen block's) are no disagreement, an observed block's own disagreement is kept —, and
/// a paper whose only block is unseen is `unobserved`, not "no block".
#[test]
fn an_unobserved_papers_observed_blocks_are_compared() {
  let root = std::env::temp_dir().join(format!("fmshadow_observed_{}", std::process::id()));
  let write = |key: &str, xml: &str, entries: &str, creators: &str| {
    let dir = root.join(key);
    fs::create_dir_all(&dir).expect("dir");
    fs::write(dir.join("out.xml"), format!("{HEAD}{xml}</document>")).expect("xml");
    fs::write(
      dir.join("out.xml.fmshadow.json"),
      format!(
        r#"{{"entries":[{entries}],"resolved":{{"creators":[{creators}],"block":[],"dropped":[],"policies":[]}}}}"#
      ),
    )
    .expect("json");
  };
  let skipped = r#"{"index":1,"kind":"author","hook":"\\lx@add@authors@append","binding":"revtex4_support.sty","status":"skipped","reason":"unstamped: out of positions"}"#;
  let observed = r#"{"index":0,"kind":"author","hook":"\\lx@add@authors@append","binding":"revtex4_support.sty","status":"events","errors":0,"events":[]}"#;
  let ann = r#"{"name":"Ann Able","requests":[],"rule":"line:names","source":0,"affiliations":[{"role":"affiliation","text":"Univ A","rule":"api:annotate:new"}],"contacts":[],"notes":[]}"#;
  let xml = r#"<creator role="author"><personname>Ann Able</personname><contact role="affiliation">Univ B</contact></creator><creator role="author"><personname>Bob Baker</personname></creator>"#;
  write("1406.2912", xml, &format!("{observed},{skipped}"), ann);
  write("2012.14750", xml, skipped, "");
  let row = compare_paper(&root, "1406.2912", false);
  let only = compare_paper(&root, "2012.14750", false);
  fs::remove_dir_all(&root).expect("cleanup");
  assert_eq!(row.status, "unobserved");
  assert_eq!(
    row
      .disagreements
      .iter()
      .map(|d| (d.kind, d.detail.as_str()))
      .collect::<Vec<_>>(),
    [
      ("affiliation", "Ann Able: shadow-only univa"),
      ("affiliation", "Ann Able: xml-only univb")
    ]
  );
  assert_eq!(only.status, "unobserved");
}

/// FM S2a.2 r5 (review revfms2a2_4 finding 1): a note set in the XML's personname that the shadow lacks is a `note`
/// disagreement of its own, unclassified (the text of `Steven Delvaux\footnotemark[1]` and the body's
/// `\footnotetext[1]{…}`, 1108.5155: S2b would drop it); one the shadow holds agrees.
#[test]
fn a_name_note_the_shadow_lacks_is_a_disagreement() {
  let xml = xml_tuples(
    format!(
      r#"{HEAD}<creator role="author"><personname>Steven Delvaux<note mark="*" role="footnote"><tags><tag>*</tag></tags>Department of Mathematics, KU Leuven.</note></personname></creator></document>"#
    )
    .as_bytes(),
  )
  .expect("xml");
  let shadow = Tuples {
    creators: vec![creator("Steven Delvaux", &[], "")],
    ..Tuples::default()
  };
  let found = compare(&shadow, &xml);
  assert_eq!(
    found
      .iter()
      .map(|d| (d.kind, d.classified, d.detail.as_str()))
      .collect::<Vec<_>>(),
    [(
      "note",
      false,
      "Steven Delvaux: xml-only departmentofmathematicskuleuven (in the name)"
    )]
  );
}

/// FM S2a.2 r5: the two r3 XML-wrong classes the r4 sample found misapplied, tightened — a row whose names hold an
/// acronym is no later names row (`IBM India, IBM CIO`, 2609.10596), a name opening with no letter or holding a
/// spacing accent is no name a wide space parted (`TIÊ ´N-SO | . N PHẠM`, 1706.00234) — and the new separator class
/// (`&` a creator, 2411.16601; `Seema Kushwaha AND`, 2106.11727).
#[test]
fn r5_classes_are_fenced_and_the_separator_class() {
  let names = |names: &[(&str, &str)]| Tuples {
    creators: names
      .iter()
      .map(|(n, rule)| {
        let mut c = creator(n, &[], "");
        c.rules.insert("name".into(), (*rule).into());
        c
      })
      .collect(),
    ..Tuples::default()
  };
  let with_affiliation = |name: &str, affiliation: &str| Tuples {
    creators: vec![creator(name, &[affiliation], "")],
    ..Tuples::default()
  };
  let classes = |shadow: &Tuples, xml: &Tuples| {
    compare(shadow, xml)
      .into_iter()
      .map(|d| d.class)
      .collect::<Vec<_>>()
  };
  // (a later names row read as the affiliation: classified for names, not for an acronym row)
  let row = "a later names row read as the affiliation of the row above";
  assert!(
    classes(
      &names(&[
        ("Eric S. Egge", "line:names"),
        ("Toufik Mansour", "line:names")
      ]),
      &with_affiliation("Eric S. Egge", "Toufik Mansour LaBRI")
    )
    .contains(&Some(row))
  );
  assert!(
    !classes(
      &names(&[
        ("Bhanwar Gupta", "line:names"),
        ("IBM India", "line:names"),
        ("IBM CIO", "line:names sep:comma")
      ]),
      &with_affiliation("Bhanwar Gupta", "IBM India, IBM CIO")
    )
    .contains(&Some(row))
  );
  // (a wide space: whole names parted, not an accent's glyphs)
  let wide =
    "names parted by a wide space (\\quad, \\enspace, \\hspace, spaces) merged into one personname";
  assert_eq!(
    classes(
      &names(&[
        ("Ann Able", "line:names"),
        ("Bob Baker", "line:names sep:gap")
      ]),
      &names(&[("Ann Able Bob Baker", "")])
    ),
    [Some(wide)]
  );
  assert_eq!(
    classes(
      &names(&[
        ("TIÊ ´N-SO", "line:names"),
        (". N PHẠM", "line:names sep:gap")
      ]),
      &names(&[("TIÊ´N-SO .N PHẠM", "")])
    ),
    [None]
  );
  // (the separator between names read as a creator or into a name)
  let separator = "the separator between two names (an ampersand, the word \"and\") read as a creator or into a name";
  assert_eq!(
    classes(
      &names(&[
        ("Sonja Hohloch", "line:names"),
        ("Pedro Santos", "line:names sep:word")
      ]),
      &names(&[("Sonja Hohloch", ""), ("&", ""), ("Pedro Santos", "")])
    ),
    [Some(separator)]
  );
  assert_eq!(
    classes(
      &names(&[
        ("Seema Kushwaha", "line:names"),
        ("Sumit Kumar Upadhyay", "line:names sep:word")
      ]),
      &names(&[("Seema Kushwaha AND", ""), ("Sumit Kumar Upadhyay", "")])
    ),
    [Some(separator)]
  );
}

/// The XML-wrong class "the address groups written after all the authors … given to the last author": the shadow parts
/// the groups among the authors in order (`owner:upfront-grid`), the XML gives them all to the last name (2507.07513,
/// 1812.04979, 2010.11440, each PDF-confirmed); not when an earlier XML creator holds one of them.
#[test]
fn address_groups_given_to_the_last_author_are_an_xml_wrong_class() {
  let grid = "the address groups written after all the authors, one per author in order, given to the last author";
  let mut ann = creator(
    "Ann Able",
    &["Univ A, City"],
    "api:annotate:last owner:upfront-grid",
  );
  ann.contacts.push(("email".into(), "ann@a.edu".into()));
  ann.rules.insert(
    "email\tannaedu".into(),
    "api:annotate:last owner:upfront-grid".into(),
  );
  let mut bob = creator("Bob Baker", &["Univ B, Town"], "api:annotate:last");
  bob.contacts.push(("email".into(), "bob@b.edu".into()));
  bob
    .rules
    .insert("email\tbobbedu".into(), "api:annotate:last".into());
  let shadow = Tuples {
    creators: vec![ann, bob],
    ..Tuples::default()
  };
  let mut piled = creator("Bob Baker", &["Univ A, City", "Univ B, Town"], "");
  piled.contacts = vec![
    ("email".into(), "ann@a.edu".into()),
    ("email".into(), "bob@b.edu".into()),
  ];
  let xml = Tuples {
    creators: vec![creator("Ann Able", &[], ""), piled.clone()],
    ..Tuples::default()
  };
  let classes: Vec<_> = compare(&shadow, &xml)
    .into_iter()
    .map(|d| d.class)
    .collect();
  assert!(
    !classes.is_empty() && classes.iter().all(|c| *c == Some(grid)),
    "{classes:?}"
  );
  // (an earlier creator holding one of the groups is no pile on the last)
  let xml = Tuples {
    creators: vec![creator("Ann Able", &["Univ A, City"], ""), piled],
    ..Tuples::default()
  };
  assert!(
    compare(&shadow, &xml)
      .into_iter()
      .all(|d| d.class != Some(grid))
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

/// FM S2a.2 r6, decision 3: a braced address group an `\institute` entry's `\email` prints, written out to the authors
/// it spells and the block, disagrees with today's whole group on every author of the entry — each row classified by
/// the decision's tag (`ruling3:written-out`).
#[test]
fn a_written_out_address_group_is_classified() {
  let tex = "\\documentclass{llncs}\n\\begin{document}\n\\title{T}\n\\author{Ann Able\\inst{1} \\and Bob Baker\\inst{1}}\n\\institute{Univ A, City\\\\ \\email{\\{ann.able,bob.baker,info\\}@example.org}}\n\\maketitle\nBody.\n\\end{document}\n";
  let (xml, dump) = convert(tex);
  let shadow = shadow_tuples(&json_resolved(&dump["resolved"]));
  let rows = compare(&shadow, &xml_tuples(xml.as_bytes()).expect("xml"));
  assert_eq!(rows.len(), 5, "{rows:?}");
  assert!(
    rows
      .iter()
      .all(|d| d.classified && d.rule.contains("ruling3:written-out")),
    "{rows:?}"
  );
}

/// FM S2a.2 r6, decision 2: an ORCID link a note keeps in its text (no one's contact) is no difference in the note — the
/// XML's prints the logo, the shadow's the iD (2602.13176).
#[test]
fn a_notes_orcid_id_is_no_note_text() {
  assert_eq!(
    without_orcid_ids("sdonahue0000000283879887iswith00000000000000001"),
    "sdonahueiswith00000000000000001"
  );
  assert_eq!(without_orcid_ids("x000900021234567xy"), "xy");
}

/// FM S2a.2 r7: today's one contact a line of an entry the shadow reads with its continuation line as one affiliation
/// (`line:continued`) is that item's line — classified by the joined item's rule (`part …`), not left unexplained
/// (0907.1243: `$\dagger$ Laboratory …, Rm. 1E116 \\ National Institutes of Health, …` under the `†` carriers once
/// `$\diamond$` is a mark; each line went to every author in today's).
#[test]
fn a_line_of_a_continued_entry_is_its_part() {
  let tex = "\\documentclass[aps,pra]{revtex4-2}\n\\begin{document}\n\\title{T}\n\\author{Ann Able $\\diamond$, Bob Baker $\\dagger$ \\\\ $\\diamond$ Univ A, City \\\\ $\\dagger$ Laboratory B, Bld. 9 \\\\ Institute C, Town}\n\\begin{abstract}\nAbstract.\n\\end{abstract}\n\\maketitle\nBody.\n\\end{document}\n";
  let (xml, dump) = convert(tex);
  let shadow = shadow_tuples(&json_resolved(&dump["resolved"]));
  let rows = compare(&shadow, &xml_tuples(xml.as_bytes()).expect("xml"));
  assert!(rows.iter().all(|d| d.classified), "{rows:?}");
  let parts: Vec<&str> = rows
    .iter()
    .filter(|d| d.rule.starts_with("part "))
    .map(|d| d.detail.as_str())
    .collect();
  assert_eq!(parts, [
    "Ann Able: xml-only institutectown",
    "Ann Able: xml-only laboratorybbld9",
    "Bob Baker: xml-only institutectown",
    "Bob Baker: xml-only laboratorybbld9",
  ]);
}

/// FM S2a.2 r7: today's note the shadow reads as an entry's legend, the affiliation of the names showing its leading
/// mark (`note:legend`: a note opening with `$\star$` under the `⋆` authors once the lexer reads `⋆` as a mark,
/// 1803.05754, 2607.19263), is explained by that rule — it was looked up among the shadow's notes alone.
#[test]
fn a_note_read_as_a_legend_affiliation_is_explained() {
  let tex = "\\documentclass{article}\n\\begin{document}\n\\title{T}\n\\author{Ann Able$^\\star$, Bob Baker$^\\star$ and Cat Cole\\thanks{$\\star$ Univ A, City.}}\n\\maketitle\nBody.\n\\end{document}\n";
  let (xml, dump) = convert(tex);
  let shadow = shadow_tuples(&json_resolved(&dump["resolved"]));
  let rows = compare(&shadow, &xml_tuples(xml.as_bytes()).expect("xml"));
  assert!(
    !rows.is_empty() && rows.iter().all(|d| d.classified),
    "{rows:?}"
  );
  assert!(
    rows
      .iter()
      .any(|d| d.kind == "note" && d.rule.contains("note:legend") && d.detail.contains("xml-only")),
    "{rows:?}"
  );
}

/// FM S2a.2 r7: legend lines welded onto the affiliation above them, one led by a mark glyph of no `\@fnsymbol` kind
/// (`${\star}$`, ruling 1's legend once the lexer reads `⋆` as a mark) and the word "and" between two of them, are the
/// PDF-confirmed XML-wrong class (2401.00056).
#[test]
fn welded_legends_of_a_mark_glyph_are_the_xml_wrong_class() {
  let tex = "\\documentclass{article}\n\\begin{document}\n\\title{T}\n\\author{Ann Able,$^{1,\\#}$ Bob Baker,$^{2,\\#}$ Cat Cole$^{2,\\star}$\\\\ $^1$Univ A, City.\\\\ $^2$Univ B, Town.\\\\ \\normalsize{\\# authors contributed equally and ${\\star}$ to whom correspondence should be addressed.}}\n\\maketitle\nBody.\n\\end{document}\n";
  let (xml, dump) = convert(tex);
  let shadow = shadow_tuples(&json_resolved(&dump["resolved"]));
  let rows = compare(&shadow, &xml_tuples(xml.as_bytes()).expect("xml"));
  assert!(rows.iter().all(|d| d.classified), "{rows:?}");
  let welded: Vec<&str> = rows
    .iter()
    .filter(|d| d.class.is_some_and(|c| c.starts_with("legend lines")))
    .map(|d| d.detail.as_str())
    .collect();
  assert_eq!(welded, [
    "Bob Baker: xml-only univbtownauthorscontributedequallyandtowhomcorrespondenceshouldbeaddressed",
    "Cat Cole: xml-only univbtownauthorscontributedequallyandtowhomcorrespondenceshouldbeaddressed",
  ]);
}

/// FM S2a.2 r7: today's affiliation the shadow reads as legend lines, notes (`$^{\$}$These authors contributed equally to
/// this work. \\ $^*$Corresponding author` welded into one contact, 1910.10432 — a legend's mark is a symbol, `$` too),
/// is explained by those entries' rule; it was looked up among the shadow's affiliations alone.
#[test]
fn an_affiliation_read_as_legend_notes_is_explained() {
  let tex = "\\documentclass{article}\n\\begin{document}\n\\title{T}\n\\author{Ann Able$^{1,\\$}$, Bob Baker$^{1,\\$}$, Cat Cole$^{1,*}$\\\\ $^1$Univ A, City.\\\\ $^\\$$These authors contributed equally to this work.\\\\ $^*$Corresponding author}\n\\maketitle\nBody.\n\\end{document}\n";
  let (xml, dump) = convert(tex);
  let shadow = shadow_tuples(&json_resolved(&dump["resolved"]));
  let rows = compare(&shadow, &xml_tuples(xml.as_bytes()).expect("xml"));
  assert!(rows.iter().all(|d| d.classified), "{rows:?}");
  assert_eq!(
    rows
      .iter()
      .filter(|d| d.kind == "affiliation" && d.rule == "overlaps ruling1:request")
      .count(),
    2,
    "{rows:?}"
  );
}

/// FM S2a.2 r8 (review revfms2a2_7 finding 1): the notes a creator's two sides both hold are compared in order — a
/// note the shadow placed after the ones the source gives later is a `note-order` disagreement (the multisets hid it),
/// unclassified unless the first note out of today's place is one a ruling placed (a legend by its label: today's
/// relocates it last, 2607.03557).
#[test]
fn note_order_is_compared() {
  let with_notes = |notes: &[&str]| {
    let mut c = creator("Ann Able", &[], "");
    for n in notes {
      c.notes.push(n.to_string());
      c.rules
        .insert(format!("note\t{}", norm(n)), "note:adjacent".into());
    }
    Tuples {
      creators: vec![c],
      ..Tuples::default()
    }
  };
  let xml = with_notes(&["ann@x.org, zed@y.org", "Second note."]);
  assert_eq!(
    compare(&with_notes(&["ann@x.org, zed@y.org", "Second note."]), &xml),
    vec![]
  );
  let rows = compare(&with_notes(&["Second note.", "ann@x.org, zed@y.org"]), &xml);
  assert_eq!(rows.len(), 1, "{rows:?}");
  assert_eq!(
    (rows[0].kind, rows[0].classified, rows[0].rule.as_str()),
    ("note-order", false, "order-of:note:adjacent")
  );
  assert_eq!(
    rows[0].detail,
    "Ann Able: shadow [secondnote | annxorgzedyorg] xml [annxorgzedyorg | secondnote]"
  );
  let mut legend = with_notes(&["Corresponding author.", "Supported by X."]);
  legend.creators[0].rules.insert(
    "note\tcorrespondingauthor".into(),
    "note:adjacent note:legend ruling1:request".into(),
  );
  let rows = compare(
    &legend,
    &with_notes(&["Supported by X.", "Corresponding author."]),
  );
  assert_eq!(
    rows
      .iter()
      .map(|d| (d.kind, d.classified, d.rule.as_str()))
      .collect::<Vec<_>>(),
    [(
      "note-order",
      true,
      "order note:adjacent note:legend ruling1:request"
    )]
  );
  // (review revfms2a2_9 finding 3: only a rule that places by order excuses the order — a note whose tag says whose it
  // is, not where it sits, does not; nor a legend the XML does not have last, where today's relocation puts it)
  let mut owned = with_notes(&["ann@x.org", "Supported by X."]);
  owned.creators[0].rules.insert(
    "note\tannxorg".into(),
    "note:adjacent ruling4:inner-contact owner:spelled".into(),
  );
  let rows = compare(&owned, &with_notes(&["Supported by X.", "ann@x.org"]));
  assert_eq!(
    rows
      .iter()
      .map(|d| (d.kind, d.classified, d.rule.as_str()))
      .collect::<Vec<_>>(),
    [(
      "note-order",
      false,
      "order-of:note:adjacent+ruling4:inner-contact+owner:spelled"
    )]
  );
  let mut legend = with_notes(&["Supported by X.", "Corresponding author.", "Third note."]);
  legend.creators[0].rules.insert(
    "note\tcorrespondingauthor".into(),
    "note:adjacent note:legend ruling1:request".into(),
  );
  let rows = compare(
    &legend,
    &with_notes(&["Corresponding author.", "Third note.", "Supported by X."]),
  );
  assert_eq!(
    rows
      .iter()
      .map(|d| (d.kind, d.classified, d.rule.as_str()))
      .collect::<Vec<_>>(),
    [("note-order", false, "order-of:note:adjacent")]
  );
}

/// FM S2a.2 r8 (review revfms2a2_7 finding 5): a `part` is one line of an entry the shadow joined from its continuation
/// lines, or a run of them, as the dump holds the lines — the middle of a line explains nothing; a dump made before r8
/// (no lines) keeps r7's reading.
#[test]
fn a_part_is_a_line_or_a_run_of_lines() {
  let piece = "laboratorybbld9institutectowncountryd";
  let shadow = Tuples {
    continued: Some(vec![vec![
      "laboratorybbld9".into(),
      "institutectown".into(),
      "countryd".into(),
    ]]),
    ..Tuples::default()
  };
  assert!(is_line_run(&shadow, piece, "institutectown"));
  assert!(is_line_run(&shadow, piece, "institutectowncountryd"));
  assert!(!is_line_run(&shadow, piece, "bbld9institutec"));
  assert!(!is_line_run(&shadow, piece, "institute"));
  assert!(is_line_run(&Tuples::default(), piece, "bbld9institutec"));
  // (FM S2a.2 r9, review revfms2a2_8 finding 4: an r8 dump with no continued entries is no old dump — nothing is a
  // part; a dump without the key is one)
  let none = Tuples {
    continued: Some(Vec::new()),
    ..Tuples::default()
  };
  assert!(!is_line_run(&none, piece, "bbld9institutec"));
  assert_eq!(json_resolved(&serde_json::json!({})).continued, None);
  assert_eq!(
    json_resolved(&serde_json::json!({"continued": []})).continued,
    Some(Vec::new())
  );
}

/// FM S2a.2 r9 (review revfms2a2_8 finding 3a): the notes a ruling placed taken out of both sides, the rest must agree
/// in order — shadow `[L, A, B]` against today's `[B, A, L]` (L a legend ruling 1 places) was explained by L's rule, A
/// and B swapped; it is unclassified now, the rule the first of the rest out of order. With the rest in order, the placed
/// note's rule explains it (2607.03557's shape).
#[test]
fn note_order_drops_the_ruling_placed_notes_first() {
  let side = |notes: &[&str]| {
    let mut c = creator("Ann Able", &[], "");
    for n in notes {
      c.notes.push(n.to_string());
      let rule = if *n == "Corresponding author." {
        "note:adjacent note:legend ruling1:request"
      } else {
        "note:adjacent"
      };
      c.rules.insert(format!("note\t{}", norm(n)), rule.into());
    }
    Tuples {
      creators: vec![c],
      ..Tuples::default()
    }
  };
  let rows = compare(
    &side(&["Corresponding author.", "First.", "Second."]),
    &side(&["Second.", "First.", "Corresponding author."]),
  );
  assert_eq!(
    rows
      .iter()
      .map(|d| (d.kind, d.classified, d.rule.as_str()))
      .collect::<Vec<_>>(),
    [("note-order", false, "order-of:note:adjacent")],
    "{rows:?}"
  );
  let rows = compare(
    &side(&["Corresponding author.", "First.", "Second."]),
    &side(&["First.", "Second.", "Corresponding author."]),
  );
  assert_eq!(
    rows
      .iter()
      .map(|d| (d.kind, d.classified, d.rule.as_str()))
      .collect::<Vec<_>>(),
    [(
      "note-order",
      true,
      "order note:adjacent note:legend ruling1:request"
    )],
    "{rows:?}"
  );
}

/// FM S2a.2 r9 (review revfms2a2_8 finding 3b): every rule of a key is kept, and each copy the XML lacks takes one, the
/// copies it holds taking those that reproduce today's first — 1910.01401's two ORCID copies, one placed by
/// `owner:adjacent` and one by the ruling, the XML holding none: one row classified, one not (the ruling's rule stood for
/// both). With the XML holding one copy, the copy it lacks is the ruling's.
#[test]
fn each_surplus_copy_takes_its_own_rule() {
  let id = "0000-0001-8509-009X";
  let shadow = |rules: &[&str]| {
    let mut c = creator("Konrad Schrempf", &[], "");
    for rule in rules {
      c.contacts.push(("orcid".into(), id.into()));
      c.all_rules
        .entry(format!("orcid\t{}", norm(id)))
        .or_default()
        .push(rule.to_string());
    }
    c.rules.insert(
      format!("orcid\t{}", norm(id)),
      "note:adjacent ruling4:inner-contact".into(),
    );
    Tuples {
      creators: vec![c],
      ..Tuples::default()
    }
  };
  let xml = |copies: usize| Tuples {
    creators: vec![TCreator {
      name: "Konrad Schrempf".into(),
      contacts: vec![("orcid".to_string(), id.to_string()); copies],
      ..TCreator::default()
    }],
    ..Tuples::default()
  };
  let two = shadow(&["owner:adjacent", "note:adjacent ruling4:inner-contact"]);
  let rows = compare(&two, &xml(0));
  assert_eq!(
    rows
      .iter()
      .map(|d| (d.classified, d.rule.as_str()))
      .collect::<Vec<_>>(),
    [
      (false, "owner:adjacent"),
      (true, "note:adjacent ruling4:inner-contact")
    ],
    "{rows:?}"
  );
  let rows = compare(&two, &xml(1));
  assert_eq!(
    rows
      .iter()
      .map(|d| (d.classified, d.rule.as_str()))
      .collect::<Vec<_>>(),
    [(true, "note:adjacent ruling4:inner-contact")],
    "{rows:?}"
  );
}

/// FM S2a.2 r9 (review revfms2a2_8 finding 2): today's affiliation the shadow reads as an API entry's legend note is
/// explained by the legend's own tag, `api:legend`, not by the leading mark that labels any API entry.
#[test]
fn an_api_legend_is_explained_by_its_own_tag() {
  let text = "Corresponding author.";
  let with_rule = |rule: &str| {
    let mut c = creator("Ann Able", &[], "");
    c.notes.push(text.into());
    c.rules.insert(format!("note\t{}", norm(text)), rule.into());
    Tuples {
      creators: vec![c],
      ..Tuples::default()
    }
  };
  assert_eq!(
    affiliation_rule_of(&with_rule("api:leading-mark api:legend"), &norm(text)),
    Some("moved api:leading-mark api:legend".to_string())
  );
  assert_eq!(
    affiliation_rule_of(&with_rule("api:leading-mark"), &norm(text)),
    None
  );
}
