//! Red/green guards for batch MANFM, two frontmatter regressions of the TeX Live manual sweep: a class's own
//! `\@setauthors` run by the ams binding (ijmart/ijmsample), and the default e-mail label digested with no e-mail
//! (lineno/lineno, lineno/ulineno); round 2 (MANFM2): LaTeXML's default labels beside a document's `\def~` (lineno/lineno).
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
  assert_element(
    &xml,
    "creator",
    &[r#"role="author""#],
    r#"<creator role="author">
    <personname>A</personname>
  </creator>"#,
  );
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

/// sectioning-frontmatter/default_{affiliation,email}_label_tilde_redefined*, default_date_keywords_labels_*: LaTeXML's
/// own default labels are spelled with `\lx@nobreakspace` (OD #1136; article.cls:457 `\figurename\nobreakspace`), so a
/// document's `\def~{\verb~}` (lineno.tex:20) reaches none of them: each name is the word and U+00A0. Red (MANFM
/// 945eab9385): 3, 1 and 2 Errors (`\verb illegal in argument`), names `Affiliation:\verb~~` and the like (Perl the same
/// names, no Error). Distinct from `default_email_label_digested_only_where_a_contact_takes_it`: a label that IS taken.
#[test]
fn default_labels_survive_a_redefined_tilde() {
  let xml = clean(repro!(
    "sectioning-frontmatter",
    "default_affiliation_label_tilde_redefined.tex"
  ));
  assert_element(
    &xml,
    "contact",
    &[r#"role="affiliation""#],
    "<contact class=\"ltx_author_line\" name=\"Affiliation:\u{a0}\" role=\"affiliation\">B</contact>",
  );
  assert_eq!(
    creators(&xml),
    vec![vec!["A".to_string(), "affiliation=B".to_string()]],
    "{xml}"
  );
  let xml = clean(repro!(
    "sectioning-frontmatter",
    "default_email_label_tilde_redefined_used.tex"
  ));
  assert_element(
    &xml,
    "contact",
    &[r#"role="email""#],
    "<contact name=\"Email:\u{a0}\" role=\"email\">a@b.org</contact>",
  );
  assert_eq!(
    creators(&xml),
    vec![vec!["A".to_string(), "email=a@b.org".to_string()]],
    "{xml}"
  );
  let xml = clean(repro!(
    "sectioning-frontmatter",
    "default_date_keywords_labels_tilde_redefined.tex"
  ));
  assert_element(
    &xml,
    "date",
    &[r#"role="received""#],
    "<date name=\"Received\u{a0}\" role=\"received\">1 May 2020</date>",
  );
  assert_element(
    &xml,
    "keywords",
    &[],
    "<keywords name=\"Keywords:\u{a0}\">k1, k2</keywords>",
  );
}
