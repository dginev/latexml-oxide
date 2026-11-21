//! Red/green guard for batch REGFIXX: a name of amsart's that a document's own `\@setauthors` calls, which the binding
//! runs at `\maketitle` since REGFIXM2.
use latexml::util::test::{assert_element, convert_with};

use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch84b::creators,
};

/// sectioning-frontmatter/amsart_document_setauthors_calls_setthanks: a document's `\@setauthors` ending with
/// `\@setthanks`, as amsart's own does (amsart.cls:836, :808; 2606.04416): the faithful `\@setthanks` prints the
/// binding's empty `\thankses`, so the authors are kept and the thanks text is one pubnote. Red (INT6D): 1 Error
/// (`\@setthanks` undefined). Distinct from `perfect_kernel_batchregfixm::amsart_setauthors_additions_are_printed`
/// (appended text): a class setter the store names.
#[test]
fn amsart_document_setauthors_calls_setthanks() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/amsart_document_setauthors_calls_setthanks.tex"
  );
  let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(
    creators(&xml),
    vec![
      vec!["Ann Able".to_string(), "email=ann@x.edu".to_string()],
      vec!["Bob Baker".to_string()]
    ],
    "{xml}"
  );
  assert_element(
    &xml,
    "pubnote",
    &[r#"role="thanks""#],
    r#"<pubnote role="thanks">Supported by grant 42.</pubnote>"#,
  );
  assert_eq!(xml.matches("Supported by grant 42").count(), 1, "{xml}");
  assert!(!xml.contains("<note"), "{xml}");
}
