//! Red/green guards for batch 65w: the lmcs, lipics, atlasdoc, spie (spieman), arximspdf and aa bindings define what
//! the versions of those classes define and papers call. Repros `tools/perfect_kernel/repros/class-bindings/*`.
use latexml::util::test::convert_files_with;

use super::perfect_kernel_batch46::{error_count, warning_count};

fn convert(tex: &str) -> String {
  let (log, xml) = latexml::util::test::convert_with(tex, Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  xml
}

/// spieman has no binding of its own: the loader's one warning that it takes spie's, and nothing else.
fn convert_spieman(tex: &str) -> String {
  let (log, xml) = latexml::util::test::convert_with(tex, Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(
    log.contains("Can't find binding for class spieman (using spie)"),
    "{log}"
  );
  xml
}

fn assert_contains(xml: &str, texts: &[&str]) {
  for text in texts {
    assert!(xml.contains(text), "{text}\n{xml}");
  }
}

/// The texts occur in this order in the XML.
fn assert_in_order(xml: &str, texts: &[&str]) {
  let at = |text: &str| xml.find(text).unwrap_or_else(|| panic!("{text}\n{xml}"));
  for pair in texts.windows(2) {
    assert!(
      at(pair[0]) < at(pair[1]),
      "{} before {}\n{xml}",
      pair[0],
      pair[1]
    );
  }
}

/// lmcs.cls (1809.04468's copy :252-280, :343, 1403.4094's :490-507): `\titlecomment` a title note, `\ACMCCS`/
/// `\amsclass` classifications named by `\ACMCCSname`/`\amsclassname`, `\rsuper`, `\qEd`, `{iteMize}{label}` items
/// carrying the label. Witnesses 2103.15292, 1309.5822, 2103.16592, 1107.1351, 1205.4458, cs0510066.
#[test]
fn lmcs_class_commands_are_defined() {
  let xml = convert(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/lmcs_class_commands.tex"
  ));
  assert_contains(&xml, &[
    r#"<note role="titlenote""#,
    ">An extended abstract appeared at a conference.</note>",
    r#"<classification name="2012 ACM CCS" scheme="ccs">["#,
    r#"<classification name="2010 Mathematics Subject Classification" scheme="AMS">03B70</classification>"#,
    "Theory of computation",
    r#"<tag><Math mode="inline" tex="\bullet""#,
    "\u{25A1}",
  ]);
}

/// lipics-v2021.cls (2609.11285's copy): `{claimproof}` ending ◁ (:1041-1056), the starred theorems (:1158-1176),
/// `\lipicsEnd` (:1057-1060), `\proofsubparagraph` (:380-383), the `(i)`/`(a)` enumerates (:572-577), and the older
/// versions' front-matter setters, `\DOI` printing its DOI (1507.03682's :391-393). Witnesses 2508.16319, 2508.09892,
/// 2008.01422, 2103.15659, 1904.11668, 2103.15651.
#[test]
fn lipics_class_commands_are_defined() {
  let xml = convert(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/lipics_class_commands.tex"
  ));
  assert_contains(&xml, &[
    "<proof",
    r#"tex="\vartriangleleft""#,
    "Inner.\u{220E}",
    "A remark.",
    ">(i)</tag>",
    ">(ii)</tag>",
    ">(a)</tag>",
    "Case 1",
    "\u{231F}",
    r#"role="event""#,
    "A Conference",
    r#"<pubnote name="DOI: " role="doi">10.4230/LIPIcs.SAMPLE.2016.1</pubnote>"#,
  ]);
  // The claim's own end is the ◁, not a ∎ besides it; the nested proof keeps its ∎.
  assert_eq!(xml.matches('\u{220E}').count(), 1, "{xml}");
  assert_eq!(xml.matches(r#"tex="\vartriangleleft""#).count(), 2, "{xml}");
}

/// lipics-v2021.cls `\claimqedhere` (:1029-1031) puts the claim's ◁ where it is called, and the claim's end adds no
/// second mark; a claim proved inside a proof leaves the proof its own end mark.
#[test]
fn lipics_claim_qed_marks() {
  let xml = convert(
    r"\documentclass{lipics-v2021}
\begin{document}
\begin{claimproof}So \[x=y.\claimqedhere\]\end{claimproof}
\end{document}
",
  );
  assert_eq!(xml.matches(r#"name="vartriangleleft""#).count(), 1, "{xml}");
  assert!(!xml.contains('\u{220E}'), "{xml}");
  let xml = convert(
    r"\documentclass{lipics-v2021}
\begin{document}
\begin{proof}Outer.\begin{claimproof}Claim.\end{claimproof}Done.\end{proof}
\end{document}
",
  );
  assert_eq!(xml.matches(r#"name="vartriangleleft""#).count(), 1, "{xml}");
  assert!(xml.contains("Done.\u{220E}"), "{xml}");
}

/// lipics-v2021.cls:1001-1005 `\flag[width]{image}`: the image (an artifact badge the class sets in the margin),
/// inline.
#[test]
fn lipics_flag_is_the_image() {
  let xml = convert(
    r"\documentclass{lipics-v2021}
\begin{document}
Artifact.\flag[1cm]{badge-available}
\end{document}
",
  );
  assert_contains(&xml, &[r#"graphic="badge-available""#]);
}

/// The older lipics versions' setters are guesses (17-19 of 92 sampled copies): a lipics-v2021 paper's own `\DOI`
/// replaces the binding's, which would otherwise make every DOI it prints a pubnote.
#[test]
fn lipics_old_setters_yield_to_the_paper() {
  let xml = convert(
    r"\documentclass{lipics-v2021}
\newcommand{\DOI}[1]{doi:#1}
\begin{document}
See \DOI{10.1/x}.
\end{document}
",
  );
  assert_contains(&xml, &["See doi:10.1/x."]);
  assert!(!xml.contains(r#"role="doi""#), "{xml}");
}

/// atlasdoc.cls (2504.04805's copy :286-345): the document-data setters store what the title page prints (:835-856),
/// the last call winning, and become pubnotes once, the journal reference replacing "Submitted to: <journal>";
/// `\AtlasPaperWebpage` reads the reference code. Witnesses 2508.20929, 2103.01029, 1904.03536, 2407.10879.
#[test]
fn atlasdoc_setters_are_defined() {
  let xml = convert(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/atlasdoc_class_commands.tex"
  ));
  assert_contains(&xml, &[
    r#"role="preprint">CERN-EP-2025-001</pubnote>"#,
    r#"role="journalref">Phys. Lett. B 800 (2025) 1</pubnote>"#,
    r#"role="doi">10.1016/j.physletb.2025.1</pubnote>"#,
    "PAPERS/STDM-2018-01",
  ]);
  for absent in ["Submitted to", "CERN-EP-2024-999"] {
    assert!(!xml.contains(absent), "{absent}\n{xml}");
  }
  assert_eq!(xml.matches(r#"role="doi""#).count(), 1, "{xml}");
}

/// atlasdoc.cls picks its title header by option (2504.04805's :82-91, :991-1006), each prefix the note's label: by
/// default "<\\subprepend> <journal>" ("Journal:", :746-747) and neither the journal reference nor the DOI; with
/// `preprint` (spaces and `=true` allowed, the last setting winning) and no journal reference, "Submitted to: <journal>"
/// (:835-841); with `HEPDATA` the preprint number and arXiv id (:866-893); with `BOOK` none, unless `PAPER` turns it
/// off (:140-192). The arXiv and HEPData setters (:324-333) are defined.
#[test]
fn atlasdoc_title_header_follows_the_options() {
  let doc = |options: &str| {
    convert(&format!(
      r"\documentclass{options}{{atlasdoc}}
\AtlasTitle{{T}}
\AtlasJournal{{Phys. Lett. B}}
\PreprintIdNumber{{CERN-EP-1}}
\AtlasDOI{{10.1/x}}
\arXivId{{2504.04805}}
\HepDataRecord{{1234}}
\HEPDataRecord{{1234}}
\begin{{document}}
\maketitle
\AtlasArXivURL{{}} \AtlasHEPDataRecord
\end{{document}}
"
    ))
  };
  let journal = |label: &str| {
    format!("<pubnote name=\"{label}\u{a0}\" role=\"journal\">Phys. Lett. B</pubnote>")
  };
  let xml = doc("");
  assert_contains(&xml, &[&journal("Journal:")]);
  for absent in [r#"role="doi""#, r#"role="preprint""#, "Submitted to"] {
    assert!(!xml.contains(absent), "{absent}\n{xml}");
  }
  let xml = doc("[PAPER, preprint = true]");
  assert_contains(&xml, &[
    &journal("Submitted to:"),
    r#"role="preprint">CERN-EP-1</pubnote>"#,
    r#"role="doi">10.1/x</pubnote>"#,
  ]);
  assert_contains(&xml, &[
    "https://arXiv.org/abs/2504.04805",
    "https://www.hepdata.net/record/ins1234",
  ]);
  let xml = doc("[preprint, preprint=false]");
  assert_contains(&xml, &[&journal("Journal:")]);
  let xml = doc("[HEPDATA]");
  assert_contains(&xml, &[
    r#"role="preprint">CERN-EP-1</pubnote>"#,
    r#"role="arxiv">2504.04805 [hep-ex]</pubnote>"#,
  ]);
  assert!(
    !xml.contains(r#"role="journal""#) && !xml.contains(r#"role="doi""#),
    "{xml}"
  );
  let xml = doc("[BOOK]");
  assert!(!xml.contains("<pubnote"), "{xml}");
  let xml = doc("[PAPER,BOOK]");
  assert_contains(&xml, &[&journal("Journal:")]);
}

/// spieman.cls :82 `\linkable` (the loader gives spieman the spie binding) and the v1.00 `\supscr`. Witnesses
/// 2604.17554, 1408.0407.
#[test]
fn spieman_linkable_is_defined() {
  let xml = convert_spieman(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/spieman_class_commands.tex"
  ));
  assert_contains(&xml, &[
    r##"<text class="ltx_underline" color="#0000FF" xml:id="p1.1.1">someone@example.org</text>"##,
    r#"<text fontsize="90%" yoffset="3.4pt" xml:id="p1.1.2">a</text>"#,
  ]);
}

/// spieman's `\linkable` is a guess for spie's binding (`DefinitionOrigin::Fallback`): a paper's own definition wins.
#[test]
fn spieman_linkable_yields_to_the_paper() {
  let xml = convert_spieman(
    r"\documentclass{spieman}
\newcommand{\linkable}[1]{X#1}
\begin{document}
\linkable{a}
\end{document}
",
  );
  assert_contains(&xml, &["Xa"]);
  assert!(!xml.contains("ltx_underline"), "{xml}");
}

/// arximspdf.cls (0911.2919's copy): `\doiurl`, `\arxivurl`, `\surl` (:3057-3162), `\setattribute`/`\getattribute`
/// (:336-338), `\tsup`, `\OrigBibText…\endOrigBibText`. Witnesses 1309.4864, 1309.5877, 1309.1380, 1107.3284.
#[test]
fn arximspdf_helpers_are_defined() {
  let xml = convert(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/arximspdf_class_commands.tex"
  ));
  assert_contains(&xml, &[
    r#"href="http://dx.doi.org/10.1214/x""#,
    r#"href="http://arxiv.org/abs/math/0601001""#,
    r#"href="http://www.example.org""#,
    "ABC",
  ]);
  assert!(!xml.contains("ignored"), "{xml}");
}

/// aa.cls up to v7's online-only floats (0801.4139's :931-940): set at the document's end, numbered N, the counter
/// stepped to N for what follows (`\onllongtab` with a longtable); `\longtabL` in place, numbered N (:939); `\psj`.
/// Witnesses 0709.2515, 0709.0404, 0709.4508, 2508.13727.
#[test]
fn aa_online_floats_keep_their_numbers() {
  let xml = convert(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/aa_online_floats.tex"
  ));
  assert_contains(&xml, &[
    "Table 1</tag>First.",
    "Table 2</tag>Online.",
    "Table 3</tag>Third.",
    "Figure 1</tag>Online figure.",
    "Table 4</tag>Long online.",
    "Table 5</tag>Fifth.",
    "Table 6</tag>Landscape.",
    "PSJ",
  ]);
  assert_in_order(&xml, &[
    "First.",
    "Third.",
    "Fifth.",
    "Landscape.",
    "Online.",
    "Online figure.",
    "Long online.",
  ]);
}

/// aa.cls v8.x's one-argument online floats (1403.1122's v8.1 :1366-1395, `\onlfloat`): the float, deferred to the
/// document's end, keeps the number it reserved where it was called; its paragraphs kept. Witnesses 1403.1122,
/// 1505.04356.
#[test]
fn aa_v8_online_floats_are_the_float() {
  let xml = convert(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/aa_online_floats_v8.tex"
  ));
  assert_contains(&xml, &[
    "Figure 1</tag>Before.",
    "Figure 2</tag>Online figure.",
    "Figure 3</tag>After.",
    "Table 1</tag>Online table.",
    "Table 2</tag>Long online.",
    "Online body.",
    "Second paragraph.",
  ]);
  assert_in_order(&xml, &[
    "Before.",
    "After.",
    "Online body.",
    "Second paragraph.",
    "Online table.",
    "Long online.",
  ]);
}

/// A v9.x aa.cls copy that loads hyperref at the document's start (2409.03418's :2035-2037) gives its paper `\href`;
/// without a copy, hyperref is the paper's to load. Witness 2406.05179.
#[test]
fn aa_follows_a_shipped_copy_loading_hyperref() {
  let (log, xml) = convert_files_with(
    r"\documentclass{aa}
\begin{document}
See \href{http://example.org}{the site}.
\end{document}
",
    &[(
      "aa.cls",
      r"\ProvidesClass{aa}[2023/01/01 v9.4 Class for AA journal]
\AtBeginDocument{\@ifpackageloaded{hyperref}{}{\RequirePackage[colorlinks=true,linkcolor=blue]{hyperref}}}
",
    )],
    Some("ar5iv.sty"),
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert_contains(&xml, &[r#"href="http://example.org""#, "the site"]);
}
