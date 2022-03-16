use rtx::util::test::lex_single_tex_formula;
use rtx_math_parser::*;

#[test]
fn basic_1() {
  let tex="1+1=2";
  let lexed = lex_single_tex_formula(tex);
  assert!(!lexed.is_empty());
  let mut lexemes = Vec::new();
  let mut nodes = Vec::new();
  for (lexeme, node) in lexed.into_iter() {
    lexemes.push(lexeme);
    nodes.push(node);
  }
  let expected_lexemes = &["NUMBER:1:1", "ADDOP:plus:2", "NUMBER:1:3", "RELOP:equals:4", "NUMBER:2:5"];
  assert_eq!(lexemes, expected_lexemes);
  let expected_xmath_before = 
r###"<XMath>
  <XMTok meaning="1" role="NUMBER">1</XMTok>
  <XMTok meaning="plus" role="ADDOP">+</XMTok>
  <XMTok meaning="1" role="NUMBER">1</XMTok>
  <XMTok meaning="equals" role="RELOP">=</XMTok>
  <XMTok meaning="2" role="NUMBER">2</XMTok>
</XMath>"###;
  let parse_tree = parse_math(lexemes, nodes);
  
  let expected_xmath_after = 
r###"<XMath>
  <XMApp>
    <XMTok meaning="equals" role="RELOP">=</XMTok>
    <XMApp>
      <XMTok meaning="plus" role="ADDOP">+</XMTok>
      <XMTok meaning="1" role="NUMBER">1</XMTok>
      <XMTok meaning="1" role="NUMBER">1</XMTok>
    </XMApp>
    <XMTok meaning="2" role="NUMBER">2</XMTok>
  </XMApp>
</XMath>"###;
}
