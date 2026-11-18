//! INLINECTOR (OXIDIZED_DESIGN_DIVERGENCES #1079): a constructor whose output is authored text where TeX starts the
//! paragraph declares `enterHorizontal`, so the space after it is kept; digested in vertical mode, the space was dropped
//! there (Perl Stomach.pm:249-251, which Perl shares). One guard per binding family, each red on M8B29 and Perl 0.8.8.

use latexml::util::test::{assert_element, convert_files_with, convert_with};

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

/// fancybox's boxes set their content in an `\hbox` and start the paragraph (fancybox.sty:60-61, :136, :152): at a
/// paragraph's start the space after the box is kept, and mid-paragraph the paragraph stays one `<p>` (Perl's
/// `internal_vertical` ended it around the box).
#[test]
fn fancybox_box_starts_the_paragraph() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/paragraph-mode/fancybox_box_starts_the_paragraph.tex"
  ));
  assert_element(
    &xml,
    "para",
    &["xml:id=\"p1\""],
    r#"<para xml:id="p1"><p xml:id="p1.1"><text cssstyle="border:1px solid black; box-shadow: 5px 5px 10px black;" xml:id="p1.1.1">A</text> starts here.</p></para>"#,
  );
  assert_element(
    &xml,
    "para",
    &["xml:id=\"p2\""],
    r#"<para xml:id="p2"><p xml:id="p2.1">Mid <text cssstyle="border:3px double black;" xml:id="p2.1.1">C</text> para.</p></para>"#,
  );
  assert_element(
    &xml,
    "para",
    &["xml:id=\"p3\""],
    r#"<para xml:id="p3"><p xml:id="p3.1">Mid <text cssstyle="border:1px solid black;border-radius:5px;" xml:id="p3.1.1">E</text> and <text cssstyle="border:2px solid black;border-radius:5px;" xml:id="p3.1.2">F</text> para.</p></para>"#,
  );
}

/// A text `\hphantom` starts the paragraph (latex.ltx:15617 `\leavevmode@ifvmode\box\tw@`), through its text branch
/// `\lx@text@hphantom`; a paragraph it alone holds stays (its box is `restricted_horizontal`, not a space).
#[test]
fn hphantom_starts_the_paragraph() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/paragraph-mode/hphantom_starts_the_paragraph.tex"
  ));
  assert_element(
    &xml,
    "para",
    &["xml:id=\"p1\""],
    r#"<para xml:id="p1"><p xml:id="p1.1"><text class="ltx_phantom" xml:id="p1.1.1">X</text> starts here.</p></para>"#,
  );
  assert_element(
    &xml,
    "para",
    &["xml:id=\"p2\""],
    r#"<para xml:id="p2"><p xml:id="p2.1"><text class="ltx_phantom" xml:id="p2.1.1">Y</text></p></para>"#,
  );
}

/// revtex4_support `\text` outside math (`\REV@text`, an `\mbox`; census 2635 revtex papers).
#[test]
fn revtex_text_starts_the_paragraph() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/paragraph-mode/revtex_text_starts_the_paragraph.tex"
  ));
  assert_element(
    &xml,
    "para",
    &["xml:id=\"p1\""],
    r#"<para xml:id="p1"><p xml:id="p1.1"><text xml:id="p1.1.1">A</text> starts here.</p></para>"#,
  );
}

/// The remaining families, one paragraph each: ams_support `\URL`, html `\hyperrefdef`, latexml `\lxWithClass`, and
/// the kernel's `\lx@text@nounicode` (wasysym `\octagon`).
#[test]
fn bindings_start_the_paragraph() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/paragraph-mode/bindings_starts_the_paragraph.tex"
  ));
  assert_element(
    &xml,
    "para",
    &["xml:id=\"S1.p2\""],
    r#"<para xml:id="S1.p2"><p xml:id="S1.p2.1"><ref href="http://x.org">http://x.org</ref> is a site.</p></para>"#,
  );
  assert_element(
    &xml,
    "para",
    &["xml:id=\"S1.p3\""],
    r#"<para xml:id="S1.p3"><p xml:id="S1.p3.1"><ref labelref="LABEL:s">link</ref> is a link.</p></para>"#,
  );
  assert_element(
    &xml,
    "para",
    &["xml:id=\"S1.p4\""],
    r#"<para xml:id="S1.p4"><p xml:id="S1.p4.1"><text class="cl" xml:id="S1.p4.1.1">A</text> has a class.</p></para>"#,
  );
  assert_element(
    &xml,
    "para",
    &["xml:id=\"S1.p5\""],
    r#"<para xml:id="S1.p5"><p xml:id="S1.p5.1"><text class="ltx_nounicode" xml:id="S1.p5.1.1">\octagon</text> is a glyph.</p></para>"#,
  );
}
