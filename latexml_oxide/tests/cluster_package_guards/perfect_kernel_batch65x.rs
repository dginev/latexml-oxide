//! Red/green guards for batch 65x: the PoS, dmtcs-episciences, svmult, iopart, acmart, IEEEtran and elsarticle
//! bindings define what the versions of those classes define and papers call. Repros
//! `tools/perfect_kernel/repros/class-bindings/*`.
use super::perfect_kernel_batch46::{error_count, warning_count};

fn convert(tex: &str) -> String {
  let (log, xml) = latexml::util::test::convert_with(tex, Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  xml
}

fn assert_contains(xml: &str, texts: &[&str]) {
  for text in texts {
    assert!(xml.contains(text), "{text}\n{xml}");
  }
}

/// `text` is the whole content of a `<text>` element carrying `font="<font>"`.
fn assert_styled(xml: &str, font: &str, text: &str) {
  let end = xml
    .find(&format!(">{text}</text>"))
    .unwrap_or_else(|| panic!("{text}\n{xml}"));
  let open = &xml[xml[..end]
    .rfind("<text")
    .unwrap_or_else(|| panic!("{text}\n{xml}"))..end];
  assert!(
    open.contains(&format!(r#"font="{font}""#)),
    "{font} {text}\n{xml}"
  );
}

/// PoS.cls :1395 `\PoS{<identifier>}`: the contribution's identifier as a pubnote. Witnesses 0711.1871, 0806.0754, 1012.3355.
#[test]
fn pos_identifier_is_a_pubnote() {
  let xml = convert(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/pos_class_commands.tex"
  ));
  assert_contains(&xml, &[
    r#"role="pubid">PoS(LAT2009)123</pubnote>"#,
    "Text.",
  ]);
}

/// dmtcs-episciences.cls :225-236 `\publicationdetails{volume}{year}{issue}{number}{id}`: the head's reference as a
/// pubnote; :483-487 the number sets. Witness 1211.0978.
#[test]
fn dmtcs_publication_details_are_the_reference() {
  let xml = convert(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/dmtcs_class_commands.tex"
  ));
  assert_contains(&xml, &[
    r#"role="journalref">DMTCS vol. 19:3, 2017, #12</pubnote>"#,
    r#"<XMTok font="blackboard" role="UNKNOWN">N</XMTok>"#,
    r#"<XMTok font="blackboard" role="UNKNOWN">Z</XMTok>"#,
    r#"<XMTok font="blackboard" role="UNKNOWN">P</XMTok>"#,
    r#"<pubnote role="license">Distributed under a Creative Commons Attribution 4.0 International License</pubnote>"#,
    r#"role="copyright">2017 by the author(s)</date>"#,
  ]);
}

/// svmult.cls (2605.18420's copy): `\orcidID` an ORCID contact (:2192), the titled boxes (:2497-2525), `\ethics`
/// (:2575-2577), `\biblstarthook` under the bibliography heading (:2029), `\subsubruninhead` (:754-758). Witness
/// 2605.18420.
#[test]
fn svmult_class_commands_are_defined() {
  let xml = convert(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/svmult_class_commands.tex"
  ));
  assert_styled(&xml, "bold", "Key Point");
  assert_styled(&xml, "bold", "Competing Interests");
  assert_contains(&xml, &[
    "0000-0001-2345-6789",
    "The text of the box.",
    "The authors declare none.",
    "References are listed alphabetically.",
    "Detail",
  ]);
  // Under the first bibliography's heading, before its list, and once (the class's hook is one-shot, :2045).
  let bib = xml.find("<bibliography").unwrap();
  let (title, hook, list) = (
    bib + xml[bib..].find("</title>").unwrap(),
    xml.find("References are listed").unwrap(),
    bib + xml[bib..].find("<biblist").unwrap(),
  );
  assert!(title < hook && hook < list, "{xml}");
  assert_eq!(xml.matches("References are listed").count(), 1, "{xml}");
}

/// The svmult 2018+ additions are guesses for older copies: a paper's own `{important}` stands.
#[test]
fn svmult_additions_yield_to_the_paper() {
  let xml = convert(
    r"\documentclass{svmult}
\newenvironment{important}{\par MINE: }{\par}
\begin{document}
\begin{important}x\end{important}
\end{document}
",
  );
  assert_contains(&xml, &["MINE: x"]);
}

/// iopart.cls (2603.00348's copy :1059-1140): the 2011-2014 journal abbreviations, italic. Witness 2603.00348.
#[test]
fn iopart_newer_journals_are_defined() {
  let xml = convert(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/iopart_journals.tex"
  ));
  assert_contains(&xml, &[r#"Theor.</text> <text font="bold""#]);
  for journal in [
    "J. Phys. A: Math. Theor.",
    "J. Stat. Mech.",
    "Nucl. Fusion",
    "Mach. Learn.: Sci. Technol.",
  ] {
    assert_styled(&xml, "italic", journal);
  }
}

/// The newer iopart journals are guesses for older copies: a paper's own `\NF` stands.
#[test]
fn iopart_newer_journals_yield_to_the_paper() {
  let xml = convert(
    r"\documentclass{iopart}
\newcommand{\NF}{NuclFus}
\begin{document}
See \NF.
\end{document}
",
  );
  assert_contains(&xml, &["See NuclFus."]);
}

/// acmart.cls (TL 2025): `\noindentparagraph`, `\anon`, `\AtBeginMaketitle`, the `acmcp` setters, and the badges with
/// their arguments consumed (Perl's `\acmBadgeL` took none).
#[test]
fn acmart_class_commands_are_defined() {
  let xml = convert(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/acmart_class_commands.tex"
  ));
  assert_contains(&xml, &[
    "Head",
    "Our tool works.",
    "Hooked.",
    r#"<date role="copyrightyear">2022"#,
  ]);
  // The hook's `\setcopyright{none}` ran before the title block read the copyright: no license statement.
  assert!(!xml.contains(r#"role="license""#), "{xml}");
  for absent in ["badge-file", "example.org/badge", "example.org/code"] {
    assert!(!xml.contains(absent), "{absent}\n{xml}");
  }
}

/// IEEEtran.cls (TL 2025): `{LaTeXdescription}`, `\yesnumber`, and the struts and margins that set nothing here.
#[test]
fn ieeetran_class_commands_are_defined() {
  let xml = convert(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/ieeetran_class_commands.tex"
  ));
  assert_contains(&xml, &["Term", "Its meaning.", "A strut."]);
  assert!(
    xml.contains("<equation") && xml.contains(r#"<tag role="refnum">1</tag>"#),
    "{xml}"
  );
}

/// elsarticle.cls :153-176: `\cormark[n]`/`\fnmark[n]` mark the author as `\corref`/`\fnref` do, so the notes are the
/// author's; `\nonumnote` an unnumbered title note.
#[test]
fn elsarticle_marks_are_the_references() {
  let xml = convert(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/elsarticle_marks.tex"
  ));
  let creator = &xml[xml.find("<creator").unwrap()..xml.find("</creator>").unwrap()];
  assert_contains(creator, &[
    r#"role="correspondent">Corresponding author.</contact>"#,
    r#"role="note">Now elsewhere.</contact>"#,
    r#"role="note">Also here.</contact>"#,
  ]);
  assert_contains(&xml, &["Supported by a grant."]);
}
