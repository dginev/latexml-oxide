//! TITLESEC (DG ruling 2026-11-23, the titlesec half of READARG): a titlesec format applies only while titlesec holds
//! the command; a later `\@startsection` definition (or sectsty's load) gives it back to LaTeX. OD #1143,
//! KNOWN_PERL_ERRORS #1317. Witnesses 1910.11159, 2005.02481, 2005.06700 (each a Fatal "Expansion of
//! '\format@title@@}' has unbalanced {}"). tex.web §395's extra `}` (`{\titleformat}`) is row R READARG395.
use super::perfect_kernel_batch46::{convert, error_count, warning_count};

/// The document's `<title>` elements, whole and in order.
fn titles(xml: &str) -> Vec<String> {
  xml
    .lines()
    .map(str::trim)
    .filter(|line| line.starts_with("<title"))
    .map(str::to_string)
    .collect()
}

/// titlesec holds `\section`…`\subparagraph` from its load; a later `\renewcommand\section{\@startsection…}` (the
/// kernel's lock keeps it as `\lx@dropped@section`) or sectsty's load gives the command back to LaTeX, whose heading
/// titlesec's formats never reach: 1910.11159/2005.02481's format label `\titleformat` and 2005.06700's are never set
/// (pdflatex 0 errors). A redefinition before titlesec's load is titlesec's again (`\paragraph`), and a command not
/// redefined keeps its format (`\subsection`; titlesec's `1em` separator is U+2003).
#[test]
fn titlesec_formats_end_where_latex_takes_the_command() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/titlesec_format_ends_at_a_startsection_redefinition.tex"
  );
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_eq!(
    titles(&xml),
    [
      r#"<title><tag close=" ">1</tag>Intro</title>"#,
      "<title class=\"ltx_runin\" font=\"italic\">1.1.\u{2003}Kept</title>",
      "<title class=\"ltx_runin\" font=\"italic\">[1.1.0.1]\u{2003}Para</title>",
    ],
    "{xml}"
  );

  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/titlesec_format_ends_at_sectsty.tex"
  );
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_eq!(
    titles(&xml),
    [
      r#"<title><tag close=" ">1</tag>Intro</title>"#,
      r#"<title><tag close=" ">1.1</tag>Next</title>"#,
    ],
    "{xml}"
  );
}

/// `\preto` extends the command titlesec holds, which the lock keeps as a new record (state.rs
/// `extended_refused_definition`): of the class's definition titlesec claimed at its load (`\section`) or of the
/// binding's own (`\subsection`). It is no new `\@startsection` definition, and titlesec keeps both (pdflatex 0 errors;
/// U+2003, titlesec's `1em`).
#[test]
fn titlesec_keeps_an_extended_command() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/titlesec_format_holds_through_an_extension.tex"
  );
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_eq!(
    titles(&xml),
    [
      "<title font=\"italic\">[1]\u{2003}Intro</title>",
      "<title font=\"italic\">[1.1]\u{2003}Next</title>",
    ],
    "{xml}"
  );
}

/// sectsty redefines the commands only under article/report/book and the KOMA classes (sectsty.sty:298-362); under
/// memoir it raises its PackageError (pdflatex's second error, memoir's `\parttitlefont` already defined, is sectsty's
/// `\newcommand`, which the binding does not raise) and titlesec keeps `\section` (U+2003, titlesec's `1em`).
#[test]
fn titlesec_keeps_the_command_where_sectsty_does_not_work() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/titlesec_keeps_the_command_where_sectsty_does_not_work.tex"
  );
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 1, "{stderr}");
  assert!(
    stderr.contains("Package sectsty Error: The sectsty package doesn't work with"),
    "{stderr}"
  );
  // (memoir's own: its warnings on titlesec and sectsty, and `\@iffirstamp` defined as a conditional)
  assert_eq!(warning_count(&stderr), 3, "{stderr}");
  assert_eq!(
    titles(&xml),
    ["<title font=\"italic\">[0.1]\u{2003}Intro</title>"],
    "{xml}"
  );
}
