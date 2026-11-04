//! Red/green guards for batch 102a: authblk's `\affil` (authblk_sty.rs) is locked, as the kernel's `\title`/`\author`
//! are, so a document's or raw class's own `\affil` — the JOSS template's copy of authblk's code (paper_joss.tex:162-182;
//! 2504.08081, 2310.11002, 2107.08407, 2507.13341, 2604.08855), oasics's and eurocg25's after
//! `\RequirePackage{authblk}` (1709.00722, 2503.02336; dagman.cls the same, 1701.09007) — is refused, and its body replayed for its side effects
//! (`\lx@replay@class@setter`, OXIDIZED_DESIGN #482): the affiliations stay the binding's contacts
//! (OXIDIZED_DESIGN_DIVERGENCES #959). Before, the copy replaced the binding's and put every affiliation in authblk's
//! `\AB@authors`, which nothing reads here: lost, with 0 Errors (Perl loses them too, with 6-8 Errors).
use super::{
  convert_files_with, convert_with,
  perfect_kernel_batch46::{error_count, warning_count},
};

/// The whole run of creators, whitespace squashed.
fn creators(xml: &str) -> String {
  let xml = xml.split_whitespace().collect::<Vec<_>>().join(" ");
  let start = xml.find("<creator").expect("a creator");
  let end = xml.rfind("</creator>").expect("a closed creator") + "</creator>".len();
  xml[start..end].to_string()
}

fn affiliation(text: &str) -> String {
  format!(r#"<contact name="Affiliation: " role="affiliation">{text}</contact>"#)
}

/// The JOSS shape: a document's `\renewcommand{\affil}[2][]` copying authblk's code. Ann Able keeps both affiliations
/// her marks pick, in order, exactly as without the copy (the control, which the lock leaves as it was). Red before:
/// `<creator role="author"><personname>Ann Able</personname></creator>`.
#[test]
fn a_documents_copy_of_authblk_affil_keeps_the_bindings_affiliations() {
  let expected = format!(
    r#"<creator role="author"> <personname>Ann Able</personname> {} {} </creator>"#,
    affiliation("Astrophysics Group, Cavendish Laboratory, Cambridge"),
    affiliation("Kavli Institute for Cosmology, Cambridge")
  );
  for tex in [
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/authblk_affil_copy/document_copy.tex"
    ),
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/authblk_affil_copy/no_copy_control.tex"
    ),
  ] {
    let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
    assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
    assert_eq!(creators(&xml), expected);
  }
}

/// A raw class's copy after `\RequirePackage{authblk}` (oasics.cls's shape, its `\@author` laying out `\AB@affillist`):
/// each author keeps the affiliation its mark picks, and no unclaimed-text note is left (red before: both
/// affiliations a `frontmatter:unclaimed` note of the author block, 1 Warning). A class path, distinct from the
/// document's: the lock refuses a file's definition at the class origin, and the class's title-page layout reads the
/// accumulators the replay fills. (`before`: authblk's separator between the two names, as without the copy.)
#[test]
fn a_raw_class_copy_of_authblk_affil_keeps_the_bindings_affiliations() {
  let (log, xml) = convert_files_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/authblk_affil_copy/class_copy.tex"
    ),
    &[(
      "abcopy.cls",
      include_str!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/authblk_affil_copy/abcopy.cls"
      ),
    )],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(
    creators(&xml),
    format!(
      r#"<creator role="author"> <personname>Ann Able</personname> {} </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> {} </creator>"#,
      affiliation("Dept. of Widgets, Univ X"),
      affiliation("Inst. of Gadgets, Univ Y")
    )
  );
}

/// The replay of the refused body: a document's `\def\affil#1{\gdef\lastaff{#1}}` keeps the store its body makes,
/// which the document prints ("Last: Univ X."), while the affiliation stays the binding's contact.
#[test]
fn a_refused_affil_still_runs_for_its_stores() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/authblk_affil_copy/tex_def_side_effect.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(
    creators(&xml),
    format!(
      r#"<creator role="author"> <personname>Ann Able</personname> {} </creator>"#,
      affiliation("Univ X")
    )
  );
  assert!(
    xml.contains(r#"<p xml:id="p1.1">Last: Univ X.</p>"#),
    "{xml}"
  );
}
