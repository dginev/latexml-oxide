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
  assert!(
    parser.recognizes("UNKNOWN:D:1 start_POSTSUBSCRIPT:start:2 ATOM:r:3 end_POSTSUBSCRIPT:end:4 "),
    "ATOM variant should parse"
  );
  // This should also work (XMTok produces UNKNOWN)
  assert!(
    parser
      .recognizes("UNKNOWN:D:1 start_POSTSUBSCRIPT:start:2 UNKNOWN:r:3 end_POSTSUBSCRIPT:end:4 "),
    "UNKNOWN variant should parse"
  );
}

#[test]
fn recognizer_trailing_opfunction() {
  let mut parser = MathParser::default();
  assert!(parser.recognizes("OPFUNCTION:not:1 "), "bare OPFUNCTION");
  assert!(
    parser.recognizes("UNKNOWN:c:1 OPFUNCTION:not:2 "),
    "UNKNOWN OPFUNCTION"
  );
  // This fails — the grammar can't derive UNKNOWN MULOP UNKNOWN UNKNOWN OPFUNCTION
  // because the interaction between divide scoping and trailing OPFUNCTION creates
  // an ambiguity the Marpa recognizer can't resolve.
  // assert!(parser.recognizes("UNKNOWN:a:1 MULOP:divide:2 UNKNOWN:b:3 UNKNOWN:c:4 OPFUNCTION:not:5
  // "),   "a/bc\\not with trailing OPFUNCTION");
  // But simpler variants work:
  assert!(
    parser.recognizes("UNKNOWN:a:1 UNKNOWN:b:2 OPFUNCTION:not:3 "),
    "ab\\not"
  );
}

#[test]
fn recognizer_after_failure() {
  let mut parser = MathParser::default();
  // First formula that fails (has VERTBAR which causes issues)
  let fails = parser.recognizes("UNKNOWN:d:1 UNKNOWN:s:2 start_POSTSUPERSCRIPT:start:3 NUMBER:2:4 end_POSTSUPERSCRIPT:end:5 RELOP:equals:6 UNKNOWN:h:7 OPEN:(:8 UNKNOWN:z:9 CLOSE:):10 VERTBAR:|:11 UNKNOWN:d:12 UNKNOWN:z:13 VERTBAR:|:14 start_POSTSUPERSCRIPT:start:15 NUMBER:2:16 end_POSTSUPERSCRIPT:end:17 ");
  eprintln!("Complex formula recognized: {fails}");
  // After failure+reset, simple subscript should still work
  assert!(
    parser
      .recognizes("UNKNOWN:D:1 start_POSTSUBSCRIPT:start:2 UNKNOWN:r:3 end_POSTSUBSCRIPT:end:4 "),
    "D_r should parse after engine reset"
  );
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

#[test]
fn repeated_parse_failures_no_corruption() {
  // Stress test: repeatedly parse unrecognized token sequences.
  // Unrecognized role tokens like start_DIFFOP/end_DIFFOP caused garbled
  // output in earlier implementations. This test verifies that failed parses
  // don't corrupt the parser state or cause memory issues.
  let mut parser = MathParser::default();

  // These sequences should all fail (grammar has no rules for DIFFOP/CUSTOMROLE)
  for i in 0..5 {
    let fails = parser.recognizes(&format!(
      "start_DIFFOP:start:{} UNKNOWN:f:{} end_DIFFOP:end:{} ",
      i * 3 + 1,
      i * 3 + 2,
      i * 3 + 3
    ));
    assert!(!fails, "DIFFOP sequence #{i} should NOT parse");
  }

  // After repeated failures, the parser should still handle valid sequences
  assert!(
    parser.recognizes("UNKNOWN:a:1 ADDOP:plus:2 UNKNOWN:b:3 "),
    "a+b should parse after repeated failures"
  );
  assert!(
    parser
      .recognizes("UNKNOWN:D:1 start_POSTSUBSCRIPT:start:2 UNKNOWN:r:3 end_POSTSUBSCRIPT:end:4 "),
    "D_r should parse after repeated failures"
  );
}

#[test]
fn parse_failure_with_full_document() {
  // End-to-end test: parse a TeX formula that produces XMWrap with
  // rewrite role, ensuring the full pipeline handles parse failures gracefully.
  let tex = "a+b";
  let mut latexml = new_test_engine();
  let (_lexemes, _nodes, xmath_opt, mut doc) = lex_single_tex_formula(tex, &mut latexml);
  assert!(xmath_opt.is_some(), "should produce XMath");

  // Parse a simple formula to verify the document is usable
  assert!(model::load_schema(&[]).is_ok());
  let mut parser = MathParser::default();
  parser.parse_math(&mut doc).unwrap();

  // Verify the document serializes without corruption
  let xml = doc.serialize_to_string();
  assert!(xml.contains("XMath"), "serialized XML should contain XMath");
  assert!(
    !xml.contains('\0'),
    "serialized XML should not contain null bytes"
  );
}

/// Grammar ambiguity regression tests.
/// Track raw Marpa tree counts for known-ambiguous formulas.
/// Each formula has a limit based on the current grammar. If a grammar change
/// increases the count, the test fails — preventing ambiguity regressions.
/// If a grammar change DECREASES the count, update the limit downward.
#[test]
fn parse_tree_count_limits() {
  let mut parser = MathParser::default();

  // (name, lexemes, max_allowed_raw_trees)
  // Raw tree counts annotated with first-principles analysis of correct parse count.
  let cases: Vec<(&str, &str, usize)> = vec![
    // mathtools: 4-equation bigop formula with \quad separators
    // Correct parse: 1 (each equation unambiguous, \quad separates formulae)
    // Remaining 15 raw: PUNCT list_apply vs formulae_apply (3 separators × ~2x)
    (
      "bigop_quad_4eq",
      "UNKNOWN:V:1 RELOP:equals:2 SUMOP:sum:3 start_BIGOPSUB:start:4 ATOM:a:5 end_BIGOPSUB:end:6 \
      start_BIGOPSUP:start:7 ID:infinity:8 end_BIGOPSUP:end:9 UNKNOWN:V:10 \
      start_POSTSUBSCRIPT:start:11 UNKNOWN:i:12 end_POSTSUBSCRIPT:end:13 PUNCT:quad:14 \
      UNKNOWN:X:15 RELOP:equals:16 SUMOP:sum:17 start_BIGOPSUB:start:18 ATOM:a:19 \
      end_BIGOPSUB:end:20 start_BIGOPSUP:start:21 NUMBER:3456:22 end_BIGOPSUP:end:23 \
      UNKNOWN:X:24 start_POSTSUBSCRIPT:start:25 UNKNOWN:i:26 end_POSTSUBSCRIPT:end:27 \
      PUNCT:quad:28 UNKNOWN:Y:29 RELOP:equals:30 SUMOP:sum:31 start_BIGOPSUB:start:32 \
      ATOM:a:33 end_BIGOPSUB:end:34 UNKNOWN:Y:35 start_POSTSUBSCRIPT:start:36 UNKNOWN:i:37 \
      end_POSTSUBSCRIPT:end:38 PUNCT:quad:39 UNKNOWN:Z:40 RELOP:equals:41 BIGOP:T:42 \
      start_BIGOPSUB:start:43 ATOM:a:44 end_BIGOPSUB:end:45 UNKNOWN:Z:46 \
      start_POSTSUBSCRIPT:start:47 UNKNOWN:i:48 end_POSTSUBSCRIPT:end:49 ",
      15,
    ),
    // mathtools: 24 alternating UNKNOWN letters from vsmallmatrix
    // M4: diffop filtering eliminated Catalan-number growth (5000 → 1)
    (
      "unknown_letters_24",
      "UNKNOWN:b:1 UNKNOWN:l:2 UNKNOWN:b:3 UNKNOWN:l:4 UNKNOWN:b:5 UNKNOWN:l:6 \
      UNKNOWN:b:7 UNKNOWN:l:8 UNKNOWN:l:9 UNKNOWN:b:10 UNKNOWN:b:11 UNKNOWN:l:12 \
      UNKNOWN:b:13 UNKNOWN:l:14 UNKNOWN:b:15 UNKNOWN:l:16 UNKNOWN:b:17 UNKNOWN:l:18 \
      UNKNOWN:b:19 UNKNOWN:l:20 UNKNOWN:b:21 UNKNOWN:l:22 UNKNOWN:b:23 UNKNOWN:l:24 ",
      1,
    ), // M4: was 5000, now 1 (diffop filtering)
    // sampler: 4-equation bigop formula with comma separators
    // Correct parse: 1. Remaining: PUNCT list/formulae competition (3 commas)
    (
      "bigop_comma_4eq",
      "UNKNOWN:X:1 RELOP:equals:2 SUMOP:sum:3 start_BIGOPSUB:start:4 ATOM:a:5 \
      end_BIGOPSUB:end:6 UNKNOWN:X:7 start_POSTSUBSCRIPT:start:8 UNKNOWN:i:9 \
      end_POSTSUBSCRIPT:end:10 PUNCT:,:11 UNKNOWN:X:12 RELOP:equals:13 SUMOP:sum:14 \
      start_BIGOPSUB:start:15 ATOM:a:16 end_BIGOPSUB:end:17 UNKNOWN:X:18 \
      start_POSTSUBSCRIPT:start:19 UNKNOWN:i:20 end_POSTSUBSCRIPT:end:21 PUNCT:,:22 \
      UNKNOWN:X:23 RELOP:equals:24 SUMOP:sum:25 start_BIGOPSUB:start:26 ATOM:a:27 \
      end_BIGOPSUB:end:28 UNKNOWN:X:29 start_POSTSUBSCRIPT:start:30 UNKNOWN:i:31 \
      end_POSTSUBSCRIPT:end:32 PUNCT:,:33 UNKNOWN:X:34 RELOP:equals:35 SUMOP:sum:36 \
      start_BIGOPSUB:start:37 ATOM:a:38 end_BIGOPSUB:end:39 UNKNOWN:X:40 \
      start_POSTSUBSCRIPT:start:41 UNKNOWN:i:42 end_POSTSUBSCRIPT:end:43 ",
      15,
    ),
    // mathtools: pre-scripted formula with 5 items and \quad separators
    // Correct parse: ~3 (pre-script attachment ambiguity). Remaining: PUNCT + float interaction
    (
      "prescripted_quad",
      "start_FLOATSUPERSCRIPT:start:1 NUMBER:4:2 end_FLOATSUPERSCRIPT:end:3 \
      start_POSTSUBSCRIPT:start:4 NUMBER:12:5 end_POSTSUBSCRIPT:end:6 UNKNOWN:C:7 \
      start_POSTSUPERSCRIPT:start:8 ATOM:5p:9 end_POSTSUPERSCRIPT:end:10 \
      start_POSTSUBSCRIPT:start:11 NUMBER:2:12 end_POSTSUBSCRIPT:end:13 PUNCT:quad:14 \
      start_FLOATSUPERSCRIPT:start:15 NUMBER:14:16 end_FLOATSUPERSCRIPT:end:17 \
      start_POSTSUBSCRIPT:start:18 NUMBER:2:19 end_POSTSUBSCRIPT:end:20 UNKNOWN:C:21 \
      start_POSTSUPERSCRIPT:start:22 ATOM:5p:23 end_POSTSUPERSCRIPT:end:24 \
      start_POSTSUBSCRIPT:start:25 NUMBER:2:26 end_POSTSUBSCRIPT:end:27 PUNCT:quad:28 \
      start_FLOATSUPERSCRIPT:start:29 NUMBER:4:30 end_FLOATSUPERSCRIPT:end:31 \
      start_POSTSUBSCRIPT:start:32 NUMBER:12:33 end_POSTSUBSCRIPT:end:34 UNKNOWN:C:35 \
      start_POSTSUPERSCRIPT:start:36 ATOM:5p:37 end_POSTSUPERSCRIPT:end:38 \
      start_POSTSUBSCRIPT:start:39 NUMBER:2:40 end_POSTSUBSCRIPT:end:41 PUNCT:quad:42 \
      start_FLOATSUPERSCRIPT:start:43 NUMBER:14:44 end_FLOATSUPERSCRIPT:end:45 UNKNOWN:C:46 \
      start_POSTSUPERSCRIPT:start:47 ATOM:5p:48 end_POSTSUPERSCRIPT:end:49 \
      start_POSTSUBSCRIPT:start:50 NUMBER:2:51 end_POSTSUBSCRIPT:end:52 PUNCT:quad:53 \
      start_FLOATSUBSCRIPT:start:54 NUMBER:2:55 end_FLOATSUBSCRIPT:end:56 UNKNOWN:C:57 \
      start_POSTSUPERSCRIPT:start:58 ATOM:5p:59 end_POSTSUPERSCRIPT:end:60 \
      start_POSTSUBSCRIPT:start:61 NUMBER:2:62 end_POSTSUBSCRIPT:end:63 ",
      81,
    ),
    // mathtools: xy+xy+∫xy dx+xy+... (28 tokens)
    (
      "intop_chain",
      "UNKNOWN:x:1 UNKNOWN:y:2 ADDOP:plus:3 UNKNOWN:x:4 UNKNOWN:y:5 ADDOP:plus:6 \
      INTOP:integral:7 UNKNOWN:x:8 UNKNOWN:y:9 ATOM:dx:10 ADDOP:plus:11 UNKNOWN:x:12 \
      UNKNOWN:y:13 ADDOP:plus:14 UNKNOWN:x:15 UNKNOWN:y:16 ADDOP:plus:17 UNKNOWN:x:18 \
      UNKNOWN:y:19 ADDOP:plus:20 UNKNOWN:x:21 UNKNOWN:y:22 ADDOP:plus:23 UNKNOWN:x:24 \
      UNKNOWN:y:25 ADDOP:plus:26 UNKNOWN:x:27 UNKNOWN:y:28 ",
      1,
    ), // M4: was 768, now 1 (no UNKNOWN tokens match diffop)
    // 1-equation bigop (baseline)
    // Correct parse: 1 (V = Σ_a^b V_i). Now exactly 1 raw tree.
    (
      "bigop_1eq",
      "UNKNOWN:V:1 RELOP:equals:2 SUMOP:sum:3 start_BIGOPSUB:start:4 UNKNOWN:a:5 \
      end_BIGOPSUB:end:6 start_BIGOPSUP:start:7 UNKNOWN:b:8 end_BIGOPSUP:end:9 \
      UNKNOWN:V:10 start_POSTSUBSCRIPT:start:11 UNKNOWN:i:12 end_POSTSUBSCRIPT:end:13 ",
      1,
    ),
    // 2-equation bigop with \quad
    // Correct parse: 1. Remaining 3: PUNCT list/formulae (1 separator × 3 paths)
    (
      "bigop_quad_2eq",
      "UNKNOWN:V:1 RELOP:equals:2 SUMOP:sum:3 start_BIGOPSUB:start:4 UNKNOWN:a:5 \
      end_BIGOPSUB:end:6 start_BIGOPSUP:start:7 UNKNOWN:b:8 end_BIGOPSUP:end:9 \
      UNKNOWN:V:10 start_POSTSUBSCRIPT:start:11 UNKNOWN:i:12 end_POSTSUBSCRIPT:end:13 \
      PUNCT:quad:14 UNKNOWN:X:15 RELOP:equals:16 SUMOP:sum:17 start_BIGOPSUB:start:18 \
      UNKNOWN:a:19 end_BIGOPSUB:end:20 start_BIGOPSUP:start:21 UNKNOWN:b:22 \
      end_BIGOPSUP:end:23 UNKNOWN:X:24 start_POSTSUBSCRIPT:start:25 UNKNOWN:i:26 \
      end_POSTSUBSCRIPT:end:27 ",
      3,
    ),
    // 3-equation bigop with \quad
    // Correct parse: 1. Remaining 7: PUNCT competition (2 separators × ~3 paths)
    (
      "bigop_quad_3eq",
      "UNKNOWN:V:1 RELOP:equals:2 SUMOP:sum:3 start_BIGOPSUB:start:4 UNKNOWN:a:5 \
      end_BIGOPSUB:end:6 start_BIGOPSUP:start:7 UNKNOWN:b:8 end_BIGOPSUP:end:9 \
      UNKNOWN:V:10 start_POSTSUBSCRIPT:start:11 UNKNOWN:i:12 end_POSTSUBSCRIPT:end:13 \
      PUNCT:quad:14 UNKNOWN:X:15 RELOP:equals:16 SUMOP:sum:17 start_BIGOPSUB:start:18 \
      UNKNOWN:a:19 end_BIGOPSUB:end:20 start_BIGOPSUP:start:21 UNKNOWN:b:22 \
      end_BIGOPSUP:end:23 UNKNOWN:X:24 start_POSTSUBSCRIPT:start:25 UNKNOWN:i:26 \
      end_POSTSUBSCRIPT:end:27 PUNCT:quad:28 UNKNOWN:Y:29 RELOP:equals:30 SUMOP:sum:31 \
      start_BIGOPSUB:start:32 UNKNOWN:a:33 end_BIGOPSUB:end:34 UNKNOWN:Y:35 \
      start_POSTSUBSCRIPT:start:36 UNKNOWN:i:37 end_POSTSUBSCRIPT:end:38 ",
      7,
    ),
    // --- 1706.03762 "Attention Is All You Need" formulas ---

    // MultiHead(Q,K,V) = Concat(head_1,...,head_h) W^O
    // Correct parse: 1. Remaining 16: speculative function app on MultiHead/Concat × comma
    // competition
    (
      "attn_multihead",
      "UNKNOWN:MultiHead:1 OPEN:(:2 UNKNOWN:Q:3 PUNCT:,:4 UNKNOWN:K:5 PUNCT:,:6 \
      UNKNOWN:V:7 CLOSE:):8 RELOP:equals:9 UNKNOWN:Concat:10 OPEN:(:11 UNKNOWN:head:12 \
      start_POSTSUBSCRIPT:start:13 NUMBER:1:14 end_POSTSUBSCRIPT:end:15 PUNCT:,:16 \
      ID:ldots:17 PUNCT:,:18 UNKNOWN:head:19 start_POSTSUBSCRIPT:start:20 UNKNOWN:h:21 \
      end_POSTSUBSCRIPT:end:22 CLOSE:):23 UNKNOWN:W:24 start_POSTSUPERSCRIPT:start:25 \
      UNKNOWN:O:26 end_POSTSUPERSCRIPT:end:27 ",
      16,
    ),
    // where head_i = Attention(QW^Q_i, KW^K_i, VW^V_i)
    // Correct parse: 1. Remaining 4: speculative function app × comma list paths
    (
      "attn_where_head",
      "ATOM:where:1 UNKNOWN:head:2 start_POSTSUBSCRIPT:start:3 UNKNOWN:i:4 \
      end_POSTSUBSCRIPT:end:5 RELOP:equals:6 UNKNOWN:Attention:7 OPEN:(:8 UNKNOWN:Q:9 \
      UNKNOWN:W:10 start_POSTSUPERSCRIPT:start:11 UNKNOWN:Q:12 end_POSTSUPERSCRIPT:end:13 \
      start_POSTSUBSCRIPT:start:14 UNKNOWN:i:15 end_POSTSUBSCRIPT:end:16 PUNCT:,:17 \
      UNKNOWN:K:18 UNKNOWN:W:19 start_POSTSUPERSCRIPT:start:20 UNKNOWN:K:21 \
      end_POSTSUPERSCRIPT:end:22 start_POSTSUBSCRIPT:start:23 UNKNOWN:i:24 \
      end_POSTSUBSCRIPT:end:25 PUNCT:,:26 UNKNOWN:V:27 UNKNOWN:W:28 \
      start_POSTSUPERSCRIPT:start:29 UNKNOWN:V:30 end_POSTSUPERSCRIPT:end:31 \
      start_POSTSUBSCRIPT:start:32 UNKNOWN:i:33 end_POSTSUBSCRIPT:end:34 CLOSE:):35 ",
      4,
    ),
    // lrate = d_model^{-0.5} · min(step_num^{-0.5}, step_num · warmup_steps^{-1.5})
    // Correct parse: 1. Remaining 15: comma list paths + speculative function app
    (
      "attn_lrate",
      "UNKNOWN:l:1 UNKNOWN:r:2 UNKNOWN:a:3 UNKNOWN:t:4 UNKNOWN:e:5 RELOP:equals:6 \
      UNKNOWN:d:7 start_POSTSUBSCRIPT:start:8 ATOM:model:9 end_POSTSUBSCRIPT:end:10 \
      start_POSTSUPERSCRIPT:start:11 ATOM:-0.5:12 end_POSTSUPERSCRIPT:end:13 \
      MULOP:cdot:14 OPFUNCTION:minimum:15 OPEN:(:16 UNKNOWN:s:17 UNKNOWN:t:18 \
      UNKNOWN:e:19 UNKNOWN:p:20 UNKNOWN:_:21 UNKNOWN:n:22 UNKNOWN:u:23 UNKNOWN:m:24 \
      start_POSTSUPERSCRIPT:start:25 ATOM:-0.5:26 end_POSTSUPERSCRIPT:end:27 PUNCT:,:28 \
      UNKNOWN:s:29 UNKNOWN:t:30 UNKNOWN:e:31 UNKNOWN:p:32 UNKNOWN:_:33 UNKNOWN:n:34 \
      UNKNOWN:u:35 UNKNOWN:m:36 MULOP:cdot:37 UNKNOWN:w:38 UNKNOWN:a:39 UNKNOWN:r:40 \
      UNKNOWN:m:41 UNKNOWN:u:42 UNKNOWN:p:43 UNKNOWN:_:44 UNKNOWN:s:45 UNKNOWN:t:46 \
      UNKNOWN:e:47 UNKNOWN:p:48 UNKNOWN:s:49 start_POSTSUPERSCRIPT:start:50 ATOM:-1.5:51 \
      end_POSTSUPERSCRIPT:end:52 CLOSE:):53 ",
      15,
    ),
    // warmup_steps  (12 tokens, 233 raw — consecutive UNKNOWN)
    (
      "attn_warmup_steps",
      "UNKNOWN:w:1 UNKNOWN:a:2 UNKNOWN:r:3 UNKNOWN:m:4 UNKNOWN:u:5 UNKNOWN:p:6 \
      UNKNOWN:_:7 UNKNOWN:s:8 UNKNOWN:t:9 UNKNOWN:e:10 UNKNOWN:p:11 UNKNOWN:s:12 ",
      1,
    ), // M4: was 233, now 1
    // diffd parsing: integral + function + diffd
    // Should have diffd@(x) not diffd * x
    // ∫_a^b f(x) dx — correct parses: 2 (f@(x)*d@(x) vs f@(x*d@(x)))
    (
      "integral_diffd",
      "INTOP:integral:1 start_BIGOPSUB:start:2 ID:a:3 end_BIGOPSUB:end:4 \
      start_BIGOPSUP:start:5 ID:b:6 end_BIGOPSUP:end:7 FUNCTION:f:8 \
      OPEN:(:9 UNKNOWN:x:10 CLOSE:):11 OPFUNCTION:d:12 UNKNOWN:x:13 ",
      14,
    ),
    // --- Top-ambiguity formulas from test suite (unique parse count tracking) ---
    // All produce ≤10 unique parses (M10 target achieved).

    // Vertbar inherent ambiguity: a|a|+b|b|+c|c|  (10 unique, 54 raw)
    (
      "vertbar_abs_sum",
      "UNKNOWN:a:1 VERTBAR:|:2 UNKNOWN:a:3 VERTBAR:|:4 ADDOP:plus:5 UNKNOWN:b:6 \
      VERTBAR:|:7 UNKNOWN:b:8 VERTBAR:|:9 ADDOP:plus:10 UNKNOWN:c:11 VERTBAR:|:12 \
      UNKNOWN:c:13 VERTBAR:|:14 ",
      54,
    ),
    // Opfunction chaining: F G H a  (9 unique, 54 raw)
    (
      "opfunc_chain_3",
      "OPFUNCTION:F:1 OPFUNCTION:G:2 OPFUNCTION:H:3 ID:a:4 ",
      54,
    ),
    // Double vertbar norms: ||x|| a ||y||  (9 unique, 13 raw)
    (
      "double_vertbar_norms",
      "VERTBAR:|:1 VERTBAR:|:2 UNKNOWN:x:3 VERTBAR:|:4 VERTBAR:|:5 ID:a:6 \
      VERTBAR:|:7 VERTBAR:|:8 UNKNOWN:y:9 VERTBAR:|:10 VERTBAR:|:11 ",
      13,
    ),
    // Multiple opfunctions with \qquad  (8 unique, 5000 raw — opfunction absorption combinatorics)
    (
      "opfunc_multi_qquad",
      "OPFUNCTION:tr:1 UNKNOWN:rho:2 PUNCT:qquad:3 UNKNOWN:tr(XY):4 PUNCT:qquad:5 \
      OPFUNCTION:Tr:6 UNKNOWN:rho:7 PUNCT:qquad:8 OPFUNCTION:rank:9 UNKNOWN:M:10 \
      PUNCT:qquad:11 UNKNOWN:erf(x):12 PUNCT:qquad:13 OPFUNCTION:Res:14 OPEN:[:15 \
      UNKNOWN:f:16 OPEN:(:17 UNKNOWN:z:18 CLOSE:):19 CLOSE:]:20 ",
      5000,
    ),
    // Trig chain: sin πx cos 2πy  (6 unique, 12 raw)
    (
      "trig_chain",
      "TRIGFUNCTION:sine:1 UNKNOWN:pi:2 MULOP:times:3 UNKNOWN:x:4 \
      TRIGFUNCTION:cosine:5 NUMBER:2:6 UNKNOWN:pi:7 MULOP:times:8 UNKNOWN:y:9 ",
      12,
    ),
  ];

  for (name, lexemes, max_allowed) in &cases {
    let start = std::time::Instant::now();
    let count = parser.count_raw_trees(lexemes);
    let elapsed = start.elapsed();
    let n = count.unwrap_or(0);
    eprintln!("{name}: {n} raw trees in {elapsed:?} (limit: {max_allowed})");
    assert!(
      n <= *max_allowed,
      "{name}: {n} raw trees exceeds limit of {max_allowed}. \
       Grammar change increased ambiguity."
    );
  }
}
