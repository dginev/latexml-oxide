//! Red/green guards for batch 81b (hyperref's autoref tag runs the kernel's `\null`, whatever the author made of it).
//! Each guard converts its repro under `tools/perfect_kernel/repros/`, asserts the diagnostics by count and the
//! elements.
use latexml::util::test::convert_files_with;

use super::perfect_kernel_batch46::{error_count, warning_count};

const PRELOAD: &str = "ar5iv.sty";

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!("../../../tools/perfect_kernel/repros/", $path))
  };
}

/// The autoref tag every numbered target builds (`\lx@autorefnum@@`, DIVERGENCES #418) ends with the token `\null`, the
/// delimiter of hyperref's `\def\equationautorefname~#1\null{…}` idiom, and runs it as the kernel's empty box: an
/// author's `\def\null0{…}` hiding a subsection is never run there, as pdflatex runs `\null` only where an `\autoref`
/// prints. Red (int110h): Fatal Stomach:Recursion, `\section` → its tag → the author's `\subsection`. Witness
/// 1501.00090. Repro cross-references/hyperref_autoref_tag_runs_the_kernel_null.
#[test]
fn an_authors_null_stays_out_of_the_autoref_tag() {
  let (log, xml) = convert_files_with(
    repro!("cross-references/hyperref_autoref_tag_runs_the_kernel_null.tex"),
    &[],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    xml.contains("<tag role=\"autoref\">section\u{a0}2<"),
    "{xml}"
  );
  assert!(!xml.contains("Hidden"), "{xml}");
}

/// xargs.sty:205-212: a `\newcommandx` command with an optional argument is `\@protected@testopt(@xargs)\cs…`, which
/// stays unexpanded (`\@x@protect`) whenever `\protect` is not `\@typeset@protect` — inside ifthen's `\xdef` of a test
/// (ifthen.sty:92-117) an `\equal` argument holding such a command keeps it as a token. Red (int110m): `\sequence`
/// expanded there and ran its own `\ifthenelse`, Fatal "File ended while scanning use of a Until:\NOT argument"
/// (2002.09653). Repro package-bindings/xargs_optional_command_is_protected_in_ifthen_test.
#[test]
fn an_xargs_optional_command_is_protected_in_an_ifthen_test() {
  let (log, xml) = convert_files_with(
    repro!("package-bindings/xargs_optional_command_is_protected_in_ifthen_test.tex"),
    &[],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(xml.contains(r#"tex="\{a\mid(1)\}""#), "{xml}");
}

/// postpictex.tex:39-46: PiCTeX's `\beginpicture` takes `\put`, `\multiput`, `\frame` and `\linethickness` over and its
/// `\@picture` gives a LaTeX picture LaTeX's back; our `{picture}` does not pass through `\@picture`, so it makes the same
/// `\let`s when PiCTeX saved them. Red (int110m): a LaTeX picture inside a PiCTeX `\put{…} at x y` read its own
/// `\put(3,0)` as PiCTeX's, which scanned to the end of the input for its `at` — Fatal (1301.4518). Repro
/// package-bindings/pictex_latex_picture_inside_put_keeps_latex_put.
#[test]
fn a_latex_picture_inside_pictex_keeps_latex_put() {
  let (log, xml) = convert_files_with(
    repro!("package-bindings/pictex_latex_picture_inside_put_keeps_latex_put.tex"),
    &[],
    Some(PRELOAD),
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert!(xml.contains("<picture"), "{xml}");
  assert!(xml.contains(">Text.</p>"), "{xml}");
}

/// xy_sty.rs's `\xy` wrapper (`\if\inxy@\lx@xy@svgnested\else\lx@xy@svg\fi\lx@xy@original`, LaTeXML's own, not xy.tex's)
/// opens with `\relax`, as its `\endxy` does: at an `\xymatrix` cell head the alignment's peek stops there and the
/// wrapper's conditional runs inside the cell. Red (int110o): expanded by the peek, its conditional was lost before its
/// `\else` — 10 errors, and a conditional-limit Fatal in 1109.4916 and 1611.07444. Repro
/// package-bindings/xymatrix_cell_opening_with_xy.
#[test]
fn an_xymatrix_cell_may_open_with_xy() {
  let (log, xml) = convert_files_with(
    repro!("package-bindings/xymatrix_cell_opening_with_xy.tex"),
    &[],
    Some(PRELOAD),
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert!(xml.contains("<picture"), "{xml}");
}

/// tex.web §1193 + §1064-1065 `off_save`: a math shift arriving while a group opened inside the formula is open inserts
/// that group's closer and is read again — in a display opened by `$$` as in an inline formula (OXIDIZED_DESIGN #553).
/// Red (int110o): the display's `$$` was deferred to the group's end, which a typo'd `H^k{X,a)` never reaches — 5
/// errors here, 99 and the error-limit Fatal in 1309.7562 (pdflatex 1). Repro
/// package-bindings/display_math_shift_closes_an_open_group.
#[test]
fn a_display_math_shift_closes_a_group_opened_in_it() {
  let (log, xml) = convert_files_with(
    repro!("package-bindings/display_math_shift_closes_an_open_group.tex"),
    &[],
    Some(PRELOAD),
  );
  assert_eq!(error_count(&log), 1, "{log}");
  assert!(xml.contains("<section"), "{xml}");
  assert_eq!(xml.matches("<equation").count(), 2, "{xml}");
  assert!(xml.contains(r#"tex="z""#), "{xml}");
}

/// latex.ltx:1156 `\new@command#1{\@testopt{\@newcommand#1}0}`: `\newcommand{\two[5]}{…}` hands `\two[5]` to
/// `\@testopt`, which sees the body's `{` and inserts `[{0}]` — five arguments, the first optional with default 0 (the
/// `LaTeXDefToken` parameter). Red (int110o): five plain arguments, the next token swallowed (1312.5793: 90 script
/// errors and the error-limit Fatal). Repro macro-state/newcommand_nargs_inside_the_braces.
#[test]
fn newcommand_nargs_inside_the_braces_take_the_testopt_default() {
  let (log, xml) = convert_files_with(
    repro!("macro-state/newcommand_nargs_inside_the_braces.tex"),
    &[],
    Some(PRELOAD),
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert!(xml.contains("[a b c d] x"), "{xml}");
}

/// pdfTeX's `\pdftexbanner` names its TeX Live year ("This is pdfTeX, Version 3.141592653-2.6-1.40.28 (TeX Live 2025)
/// kpathsea version 6.4.1"), which documents parse to branch on the year; ours says the runtime tree's. Red (int110o):
/// empty, as Perl's — "Paragraph ended before \gettexliveversion was complete" and the error-limit Fatal in 2309.17381.
/// Repro expansion-primitives/pdftexbanner_names_the_texlive_year.
#[test]
fn pdftexbanner_names_the_texlive_year() {
  let (log, xml) = convert_files_with(
    repro!("expansion-primitives/pdftexbanner_names_the_texlive_year.tex"),
    &[],
    Some(PRELOAD),
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert!(xml.contains("NEW"), "{xml}");
}

/// A package requested by an installed path (`\usepackage{graphics/graphicx}`) that resolves to the same file as its
/// basename is the package the binding of that name serves (`source_tree_basename`, OXIDIZED_DESIGN #177's rerouting
/// widened). Red (int110o): the raw graphicx.sty ran over our graphics binding and the first `\rotatebox` exhausted
/// memory (2309.12794). Repro loader/package_by_installed_path_uses_its_binding.
#[test]
fn a_package_by_its_installed_path_uses_its_binding() {
  let (log, xml) = convert_files_with(
    repro!("loader/package_by_installed_path_uses_its_binding.tex"),
    &[],
    Some(PRELOAD),
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert!(xml.contains(r#"angle="90""#), "{xml}");
}

/// `LaTeXDefToken` strips brace layers one by one, as DefToken does: `\newcommand{{\datanum}}{…}` defines `\datanum`
/// (LaTeX: `\@newcommand#1` reads `{\datanum}`'s content). Red (82h r1): the token was the inner `{` and the document
/// ran to the error limit (2407.13761, nets eccv/llncs/wasysym).
#[test]
fn newcommand_of_a_doubly_braced_name_defines_the_name() {
  let (log, xml) = convert_files_with(
    r"\documentclass{article}
\newcommand{{\datanum}}{2,565}
\newcommand{\two[5]}{[#2 #3 #4 #5]}
\begin{document}
\datanum{} \two{a}{b}{c}{d} x
\end{document}",
    &[],
    Some(PRELOAD),
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert!(xml.contains("2,565 [a b c d] x"), "{xml}");
}

/// `\renewcommand`/`\providecommand` take no `\@testopt` default from a count inside the braces: latex.ltx's
/// `\renew@command#1` and `\provide@command#1` hand `\new@command` their argument unbraced, which takes the command alone,
/// and `\@testopt` then meets the `[2]` — two plain arguments (pdflatex "A [RJ: Dynamical] B"). Red (82h): the
/// declarators shared `\newcommand`'s `LaTeXDefToken` and printed "[0: RJ]Dynamical" (1504.03761).
#[test]
fn providecommand_nargs_inside_the_braces_are_plain() {
  let (log, xml) = convert_files_with(
    r"\documentclass{article}
\providecommand{\com[2]}{[#1: #2]}
\providecommand{\comrj[1]}{\com{RJ}{#1}}
\begin{document}
A \comrj{Dynamical} B
\end{document}",
    &[],
    Some(PRELOAD),
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert!(xml.contains("A [RJ: Dynamical] B"), "{xml}");
}

/// AASTeX 6.1's `\collaboration{name}` takes REVTeX's one argument and its `\nocollaboration` none (aastex61.cls,
/// revtex4-1's `\@author@def`); AASTeX 6.3+'s `\collaboration{n}{name}` and `\nocollaboration{n}` one more
/// (aastex701.cls:13355, :13377) — read by the call's shape. Red: the two-argument binding (Perl's
/// aas_support.sty.ltxml:126, KNOWN_PERL_ERRORS) took the next `\author` as the name, the author "P. Mróz" a stray
/// paragraph (1802.09023). Repro package-bindings/aastex61_collaboration_takes_one_argument.
#[test]
fn an_aastex61_collaboration_takes_one_argument() {
  let (log, xml) = convert_files_with(
    repro!("package-bindings/aastex61_collaboration_takes_one_argument.tex"),
    &[],
    Some(PRELOAD),
  );
  assert_eq!(error_count(&log), 0, "{log}");
  for name in ["Ann Able", "Bob Baker", "Cy Cole"] {
    assert!(
      xml.contains(&format!("<personname>{name}</personname>")),
      "{name}: {xml}"
    );
  }
  assert!(
    xml.contains("(Spitzer Team)") && xml.contains("(OGLE Collaboration)"),
    "{xml}"
  );
  assert!(
    !xml.contains("<p>Bob Baker") && !xml.contains("<p>Cy Cole"),
    "{xml}"
  );
}

/// FIDELITY_CHARTER §4, "ownership by evidence, never by bare position": an `\email` call's address that spells one
/// author goes to that author, not to the one its `annotate=last` names — an AASTeX 5 `\email{…}` after a one-call author
/// list (0911.3506, 1506.04035, 1203.1317, 0904.0560: 92 of the 688 aas census papers). Red: on the last author, as
/// Perl. Repro sectioning-frontmatter/email_call_goes_to_the_author_it_spells.
#[test]
fn an_email_call_goes_to_the_author_it_spells() {
  let (log, xml) = convert_files_with(
    repro!("sectioning-frontmatter/email_call_goes_to_the_author_it_spells.tex"),
    &[],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let creators = super::perfect_kernel_batch61::creators_of(&xml);
  assert_eq!(
    creators,
    [
      "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact><contact name=\"Email:\u{a0}\" role=\"email\"><text font=\"typewriter\" xml:id=\"id1\">able@x.org</text></contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact></creator>",
    ],
    "{xml}"
  );
}
