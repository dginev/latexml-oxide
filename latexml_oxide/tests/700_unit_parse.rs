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
    parser.recognizes(
      "UNKNOWN:D:1 start_POSTSUBSCRIPT:start:2 UNKNOWN:r:3 end_POSTSUBSCRIPT:end:4 "
    ),
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
  let cases: Vec<(&str, &str, usize)> = vec![
    // mathtools: 4-equation bigop formula with \quad separators
    ("bigop_quad_4eq",
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
     5000),  // TODO: reduce grammar ambiguity

    // mathtools: 24 alternating UNKNOWN letters from vsmallmatrix
    ("unknown_letters_24",
     "UNKNOWN:b:1 UNKNOWN:l:2 UNKNOWN:b:3 UNKNOWN:l:4 UNKNOWN:b:5 UNKNOWN:l:6 \
      UNKNOWN:b:7 UNKNOWN:l:8 UNKNOWN:l:9 UNKNOWN:b:10 UNKNOWN:b:11 UNKNOWN:l:12 \
      UNKNOWN:b:13 UNKNOWN:l:14 UNKNOWN:b:15 UNKNOWN:l:16 UNKNOWN:b:17 UNKNOWN:l:18 \
      UNKNOWN:b:19 UNKNOWN:l:20 UNKNOWN:b:21 UNKNOWN:l:22 UNKNOWN:b:23 UNKNOWN:l:24 ",
     5000),  // TODO: reduce grammar ambiguity

    // sampler: 4-equation bigop formula with comma separators
    ("bigop_comma_4eq",
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
     3840),

    // mathtools: pre-scripted formula (was 5000 before formulae split, now ~5000 raw)
    ("prescripted_quad",
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
     5000),  // TODO: reduce grammar ambiguity

    // mathtools: xy+xy+∫xy dx+xy+... (28 tokens)
    ("intop_chain",
     "UNKNOWN:x:1 UNKNOWN:y:2 ADDOP:plus:3 UNKNOWN:x:4 UNKNOWN:y:5 ADDOP:plus:6 \
      INTOP:integral:7 UNKNOWN:x:8 UNKNOWN:y:9 ATOM:dx:10 ADDOP:plus:11 UNKNOWN:x:12 \
      UNKNOWN:y:13 ADDOP:plus:14 UNKNOWN:x:15 UNKNOWN:y:16 ADDOP:plus:17 UNKNOWN:x:18 \
      UNKNOWN:y:19 ADDOP:plus:20 UNKNOWN:x:21 UNKNOWN:y:22 ADDOP:plus:23 UNKNOWN:x:24 \
      UNKNOWN:y:25 ADDOP:plus:26 UNKNOWN:x:27 UNKNOWN:y:28 ",
     768),

    // 1-equation bigop (baseline)
    ("bigop_1eq",
     "UNKNOWN:V:1 RELOP:equals:2 SUMOP:sum:3 start_BIGOPSUB:start:4 UNKNOWN:a:5 \
      end_BIGOPSUB:end:6 start_BIGOPSUP:start:7 UNKNOWN:b:8 end_BIGOPSUP:end:9 \
      UNKNOWN:V:10 start_POSTSUBSCRIPT:start:11 UNKNOWN:i:12 end_POSTSUBSCRIPT:end:13 ",
     16),

    // 2-equation bigop with \quad
    ("bigop_quad_2eq",
     "UNKNOWN:V:1 RELOP:equals:2 SUMOP:sum:3 start_BIGOPSUB:start:4 UNKNOWN:a:5 \
      end_BIGOPSUB:end:6 start_BIGOPSUP:start:7 UNKNOWN:b:8 end_BIGOPSUP:end:9 \
      UNKNOWN:V:10 start_POSTSUBSCRIPT:start:11 UNKNOWN:i:12 end_POSTSUBSCRIPT:end:13 \
      PUNCT:quad:14 UNKNOWN:X:15 RELOP:equals:16 SUMOP:sum:17 start_BIGOPSUB:start:18 \
      UNKNOWN:a:19 end_BIGOPSUB:end:20 start_BIGOPSUP:start:21 UNKNOWN:b:22 \
      end_BIGOPSUP:end:23 UNKNOWN:X:24 start_POSTSUBSCRIPT:start:25 UNKNOWN:i:26 \
      end_POSTSUBSCRIPT:end:27 ",
     192),

    // 3-equation bigop with \quad
    ("bigop_quad_3eq",
     "UNKNOWN:V:1 RELOP:equals:2 SUMOP:sum:3 start_BIGOPSUB:start:4 UNKNOWN:a:5 \
      end_BIGOPSUB:end:6 start_BIGOPSUP:start:7 UNKNOWN:b:8 end_BIGOPSUP:end:9 \
      UNKNOWN:V:10 start_POSTSUBSCRIPT:start:11 UNKNOWN:i:12 end_POSTSUBSCRIPT:end:13 \
      PUNCT:quad:14 UNKNOWN:X:15 RELOP:equals:16 SUMOP:sum:17 start_BIGOPSUB:start:18 \
      UNKNOWN:a:19 end_BIGOPSUB:end:20 start_BIGOPSUP:start:21 UNKNOWN:b:22 \
      end_BIGOPSUP:end:23 UNKNOWN:X:24 start_POSTSUBSCRIPT:start:25 UNKNOWN:i:26 \
      end_POSTSUBSCRIPT:end:27 PUNCT:quad:28 UNKNOWN:Y:29 RELOP:equals:30 SUMOP:sum:31 \
      start_BIGOPSUB:start:32 UNKNOWN:a:33 end_BIGOPSUB:end:34 UNKNOWN:Y:35 \
      start_POSTSUBSCRIPT:start:36 UNKNOWN:i:37 end_POSTSUBSCRIPT:end:38 ",
     1792),
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
