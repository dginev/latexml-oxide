//! Red/green guards for batch 65v: the jfm and mdpi bindings define what the versions of those author-shipped classes
//! define and papers call. Repros `tools/perfect_kernel/repros/class-bindings/{jfm,mdpi}_class_commands.tex`.
use latexml::util::test::convert_files_with;

use super::perfect_kernel_batch46::{error_count, warning_count};

/// jfm.cls :58 `\ifCUPmtlplainloaded` (the template preamble's font checks), :1176-1183 `\checkfont`, the 2020
/// version's caption (:1837) and the author conveniences of the versions (:1107-1108, :1263-1321, :173-187).
/// Witnesses 2103.10260, 2508.02297, 2604.04559, 1309.5858, 2103.09196, 2406.15733.
#[test]
fn jfm_class_commands_are_defined() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/class-bindings/jfm_class_commands.tex");
  let (log, xml) = latexml::util::test::convert_with(tex, Some("ar5iv.sty"));
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  for text in [
    "NOCUP", "FOUND", ">Re<", ">Pr<", ">Ra<", "\u{2202}", "\u{03C0}",
  ] {
    assert!(xml.contains(text), "{text}\n{xml}");
  }
  assert!(!xml.contains(">CUP<"), "{xml}");
  // jfm.cls:1361 (73 of 119 sampled copies, and with no copy shipped): `\le` is the slanted ⩽ once amssymb loads
  assert!(xml.contains("\u{2A7D}"), "{xml}");
}

/// A convenience one jfm.cls version defines and another lacks yields to the paper's own definition, as a paper
/// whose copy lacks it defines it (`DefinitionOrigin::Fallback`): `\p` is the paper's π, not the class's ∂.
#[test]
fn jfm_conveniences_yield_to_the_paper() {
  let (log, xml) = latexml::util::test::convert_with(
    r"\documentclass{jfm}
\newcommand\p{\pi}
\begin{document}
$\p$
\end{document}
",
    Some("ar5iv.sty"),
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert!(xml.contains("\u{03C0}"), "{xml}");
  assert!(!xml.contains("\u{2202}"), "{xml}");
}

/// mdpi.cls (2012-2025) title-page setters — `\doinum` printed once accepted (:424, :446), `\secondnote` (:560),
/// `\conference` (:521, :964-965), `\simplesumm` (:574, :1030) — and the packages its versions load (caption,
/// lastpage, float, enumitem, mathpazo, epstopdf, upgreek, subfig). Witnesses 2008.03867, 2103.15665, 2008.00705,
/// 2508.02963, 1904.02732, 2508.18441, 2204.11608.
#[test]
fn mdpi_class_commands_are_defined() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/class-bindings/mdpi_class_commands.tex");
  let (log, xml) = latexml::util::test::convert_with(tex, Some("ar5iv.sty"));
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  for (role, text) in [
    ("doi", "10.3390/x1010001"),
    ("secondnote", "These authors contributed equally."),
    ("simplesummary", "A plain-language summary."),
    (
      "conference",
      "This paper is an extended version of our paper published in Proceedings of a Conference 2020.",
    ),
  ] {
    assert_eq!(note_text(&xml, role), Some(text), "{role}\n{xml}");
  }
  assert!(xml.contains("\u{03BC}"), "{xml}");
  // the later versions' `\conflictofinterests` (43 of the 62 sampled copies that define it, and with no copy
  // shipped): the statement under a bold label
  assert!(
    xml.contains("Conflicts of Interest:</text> The authors declare no conflict of interest."),
    "{xml}"
  );
  let label = xml
    .find(">Conflicts of Interest:</text>")
    .expect("the label");
  let element = &xml[xml[..label].rfind("<text").expect("its element")..label];
  assert!(element.contains(r#"font="bold""#), "{element}\n{xml}");
}

/// mdpi.cls:592-593, :1035-1039: `\addhighlights{yes}` switches the "Highlights" block on, and the block is the
/// paper's own redefinition of `\addhighlights` (the class comment: "redefined in .tex to use itemize"); the switch
/// alone, or the setter left as it is, prints nothing.
#[test]
fn mdpi_highlights_are_the_papers_redefinition() {
  let (log, xml) = latexml::util::test::convert_with(
    r"\documentclass[journal,article,accept]{mdpi}
\Title{Title}
\addhighlights{yes}
\renewcommand{\addhighlights}{\textbf{What are the main findings?}\begin{itemize}\item First bullet.\end{itemize}}
\begin{document}
Text.
\end{document}
",
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let note = note_text(&xml, "highlights").expect("a highlights note");
  assert!(note.contains("What are the main findings?"), "{xml}");
  assert!(note.contains("First bullet."), "{xml}");
  assert!(!xml.contains(">yes<"), "{xml}");
  let (log, xml) = latexml::util::test::convert_with(
    r"\documentclass[journal,article,accept]{mdpi}
\Title{Title}
\addhighlights{yes}
\begin{document}
Text.
\end{document}
",
    Some("ar5iv.sty"),
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert!(note_text(&xml, "highlights").is_none(), "{xml}");
}

/// What one mdpi.cls version defines and another lacks yields to the paper's own definition
/// (`DefinitionOrigin::Fallback`): a paper whose copy lacks `\orcidicon` or `\highlighting` defines its own.
#[test]
fn mdpi_conveniences_yield_to_the_paper() {
  let (log, xml) = latexml::util::test::convert_with(
    r"\documentclass[journal,article,accept]{mdpi}
\newcommand{\orcidicon}{ORC}
\newcommand{\highlighting}[1]{HL(#1)}
\begin{document}
Body \orcidicon{} and \highlighting{x}.
\end{document}
",
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(xml.contains("Body ORC and HL(x)."), "{xml}");
}

/// A submitted manuscript (mdpi's default status) prints no DOI: the journal line is "Version … submitted to …"
/// (mdpi.cls:664-670).
#[test]
fn mdpi_submitted_manuscript_has_no_doi() {
  let (log, xml) = latexml::util::test::convert_with(
    r"\documentclass[journal,article,submit]{mdpi}
\Title{Title}
\doinum{10.3390/x1010001}
\begin{document}
Text.
\end{document}
",
    Some("ar5iv.sty"),
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert!(!xml.contains("10.3390/x1010001"), "{xml}");
}

/// The text of the frontmatter `<note>` with this role (its attributes, an `xml:id` among them, skipped).
fn note_text<'a>(xml: &'a str, role: &str) -> Option<&'a str> {
  let start = xml.find(&format!("<note role=\"{role}\""))?;
  let rest = &xml[start..];
  let open = rest.find('>')? + 1;
  Some(&rest[open..open + rest[open..].find("</note>")?])
}

/// A 2012-2015 mdpi.cls copy (1307.3961's, abridged; found by the `Definitions/` path papers use) loads no enumitem and
/// sets `\conflictofinterests`'s argument as a bold title: the paper's own `\usepackage[shortlabels]{enumitem}` is
/// the first load, so its `[a)]` labels stand, and the title stays a bold paragraph.
#[test]
fn mdpi_follows_an_old_shipped_copy() {
  let (log, xml) = convert_files_with(
    r"\documentclass{Definitions/mdpi}
\usepackage[shortlabels]{enumitem}
\begin{document}
\begin{enumerate}[a)]\item One\item Two\end{enumerate}
\conflictofinterests{Conflicts of Interest}
None declared.
\end{document}
",
    &[(
      "Definitions/mdpi.cls",
      r"\ProvidesClass{mdpi}[2013/04/05 MDPI paper class]
\RequirePackage{graphicx}
\newcommand{\conflictofinterests}[1]{%
    \par\vspace{12pt}\noindent\textbf{#1}%
    \par\vspace{12pt}}
",
    )],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  for label in ["a)", "b)"] {
    assert!(
      xml.contains(&format!("<tag>{label}</tag>")),
      "{label}\n{xml}"
    );
  }
  assert!(xml.contains(r#"font="bold""#), "{xml}");
  assert!(xml.contains(">Conflicts of Interest</text>"), "{xml}");
  assert!(!xml.contains("Conflicts of Interest:"), "{xml}");
}

/// jfm.cls copies differ in the msam10 check: most set `\le` to the slanted ⩽ (2103.10260's :1361), one comments it
/// out (2608.01828's :2179). The paper's own copy decides.
#[test]
fn jfm_follows_the_shipped_copy_for_the_inequalities() {
  let class = |active: bool| {
    format!(
      "\\ProvidesClass{{jfm}}[2007/03/13 v1.28 Journal of Fluid Mechanics document class]\n\\IfFileExists{{amssymb.sty}}{{\\usepackage{{amssymb}}%\n{}\\let\\le=\\leqslant  \\let\\leq=\\leqslant\n}}{{}}\n",
      if active { "" } else { "%" }
    )
  };
  for (active, name) in [(true, "leqslant"), (false, "leq")] {
    let text = class(active);
    let (log, xml) = convert_files_with(
      "\\documentclass{jfm}\n\\begin{document}\n$a\\le b$\n\\end{document}\n",
      &[("jfm.cls", text.as_str())],
      Some("ar5iv.sty"),
    );
    assert_eq!(error_count(&log), 0, "{log}");
    assert!(xml.contains(&format!(r#"name="{name}""#)), "{name}\n{xml}");
  }
}
