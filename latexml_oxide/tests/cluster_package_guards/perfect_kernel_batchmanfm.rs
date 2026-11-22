//! Red/green guards for batch MANFM, two frontmatter regressions of the TeX Live manual sweep: a class's own
//! `\@setauthors` run by the ams binding (ijmart/ijmsample), and the default e-mail label digested with no e-mail
//! (lineno/lineno, lineno/ulineno).
use latexml::util::test::{assert_element, convert_with, kpse_has};

use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch84b::creators,
};

macro_rules! repro {
  ($dir:literal, $path:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/",
      $dir,
      "/",
      $path
    ))
  };
}

/// Converts `tex` with ar5iv.sty preloaded and checks it converts clean; returns the XML.
#[track_caller]
fn clean(tex: &str) -> String {
  let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  xml
}

/// class-bindings/ijmart_setauthors_*: a class built on amsart redefines `\@setauthors` (ijmart.cls:229-244, its
/// `\if\authors\@empty\relax\else` with no `\fi`). The ams binding runs only what was added after the class's load
/// ended: nothing for the class alone, a note for the document's appended text. Red (4a74afcbfd): 2 Errors
/// (`expected:\fi`, `\endgroup` in internal_vertical) + 1 Warning, the class's setter run with `\authors` emptied.
/// Distinct from `perfect_kernel_batchregfixm::amsart_setauthors_additions_are_printed` (amsart's own empty store): a
/// class's store, its tail past the class's definition. Skipped visibly where ijmart.cls is absent.
#[test]
fn ams_based_class_setauthors_runs_only_the_document_additions() {
  if !kpse_has("ijmart.cls") {
    eprintln!("SKIP ijmart_setauthors: ijmart.cls is not in this TeX Live");
    return;
  }
  let xml = clean(repro!("class-bindings", "ijmart_setauthors_class_own.tex"));
  assert_eq!(creators(&xml), vec![vec!["A".to_string()]], "{xml}");
  assert!(!xml.contains("<note") && !xml.contains("BY"), "{xml}");
  let xml = clean(repro!(
    "class-bindings",
    "ijmart_setauthors_document_adds.tex"
  ));
  assert_eq!(creators(&xml), vec![vec!["A".to_string()]], "{xml}");
  assert_element(
    &xml,
    "creator",
    &[r#"role="author""#],
    r#"<creator role="author">
    <personname>A</personname>
    <note role="note" xml:id="id1.1">Supported by grant 42.</note>
  </creator>"#,
  );
  assert!(!xml.contains("BY"), "{xml}");
  assert_eq!(xml.matches("Supported by grant 42.").count(), 1, "{xml}");
}

/// sectioning-frontmatter/default_email_label_tilde_redefined*: the steps after the frontmatter is placed digest the
/// default e-mail label (`Email:~`) only for a contact that takes it, as Perl digests a name only for an entry it adds
/// (Base_Utility.pool.ltxml:408): a document's `\def~{\verb~}` (lineno.tex:20, ulineno.tex:33) meets it neither with no
/// e-mail nor with one its class names (amsart's "Email address: "). Red (4a74afcbfd): 1 Error each (`\verb illegal in
/// argument`).
#[test]
fn default_email_label_digested_only_where_a_contact_takes_it() {
  let xml = clean(repro!(
    "sectioning-frontmatter",
    "default_email_label_tilde_redefined.tex"
  ));
  assert_element(&xml, "title", &[], "<title>T</title>");
  assert_element(&xml, "p", &[], r#"<p xml:id="p1.1">x</p>"#);
  let xml = clean(repro!(
    "sectioning-frontmatter",
    "default_email_label_tilde_redefined_own_label.tex"
  ));
  assert_element(
    &xml,
    "contact",
    &[r#"role="email""#],
    r#"<contact name="Email address: " role="email"><text font="typewriter" xml:id="id1">a@b.org</text></contact>"#,
  );
  assert_eq!(
    creators(&xml),
    vec![vec!["A".to_string(), "email=a@b.org".to_string()]],
    "{xml}"
  );
}
