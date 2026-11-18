//! INLINECTOR (OXIDIZED_DESIGN_DIVERGENCES #1079): a constructor whose output is authored text where TeX starts the
//! paragraph declares `enterHorizontal`, so the space after it is kept; digested in vertical mode, the space was dropped
//! there (Perl Stomach.pm:249-251, which Perl shares). One guard per binding family; each was red on M8B29 (and Perl
//! 0.8.8): the word after the command glued to it.

use latexml::util::test::{assert_element, convert_files_with, convert_with, xml_element};

use crate::perfect_kernel_batch46::{error_count, warning_count};

fn convert_clean(tex: &str) -> String {
  let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  xml
}

/// aa_support `\object` (aa.cls `\def\object##1{##1}`; census 447 aa papers).
#[test]
fn aa_object_starts_the_paragraph() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/paragraph-mode/aa_object_starts_the_paragraph.tex"
  ));
  assert_element(
    &xml,
    "para",
    &["xml:id=\"p1\""],
    r#"<para xml:id="p1"><p xml:id="p1.1"><text class="ltx_ast_objectname" xml:id="p1.1.1">M31</text> is a galaxy.</p></para>"#,
  );
}

/// llncs `\url` (llncs.cls:899 `\def\url#1{#1}`; census 640 llncs papers).
#[test]
fn llncs_url_starts_the_paragraph() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/paragraph-mode/llncs_url_starts_the_paragraph.tex"
  ));
  assert_element(
    &xml,
    "para",
    &["xml:id=\"p1\""],
    r#"<para xml:id="p1"><p xml:id="p1.1"><ref href="http://x.org">http://x.org</ref> is a site.</p></para>"#,
  );
}

/// amsppt `\cite` (amsppt.sty:847 `\rom{[…]}`), a plain AMS-TeX document.
#[test]
fn amsppt_cite_starts_the_paragraph() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/paragraph-mode/amsppt_cite_starts_the_paragraph.tex"
  ));
  assert_element(
    &xml,
    "para",
    &["xml:id=\"p1\""],
    r#"<para xml:id="p1"><p xml:id="p1.1"><cite>[<bibref bibrefs="1" show="refnum"/>]</cite> is the reference.</p></para>"#,
  );
}

/// The kernel's `\lx@orcidlink` (orcidlink.sty `\orcidlink`: `\href{…}{\mbox{…}}`): the engine-level member. The logo's
/// SVG between is not this guard's subject; the link's opening and what follows it are.
#[test]
fn orcidlink_starts_the_paragraph() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/paragraph-mode/orcidlink_starts_the_paragraph.tex"
  ));
  let para =
    xml_element(&xml, "para", &["xml:id=\"p1\""]).unwrap_or_else(|| panic!("no para p1\n{xml}"));
  assert!(
    para.starts_with(
      r#"<para xml:id="p1"><p xml:id="p1.1"><ref class="ltx_orcid" href="https://orcid.org/0000-0002-1825-0097" title="ORCID 0000-0002-1825-0097"><svg:svg"#
    ),
    "{para}"
  );
  assert!(
    para.ends_with(r#"</svg:svg></ref> is my iD.</p></para>"#),
    "{para}"
  );
  assert_eq!(para.matches("<p ").count(), 1, "{para}");
}

/// A Rhai binding declares the option as Perl's does (`#{enterHorizontal: true}`): `\lxa`, a template constructor
/// opening the paragraph, keeps the space after it, and so the imperative `\lxb` (`openElement`) has its space before
/// it — the RED `imperative_constructor_drops_space_red` blamed `\lxb`; the space was `\lxa`'s.
#[test]
fn rhai_constructor_entering_horizontal_keeps_the_space() {
  let (log, xml) = convert_files_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/rhai-bindings/constructor_opening_the_paragraph_keeps_the_space.tex"
    ),
    &[(
      "lximp.sty.rhai",
      include_str!("../../../tools/perfect_kernel/repros/rhai-bindings/lximp.sty.rhai"),
    )],
    None,
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  assert_element(
    &xml,
    "para",
    &["xml:id=\"p1\""],
    r#"<para xml:id="p1"><p><text class="a">a</text> <text class="b">b</text></p></para>"#,
  );
}
