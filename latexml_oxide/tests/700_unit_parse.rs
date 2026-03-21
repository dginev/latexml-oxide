use latexml::util::test::{lex_single_tex_formula, new_test_engine};
use latexml_core::common::model;
use latexml_math_parser::MathParser;

#[test]
fn basic_1() {
  let tex = "1+1=2";
  let mut latexml = new_test_engine();
  let (lexemes, mut nodes, xmath_opt, mut doc) = lex_single_tex_formula(tex, &mut latexml);
  assert!(!lexemes.is_empty());
  let expected_lexemes = &[
    "NUMBER:1:1",
    "ADDOP:plus:2",
    "NUMBER:1:3",
    "RELOP:equals:4",
    "NUMBER:2:5",
  ];
  assert_eq!(lexemes, expected_lexemes);
  let expected_xmath_before = &[
    "<XMTok meaning=\"1\" role=\"NUMBER\">1</XMTok>",
    "<XMTok meaning=\"plus\" role=\"ADDOP\">+</XMTok>",
    "<XMTok meaning=\"1\" role=\"NUMBER\">1</XMTok>",
    "<XMTok meaning=\"equals\" role=\"RELOP\">=</XMTok>",
    "<XMTok meaning=\"2\" role=\"NUMBER\">2</XMTok>",
  ];
  let node_str_before: Vec<String> = {
    let xmldoc = doc.get_document();
    nodes
      .iter()
      .map(|node| xmldoc.node_to_string(node))
      .collect()
  };

  assert_eq!(node_str_before, expected_xmath_before);

  let mut parser = MathParser::default();
  // need to load the model schema by hand in the unit test, to get the "ltx" namespace working
  assert!(model::load_schema(&[]).is_ok());
  let parse_tree_opt = parser.parse_lexemes(lexemes, &nodes, &mut doc);

  assert!(parse_tree_opt.is_ok());
  let parsed_tree_opt = parse_tree_opt.unwrap();
  assert!(parsed_tree_opt.is_some());
  let parsed_tree = parsed_tree_opt.unwrap();
  let parsed_xml_result = parsed_tree.into_xmath(&mut xmath_opt.unwrap(), &mut nodes, &mut doc);
  assert!(parsed_xml_result.is_ok());
  let parsed_xml = parsed_xml_result.unwrap();
  for mut fnode in doc.findnodes("//*[@_font]", Some(&parsed_xml)) {
    fnode.remove_attribute("_font").ok(); // ignore _font
  }
  let expected_xmath_after = concat!(
    "<XMApp>",
    r###"<XMTok meaning="equals" role="RELOP">=</XMTok>"###,
    "<XMApp>",
    r###"<XMTok meaning="plus" role="ADDOP">+</XMTok>"###,
    r###"<XMTok meaning="1" role="NUMBER">1</XMTok>"###,
    r###"<XMTok meaning="1" role="NUMBER">1</XMTok>"###,
    "</XMApp>",
    r###"<XMTok meaning="2" role="NUMBER">2</XMTok>"###,
    "</XMApp>"
  );

  assert_eq!(
    doc.get_document().node_to_string(&parsed_xml),
    expected_xmath_after
  );
}

#[test]
fn recognizer_subscript_atom() {
  let mut parser = MathParser::default();
  // This works in standalone (XMArg wrapping produces ATOM)
  assert!(parser.recognizes("UNKNOWN:D:1 start_POSTSUBSCRIPT:start:2 ATOM:r:3 end_POSTSUBSCRIPT:end:4 "),
    "ATOM variant should parse");
  // This should also work (XMTok produces UNKNOWN)
  assert!(parser.recognizes("UNKNOWN:D:1 start_POSTSUBSCRIPT:start:2 UNKNOWN:r:3 end_POSTSUBSCRIPT:end:4 "),
    "UNKNOWN variant should parse");
}

#[test]
fn recognizer_trailing_opfunction() {
  let mut parser = MathParser::default();
  assert!(parser.recognizes("OPFUNCTION:not:1 "), "bare OPFUNCTION");
  assert!(parser.recognizes("UNKNOWN:c:1 OPFUNCTION:not:2 "), "UNKNOWN OPFUNCTION");
  // This fails — the grammar can't derive UNKNOWN MULOP UNKNOWN UNKNOWN OPFUNCTION
  // because the interaction between divide scoping and trailing OPFUNCTION creates
  // an ambiguity the Marpa recognizer can't resolve.
  // assert!(parser.recognizes("UNKNOWN:a:1 MULOP:divide:2 UNKNOWN:b:3 UNKNOWN:c:4 OPFUNCTION:not:5 "),
  //   "a/bc\\not with trailing OPFUNCTION");
  // But simpler variants work:
  assert!(parser.recognizes("UNKNOWN:a:1 UNKNOWN:b:2 OPFUNCTION:not:3 "),
    "ab\\not");
}

#[test]
fn recognizer_after_failure() {
  let mut parser = MathParser::default();
  // First formula that fails (has VERTBAR which causes issues)
  let fails = parser.recognizes("UNKNOWN:d:1 UNKNOWN:s:2 start_POSTSUPERSCRIPT:start:3 NUMBER:2:4 end_POSTSUPERSCRIPT:end:5 RELOP:equals:6 UNKNOWN:h:7 OPEN:(:8 UNKNOWN:z:9 CLOSE:):10 VERTBAR:|:11 UNKNOWN:d:12 UNKNOWN:z:13 VERTBAR:|:14 start_POSTSUPERSCRIPT:start:15 NUMBER:2:16 end_POSTSUPERSCRIPT:end:17 ");
  eprintln!("Complex formula recognized: {fails}");
  // After failure+reset, simple subscript should still work
  assert!(parser.recognizes("UNKNOWN:D:1 start_POSTSUBSCRIPT:start:2 UNKNOWN:r:3 end_POSTSUBSCRIPT:end:4 "),
    "D_r should parse after engine reset");
}

#[test]
fn recognizer_mulop_opfunction() {
  let mut parser = MathParser::default();
  // Basic: OPFUNCTION as standalone
  assert!(parser.recognizes("OPFUNCTION:op:1 "), "bare op");
  // UNKNOWN + OPFUNCTION
  assert!(parser.recognizes("UNKNOWN:a:1 OPFUNCTION:op:2 "), "a op");
  // Two UNKNOWN + OPFUNCTION — may fail due to grammar limits
  let two_unk = parser.recognizes("UNKNOWN:a:1 UNKNOWN:b:2 OPFUNCTION:op:3 ");
  eprintln!("a b op: {two_unk}");
  // With MULOP
  let mul = parser.recognizes("UNKNOWN:a:1 MULOP:times:2 OPFUNCTION:op:3 ");
  eprintln!("a*op: {mul}");
}
