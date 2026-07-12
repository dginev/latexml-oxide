//! Integration tests for the latexml_post processing pipeline.
//!
//! Port of LaTeXML/t/90_latexmlpost.t
//! For each test `$name` there should be `$name.xml` (input) and
//! `$name-post.xml` (expected output from `latexmlpost --keepXMath --pmml`).

use latexml_post::{
  Post,
  document::{PostDocument, PostDocumentOptions},
  mathml::MathML,
  processor::Processor,
};

const DIR: &str = "tests/post";

/// Normalize an XML file exactly the way `xmllint --format` does: parse and
/// re-serialize through libxml2's pretty-printer (the same C code path —
/// xmllint IS libxml2). Returns the formatted serialization split into lines.
///
/// Signal integrity: the predecessor of this helper piped through
/// `bash -c "diff <(xmllint --format …)"`, which vacuously PASSED with empty
/// output when xmllint was missing (this hid two stale goldens until the
/// macOS runner — which has xmllint — failed honestly, 2026-07-03), and
/// required a Unix userland (bash, xmllint, diff, grep, wc) that native
/// Windows lacks. In-process parsing fails toward flagging by construction:
/// a missing or malformed file panics instead of comparing empty-to-empty.
fn xmllint_format(path: &str) -> String {
  // no_blanks mirrors xmllint --format's xmlKeepBlanksDefault(0): blank
  // text nodes are dropped at parse time, which is what licenses libxml2's
  // pretty-printer to re-indent element-only content. Without it, files
  // whose stored indentation differs (compact vs indented) diff on pure
  // whitespace despite identical XML.
  let doc = libxml::parser::Parser::default()
    .parse_file_with_options(
      path,
      libxml::parser::ParserOptions {
        no_blanks: true,
        ..libxml::parser::ParserOptions::default()
      },
    )
    .unwrap_or_else(|e| panic!("Failed to parse {path} for normalized comparison: {e:?}"));
  doc.to_string_with_options(libxml::tree::SaveOptions {
    format: true,
    ..libxml::tree::SaveOptions::default()
  })
}

/// Run a post-processing test: read input XML, apply PMML conversion with
/// keepXMath, compare against reference via libxml2-normalized line diff.
fn post_test(name: &str, max_allowed_diffs: usize) {
  let input_path = format!("{}/{}.xml", DIR, name);
  let reference_path = format!("{}/{}-post.xml", DIR, name);

  let input = std::fs::read_to_string(&input_path)
    .unwrap_or_else(|e| panic!("Failed to read {}: {}", input_path, e));

  let doc = PostDocument::new_from_string(&input, PostDocumentOptions::default())
    .unwrap_or_else(|e| panic!("Failed to parse {}: {}", input_path, e));

  let mut post = Post::new();
  let pmml = MathML::new_presentation().with_keep_xmath(true);
  let mut processors: Vec<Box<dyn Processor>> = vec![Box::new(pmml)];

  let results = post
    .process_chain(vec![doc], &mut processors)
    .expect("post-processing failed");

  assert_eq!(results.len(), 1, "Expected 1 output document");

  let result_doc = &results[0];
  let actual = result_doc.to_xml_string();

  // Save actual output for debugging
  let actual_path = format!("{}/{}-post-actual.xml", DIR, name);
  std::fs::write(&actual_path, &actual).ok();

  // Normalized comparison: libxml2 pretty-print both sides (the exact
  // `xmllint --format` normalization), then LCS line-diff. The count
  // matches the old `diff | grep '^[<>]' | wc -l` semantics: every
  // inserted or deleted line counts one (a changed line counts two).
  let actual_formatted = xmllint_format(&actual_path);
  let reference_formatted = xmllint_format(&reference_path);
  let diff = similar::TextDiff::from_lines(&actual_formatted, &reference_formatted);
  let diff_count = diff
    .iter_all_changes()
    .filter(|c| c.tag() != similar::ChangeTag::Equal)
    .count();

  if diff_count > max_allowed_diffs {
    // Show the leading changed lines for debugging (old behavior: head -40
    // of the raw diff).
    let detail: Vec<String> = diff
      .iter_all_changes()
      .filter(|c| c.tag() != similar::ChangeTag::Equal)
      .take(40)
      .map(|c| {
        let sigil = match c.tag() {
          similar::ChangeTag::Delete => '<',
          similar::ChangeTag::Insert => '>',
          similar::ChangeTag::Equal => unreachable!(),
        };
        format!("{} {}", sigil, c.value().trim_end_matches('\n'))
      })
      .collect();

    panic!(
      "Post-processing output for '{}' has {} diff lines (max allowed: {}).\n\n{}\n",
      name,
      diff_count,
      max_allowed_diffs,
      detail.join("\n")
    );
  } else {
    eprintln!(
      "{}: {} diff lines (max allowed: {})",
      name, diff_count, max_allowed_diffs
    );
  }
}

#[test]
fn simplemath_post_test() {
  // 4 diff lines: spacing adjustments for && (BINOP) and !! (POSTFIX)
  // where our spacing algorithm adds spacing that Perl's doesn't
  post_test("simplemath", 4);
}

#[test]
fn opdecoration_post_test() {
  // FUNCTION APPLICATION (⁡) over-insertion: an operator whose presentation is an
  // <m:mo> (∇, ∂, ∑, ∫, …) must juxtapose its argument (∇ϕ, ∂f, ∑a, ∫g) — NOT
  // emit ∇⁡ϕ — matching Perl's is_mo rule (MathML.pm Apply:?:?). Regression guard
  // for presentation.rs op_base_is_mo.
  post_test("opdecoration", 0);
}

#[test]
fn hyperref_post_test() { post_test("hyperref", 0); }

#[test]
fn mathgolden_post_test() {
  // The MathML-post audit golden set (PR_READINESS): mathstyle transitions
  // (\tfrac/\dfrac/\displaystyle), inherited context color on frac/cancel/
  // sqrt/tokens, menclose (\boxed), minsize/maxsize (\bigl), author spacing
  // (\, \! \qquad), Inner-Punct array-comma spacing, movablelimits (lim/sum),
  // cfrac nesting and nth-root order. Golden generated by REFERENCE-tree
  // Perl latexmlpost --keepXMath --pmml over the identical core XML.
  // ZERO diff lines: byte-identical to reference-tree Perl.
  post_test("mathgolden", 0);
}
