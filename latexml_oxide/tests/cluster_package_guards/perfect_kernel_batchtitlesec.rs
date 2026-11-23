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
