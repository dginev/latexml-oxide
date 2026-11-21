//! Guards for FUPB, the #944 follow-ups: a name a binding defines from user data is a token, never re-read as a
//! prototype, and the TeX-level `\DeclareOption{}` keeps latex.ltx's `\ds@` where a binding's empty name is Perl's
//! default handler. The `DeclareOption` binding forms are guarded beside the Rhai ones (`latexml_contrib`
//! `script_bindings::tests::declare_option_*`). Every input is checked against pdflatex (TL 2025).
use latexml::util::test::{assert_element, convert_with, error_count, warning_count};

/// fonts-nfss/declared_math_alphabet_name_is_one_token: `\DeclareMathAlphabet` on `\csname mathx-y\endcsname` defines
/// that one control sequence, as Perl's `DefPrimitiveI(T_CS($csname), …)` (latex_constructs.pool.ltxml:2684-2694;
/// pdflatex 0 errors). Before: the name re-read as the prototype `\mathx-y{}`, Fatal:Parameter:Unknown on `y`.
#[test]
fn declared_math_alphabet_name_is_one_token() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/fonts-nfss/declared_math_alphabet_name_is_one_token.tex"
    ),
    None,
  );
  assert!(!stderr.contains("Fatal:"), "{stderr}");
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "XMath",
    &[],
    r#"<XMath><XMApp><XMTok meaning="plus" role="ADDOP">+</XMTok><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="sansserif" role="UNKNOWN">W</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">c</XMTok></XMApp><XMTok font="italic" role="UNKNOWN">x</XMTok></XMApp></XMath>"#,
  );
}

/// latex.ltx:18535-18538 `\@declareoption`: `\DeclareOption{}{…}` defines `\ds@` (pdflatex 0 errors, "ds-at-ran"),
/// while an empty name to a binding's `DeclareOption` is Perl's default handler `\default@ds` (Package.pm:2423), the
/// path the TeX-level command shares.
#[test]
fn tex_declareoption_empty_name_defines_ds_at() {
  let (stderr, xml) = convert_with(
    "\\documentclass{article}\n\\makeatletter\n\\DeclareOption{}{\\gdef\\emptyran{ds-at-ran}}\n\\def\\emptyran{unset}\n\
     \\@nameuse{ds@}\n\\makeatother\n\\begin{document}\nEmpty: \\emptyran.\n\\end{document}\n",
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(&xml, "p", &[], "<p>Empty: ds-at-ran.</p>");
}
