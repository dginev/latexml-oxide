use rtx_core::state::{State, StateOptions};
use rtx::util::test::lex_single_tex_formula;
use rtx_math_parser::MathParser;

#[test]
fn basic_1() {
  let tex = "1+1=2";
  let (lexemes, nodes, _xmath_opt, mut doc) = lex_single_tex_formula(tex);
  assert!(!lexemes.is_empty());
  let expected_lexemes = &["NUMBER:1:1", "ADDOP:plus:2", "NUMBER:1:3", "RELOP:equals:4", "NUMBER:2:5"];
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
    nodes.iter().map(|node| xmldoc.node_to_string(node)).collect()
  };

  assert_eq!(node_str_before, expected_xmath_before);

  let mut parser = MathParser::default();
  let mut state = State::new(StateOptions::default());
  let parse_tree_opt = parser.parse_lexemes(lexemes, nodes, &mut doc, &mut state);

  assert!(parse_tree_opt.is_ok());
  let parsed_tree_opt = parse_tree_opt.unwrap();
  assert!(parsed_tree_opt.is_some());
  let parsed_tree = parsed_tree_opt.unwrap();

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

  assert_eq!(doc.get_document().node_to_string(&parsed_tree), expected_xmath_after);
}
