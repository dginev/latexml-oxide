use latexml::core_interface::DigestionAPI;
use latexml::util::test::new_test_engine;

#[test]
fn footnotetext_with_matching_nested_mark_does_not_self_relocate() {
  let tex = concat!(
    r"\begin{document}",
    r"Author\footnotemark[1].",
    r"\footnotetext{\footnotemark[1]Electronic address: author@example.test}",
    r"\end{document}",
  );
  let mut latexml = new_test_engine();
  let mut doc = latexml
    .convert_file(format!("literal:{tex}"))
    .expect("footnote self-relocation regression should convert");

  let notes = doc.findnodes("//ltx:note", None);
  assert!(
    notes.len() <= 3,
    "self-relocation should not grow note nodes unexpectedly; got {}",
    notes.len()
  );

  let xml = doc.serialize_to_string();
  assert!(xml.contains("Electronic address: author@example.test"));
}
