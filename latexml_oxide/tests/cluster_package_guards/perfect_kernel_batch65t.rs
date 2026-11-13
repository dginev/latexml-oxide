//! Red/green guards for batch 65t: the AASTeX binding defines the commands of the AASTeX 6.x class files (aastex6,
//! aastex62, aastex63, aastex631) that papers shipping those classes call, and aas_macros the journal names
//! aastex631 adds. Repro `tools/perfect_kernel/repros/class-bindings/aastex6x_class_commands.tex`.
use super::perfect_kernel_batch46::{error_count, warning_count};

/// aastex631.cls:651 `\reportnum`, :1494 `\journalinfo`, aastex6.cls:972 `\submitted`, the edit and layout switches
/// (:5789-5798, :7323, :6816, :5856), the author-limit count (:6427-6430), `\tablefontsize` (:1930),
/// `\abovedeluxetableskip` (:2096), `\lambdabar` (aastex701.cls:6737) and the journal names `\maps`, `\jaavso`, `\aas`,
/// `\dps` (witnesses 2604.07983, 2508.14467, 2604.22347, 1609.08711, 2103.02219, 2103.07688; KNOWN_PERL_ERRORS #633).
#[test]
fn aastex6x_class_commands_are_defined() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/class-bindings/aastex6x_class_commands.tex");
  let (log, xml) = latexml::util::test::convert_with(tex, Some("ar5iv.sty"));
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  for note in [
    r#"role="preprint">LIGO-P2000001</pubnote>"#,
    r#"role="journal">Accepted to ApJ</pubnote>"#,
    r#"role="journal">ApJ</pubnote>"#,
  ] {
    assert!(xml.contains(note), "{note}\n{xml}");
  }
  for journal in [
    "M&amp;PS",
    "JAAVSO",
    "AAS Meeting Abstracts",
    "AAS/DPS Meeting Abstracts",
  ] {
    assert!(xml.contains(journal), "{journal}\n{xml}");
  }
  assert!(xml.contains("\u{03BB}\u{0304}"), "{xml}");
}

/// The World Scientific journal classes (ws_journal_cls.rs): ws-mpla.cls:277-282 `\pub{received}{revised}` as
/// pubnotes, ws-ijmpa.cls:152 `\cdash`, ws-mpla.cls:451-482 `{alphlist}`, `{arabiclist}`, `{romanlist}`, `{itemlist}`,
/// and the figure packages the classes input (ws-ijmpa.cls:551-555: psfig, epsfig, epsf). Witnesses 2508.15289,
/// 0709.0836, 0709.1411. Repro class-bindings/ws_journal_class_commands.
#[test]
fn ws_journal_class_commands_are_defined() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/ws_journal_class_commands.tex"
  );
  let (log, xml) = latexml::util::test::convert_with(tex, Some("ar5iv.sty"));
  assert_eq!(error_count(&log), 0, "{log}");
  for note in [
    r#"role="received">Received (1 May 2020)</pubnote>"#,
    r#"role="revised">Revised (2 June 2020)</pubnote>"#,
  ] {
    assert!(xml.contains(note), "{note}\n{xml}");
  }
  // ws-mpla.cls:451-482: the three numbered lists and the alphlist's nested enumerate (its `\theenumii`).
  assert_eq!(xml.matches("<enumerate").count(), 4, "{xml}");
  assert_eq!(xml.matches("<itemize").count(), 1, "{xml}");
  for label in ["(i)", "(ii)", "(a)", "(1)"] {
    assert!(xml.contains(&format!(">{label}</tag>")), "{label}\n{xml}");
  }
  assert_eq!(
    xml.matches(">(i)</tag>").count(),
    2,
    "romanlist's first item and the nested item\n{xml}"
  );
  for defined in ["PSFIG", "EPSFIG", "EPSFBOX"] {
    assert!(xml.contains(defined), "{defined}\n{xml}");
  }
}

/// ws-rv9x6.cls:27 `\newif\if@addchapnum`, which papers test in their own numbering macros (witnesses 2103.07802,
/// 2008.12305).
#[test]
fn ws_rv9x6_defines_addchapnum() {
  let (log, xml) = latexml::util::test::convert_with(
    r"\documentclass{ws-rv9x6}
\begin{document}
\makeatletter\if@addchapnum CHAPNUM\else NOCHAPNUM\fi\makeatother
\end{document}
",
    Some("ar5iv.sty"),
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert!(xml.contains("NOCHAPNUM"), "{xml}");
}
