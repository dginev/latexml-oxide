//! Red/green guards for batch 65w: the lmcs, lipics, atlasdoc, spie (spieman), arximspdf and aa bindings define what
//! the versions of those classes define and papers call. Repros `tools/perfect_kernel/repros/class-bindings/*`.
use super::perfect_kernel_batch46::error_count;

fn convert(tex: &str) -> String {
  let (log, xml) = latexml::util::test::convert_with(tex, Some("ar5iv.sty"));
  assert_eq!(error_count(&log), 0, "{log}");
  xml
}

fn assert_contains(xml: &str, texts: &[&str]) {
  for text in texts {
    assert!(xml.contains(text), "{text}\n{xml}");
  }
}

/// lmcs.cls (1809.04468's copy :252-280, 1403.4094's :490-507): `\titlecomment` a title note, `\ACMCCS`/`\amsclass`
/// classifications, `\rsuper`, `\qEd`, `{iteMize}{label}` items carrying the label. Witnesses 2103.15292, 1309.5822,
/// 2103.16592, 1107.1351, 1205.4458, cs0510066.
#[test]
fn lmcs_class_commands_are_defined() {
  let xml = convert(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/lmcs_class_commands.tex"
  ));
  assert_contains(&xml, &[
    r#"role="titlenote""#,
    "An extended abstract appeared at a conference.",
    r#"scheme="ccs""#,
    "Theory of computation",
    r#"scheme="AMS""#,
    "03B70",
    r#"<tag><Math mode="inline" tex="\bullet""#,
    "\u{25A1}",
  ]);
}

/// lipics-v2021.cls (2609.11285's copy): `{claimproof}` (:1041-1056), the starred theorems (:1158-1176),
/// `\lipicsEnd` (:1057-1060), `\proofsubparagraph` (:380-383), the `(i)`/`(a)` enumerates (:537-542), and the older
/// versions' front-matter setters. Witnesses 2508.16319, 2508.09892, 2008.01422, 2103.15659, 1904.11668, 2103.15651.
#[test]
fn lipics_class_commands_are_defined() {
  let xml = convert(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/lipics_class_commands.tex"
  ));
  assert_contains(&xml, &[
    "<proof",
    "It holds.",
    "A remark.",
    ">(i)</tag>",
    ">(ii)</tag>",
    ">(a)</tag>",
    "Case 1",
    "\u{231F}",
    r#"role="event""#,
    "A Conference",
  ]);
}

/// atlasdoc.cls (2504.04805's copy :286-345): the document-data setters store what the title page prints and become
/// pubnotes; `\AtlasPaperWebpage` reads the reference code. Witnesses 2508.20929, 2103.01029, 1904.03536, 2407.10879.
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
}

/// spieman.cls :82 `\linkable` (the loader gives spieman the spie binding) and the v1.00 `\supscr`. Witnesses
/// 2604.17554, 1408.0407.
#[test]
fn spieman_linkable_is_defined() {
  let xml = convert(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/spieman_class_commands.tex"
  ));
  assert_contains(&xml, &["someone@example.org"]);
}

/// arximspdf.cls (0911.2919's copy): `\doiurl`, `\arxivurl`, `\surl` (:3077-3162), `\setattribute`/`\getattribute`
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

/// aa.cls's online-only floats (0709.2515's :925-941) stay in place numbered as the class numbers them, and the
/// following floats continue from there; `\psj`. Witnesses 0709.2515, 0709.0404, 2508.13727.
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
    "PSJ",
  ]);
}
