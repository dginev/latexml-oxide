//! Guards for REGBIND: the post-cutoff sandbox reruns' new Errors of the bindings (CUTSBX clusters A, D, F).
//! - A binding defines the macros its real class or package defines, which documents and other packages extend with
//!   `\g@addto@macro`: an undefined one stays an Error, as in TeX (OXIDIZED_DESIGN_DIVERGENCES #1036), so the binding,
//!   not the Error, is what changes. One row per binding: each is its own code path (two classes' and four packages'
//!   bindings, revtex's shared helper, icml's version gate).
//! - cleveref's binding re-applies the kernel first aid's repair of the raw cleveref.sty's amsmath code (content: an
//!   explicit `\ltx@label` lost its key's first character to the text).
//! - pdfcomment's binding allocates the soulpos registers a document's copy of soulpos's end pass reads.
//!
//! Each converts its repro and asserts the exact Error and Warning counts, every creator's name and contacts in order
//! where the class has a frontmatter, and the elements the addition shows in, whole.

use latexml::util::test::{convert_with, error_count, warning_count};

use super::perfect_kernel_batch84b::creators;

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!("../../../tools/perfect_kernel/repros/", $path))
  };
}

fn run(tex: &str) -> (String, String) { convert_with(tex, Some("ar5iv.sty")) }

struct Row {
  name:     &'static str,
  tex:      &'static str,
  creators: &'static [&'static [&'static str]],
  elements: &'static [&'static str],
}

const BINDING_ROWS: &[Row] = &[
  // acmart.cls:1707, :1716, :1726 the title page's note lists; :2894 no address list at the end, where a document's
  // `\@authornotemark` would add to `\@currentauthors`, which only `\maketitle` defines (2605.28175, 2605.16813).
  Row {
    name:     "acmart",
    tex:      repro!("class-bindings/acmart_note_lists_are_the_class_macros.tex"),
    creators: &[&["Ann Able", "affiliation=Univ A , C"]],
    elements: &[r#"<p xml:id="p1.1">Text.</p>"#],
  },
  // elsarticle.cls:178-179, :294 (2605.08232, 2606.15628's cnf.sty `\cortext`; 2605.22682).
  Row {
    name:     "elsarticle",
    tex:      repro!("class-bindings/elsarticle_note_and_address_lists_are_the_class_macros.tex"),
    creators: &[&["Ann Able", "address=Univ A"]],
    elements: &[r#"<p xml:id="p1.1">Text.</p>"#],
  },
  // revtex4-2.cls:2497-2499 `\@FMN@list` at `\begin{document}`, :2656-2657, :7201 (2605.00976, 2606.06621).
  Row {
    name:     "revtex4-2",
    tex:      repro!("class-bindings/revtex_frontmatter_note_lists_are_the_class_macros.tex"),
    creators: &[&["Ann Able", "affiliation=Univ A"]],
    elements: &[r#"<p xml:id="p1.1">Text.</p>"#],
  },
  // oup-authoring-template.cls:813 `\ps@opening` (2606.29739).
  Row {
    name:     "oup-authoring-template",
    tex:      repro!("class-bindings/oup_opening_page_style_is_the_class_macro.tex"),
    creators: &[&["Ann Able"]],
    elements: &[r#"<p xml:id="p1.1">Text.</p>"#],
  },
  // enumerate.sty:66 `\@enhook` (2606.12178, 2606.17008 through misccorr.sty).
  Row {
    name:     "enumerate",
    tex:      repro!("package-bindings/enumerate_enhook_is_the_package_macro.tex"),
    creators: &[],
    elements: &[
      r#"<p xml:id="S0.I1.i1.p1.1">First.</p>"#,
      r#"<p xml:id="p1.1">Hooked.</p>"#,
    ],
  },
  // amsaddr.sty:43 `\emails` (2606.23853, 2606.25954): the document's own `\email` fills it.
  Row {
    name:     "amsaddr",
    tex:      repro!("package-bindings/amsaddr_emails_is_the_package_macro.tex"),
    creators: &[&["Ann Able"]],
    elements: &[r#"<p xml:id="p1.1">Listed: ann@x.org, bob@x.org.</p>"#],
  },
  // threeparttable.sty:302-303 `\TPT@defaults` (2605.22976).
  Row {
    name:     "threeparttable",
    tex:      repro!("package-bindings/threeparttable_defaults_is_the_package_macro.tex"),
    creators: &[],
    elements: &[r#"<p xml:id="S0.I1.ix1.p1.1">Note.</p>"#],
  },
  // icml2026.sty:490-499: `\icmlcorrespondingauthor` builds `\icmlcorrespondingauthor@text` (2605.04712, 2605.21190,
  // 2606.01172) and still gives the e-mail contact.
  Row {
    name:     "icml2026",
    tex:      repro!("package-bindings/icml_corresponding_author_text_is_built.tex"),
    creators: &[&["Ann Able", "affiliation=Univ A", "email=ann@x.org"]],
    elements: &[r#"<p xml:id="p1.1">Listed: Ann Able &lt;ann@x.org&gt;. Code at X.</p>"#],
  },
];

fn check(row: &Row) {
  let (log, xml) = run(row.tex);
  let name = row.name;
  assert_eq!(
    (error_count(&log), warning_count(&log)),
    (0, 0),
    "{name}: {log}"
  );
  let want: Vec<Vec<String>> = row
    .creators
    .iter()
    .map(|c| c.iter().map(|s| (*s).to_string()).collect())
    .collect();
  assert_eq!(creators(&xml), want, "{name}: {xml}");
  for element in row.elements {
    assert_eq!(xml.matches(element).count(), 1, "{name}: {element}\n{xml}");
  }
}

/// Cluster A: each binding defines what its real class or package defines, so a document's `\g@addto@macro` on it is
/// clean (it was `Error:undefined` since OD #1036 made the addition to an undefined macro an Error, as TeX's).
#[test]
fn bindings_define_the_macros_their_files_define() {
  for row in BINDING_ROWS {
    check(row);
  }
}

/// Cluster D (2605.07565): the first aid's `\ltx@label#1{\cref@label{#1}}` under amsmath 2025-05-18, re-applied after
/// the binding reads the raw cleveref.sty — the display's label whole and its math untouched ("a=b", not "a=b3a"
/// labelled `l`), the text's label taking nothing from the text ("XY", not "X4bY").
#[test]
fn cleveref_ltx_label_braces_its_key_under_amsmath() {
  let (log, xml) = run(repro!(
    "cross-references/cleveref_ltx_label_braces_its_key_under_amsmath.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  for element in [
    r#"<document xmlns="http://dlmf.nist.gov/LaTeXML" labels="LABEL:l4b" xml:id="id1">"#,
    r#"<equation labels="LABEL:l3a" xml:id="S0.E1">"#,
    r#"<Math mode="display" tex="a=b" text="a = b" xml:id="S0.E1.m1">"#,
    r#"<p xml:id="p1.1">XY</p>"#,
  ] {
    assert_eq!(xml.matches(element).count(), 1, "{element}\n{xml}");
  }
}

