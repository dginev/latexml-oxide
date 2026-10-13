//! Red/green guards for batch 65v: the jfm and mdpi bindings define what the versions of those author-shipped classes
//! define and papers call. Repros `tools/perfect_kernel/repros/class-bindings/{jfm,mdpi}_class_commands.tex`.
use super::perfect_kernel_batch46::error_count;

/// jfm.cls :58 `\ifCUPmtlplainloaded` (the template preamble's font checks), :1176-1183 `\checkfont`, the 2020
/// version's caption (:1837) and the author conveniences of the versions (:1107-1108, :1263-1321, :173-187).
/// Witnesses 2103.10260, 2508.02297, 2604.04559, 1309.5858, 2103.09196, 2406.15733.
#[test]
fn jfm_class_commands_are_defined() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/class-bindings/jfm_class_commands.tex");
  let (log, xml) = latexml::util::test::convert_with(tex, Some("ar5iv.sty"));
  assert_eq!(error_count(&log), 0, "{log}");
  for text in [
    "NOCUP", "FOUND", ">Re<", ">Pr<", ">Ra<", "\u{2202}", "\u{03C0}",
  ] {
    assert!(xml.contains(text), "{text}\n{xml}");
  }
  assert!(!xml.contains(">CUP<"), "{xml}");
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
  assert!(
    xml.contains("The authors declare no conflict of interest."),
    "{xml}"
  );
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
