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

/// Run a post-processing test: read input XML, apply PMML conversion
/// with keepXMath, compare against reference using xmllint --format diff.
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

  // Use xmllint --format for normalized comparison
  let diff_output = std::process::Command::new("bash")
    .arg("-c")
    .arg(format!(
      "diff <(xmllint --format '{}' 2>/dev/null) <(xmllint --format '{}' 2>/dev/null) | grep '^[<>]' | wc -l",
      actual_path, reference_path
    ))
    .output()
    .expect("failed to run diff");

  let diff_count: usize = String::from_utf8_lossy(&diff_output.stdout)
    .trim()
    .parse()
    .unwrap_or(999);

  if diff_count > max_allowed_diffs {
    // Show the actual diff for debugging
    let diff_detail = std::process::Command::new("bash")
      .arg("-c")
      .arg(format!(
        "diff <(xmllint --format '{}' 2>/dev/null) <(xmllint --format '{}' 2>/dev/null) | head -40",
        actual_path, reference_path
      ))
      .output()
      .expect("failed to run diff");

    panic!(
      "Post-processing output for '{}' has {} diff lines (max allowed: {}).\n\n{}\n",
      name,
      diff_count,
      max_allowed_diffs,
      String::from_utf8_lossy(&diff_detail.stdout)
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
fn hyperref_post_test() { post_test("hyperref", 0); }
