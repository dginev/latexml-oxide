//! Red/green guards for batch REGFIXR, six regressions of the v0.8.0 per-paper check: a package's `{gather}` over
//! amsmath's `\start@gather` (aa.cls's linenoaa.sty), a sub-caption label under a shipped caption3 v1.8h, a document's
//! addition to amsmath's `\displ@y@`, a `\fig` defined before an AASTeX 5.x or emulateapj `\documentclass`, a
//! `{restatable}` environment named by a macro, and bmcart ids with `_`/`^`. Each guard converts its repro under
//! `tools/perfect_kernel/repros/` (pdflatex 0 errors) and asserts no Error or Warning and the content in whole elements.
use latexml::util::test::{
  assert_element, check_all_requirements, convert_files_with, convert_with, normalize_markup,
  xml_element,
};

use super::perfect_kernel_batch46::{error_count, warning_count};

const PRELOAD: &str = "ar5iv.sty";

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!("../../../tools/perfect_kernel/repros/", $path))
  };
}

/// The counts `(errors, warnings)` of `log`, which must hold no Fatal.
#[track_caller]
fn counts(log: &str) -> (usize, usize) {
  assert!(!log.contains("Fatal:"), "{log}");
  (error_count(log), warning_count(log))
}

/// The `<tag role="refnum">` numbers of `xml`, in document order.
fn refnums(xml: &str) -> Vec<&str> {
  xml
    .split("<tag role=\"refnum\">")
    .skip(1)
    .filter_map(|rest| rest.split("</tag>").next())
    .collect()
}

/// amsmath.sty:1540 `\start@gather` starts `{gather}`/`{gather*}`, whose end (:1552-1557 `\math@cr \black@\totwidth@
/// \egroup \dollardollar@end`, `$$` in older amsmath) a package copies (linenoaa.sty:51-55, which aa.cls's copy loads:
/// 2407.04074 2503.08108 2503.20535 2604.03365 2608.16665): undefined, then that end's cascade, 13 errors. Each environment ends as the binding's
/// `{gather}`, numbered as pdflatex: (1) labelled e1, the `\notag` row and `{gather*}` unnumbered, a document's own
/// `{mygather}` (2), one in a list (3). Distinct from `perfect_kernel_batchregfixo`'s `\start@align` (no raw end).
#[test]
fn start_gather_ends_a_package_gather() {
  let (log, xml) = convert_with(
    repro!("alignment/start_gather_by_a_package.tex"),
    Some(PRELOAD),
  );
  assert_eq!(counts(&log), (0, 0), "{log}");
  assert_eq!(
    xml
      .matches("<equationgroup class=\"ltx_eqn_gather\"")
      .count(),
    4,
    "{xml}"
  );
  assert_eq!(refnums(&xml), ["1", "2", "3"], "{xml}");
  assert!(xml.contains("<equation labels=\"LABEL:e1\""), "{xml}");
  for text in [
    "text=\"a = b\"",
    "text=\"c = d\"",
    "text=\"e = f\"",
    "text=\"g = h\"",
    "text=\"i = j\"",
    "text=\"k = l\"",
    "<p xml:id=\"S0.I1.i1.p1.2\">after.</p>",
    "See (<ref labelref=\"LABEL:e1\"/>).",
  ] {
    assert!(xml.contains(text), "{text}\n{xml}");
  }
}

/// A caption3 before v2.0 (v1.8h:619-620, the copy 2010.15148 2011.14129 2102.00283 2112.02137 2201.07802 ship, which
/// raw floatrow loads, so `subfig[caption=false]` loads no caption binding) names a label format
/// `\caption@lfmt@<name>`: the sub-caption label is `(a)`, not an undefined `\caption@labelformat@parens`.
#[test]
fn subfig_label_under_an_old_caption3() {
  if !check_all_requirements(&[&["floatrow.sty"]]) {
    return;
  }
  let (log, xml) = convert_files_with(
    repro!("captions-floats/caption3_v18h/subfig_label_under_old_caption3.tex"),
    &[(
      "caption3.sty",
      repro!("captions-floats/caption3_v18h/caption3.sty"),
    )],
    Some(PRELOAD),
  );
  assert_eq!(counts(&log), (0, 0), "{log}");
  assert_eq!(refnums(&xml), ["1", "1a", "1b"], "{xml}");
  for (panel, tag, text) in [("S0.F1.sf1", "(a)", "A"), ("S0.F1.sf2", "(b)", "B")] {
    let found = xml_element(&xml, "figure", &[&format!("xml:id=\"{panel}\"")]).unwrap_or_default();
    assert!(
      found.contains(&format!(
        "<caption><tag close=\" \">{tag}</tag>{text}</caption>"
      )),
      "{found}\n{xml}"
    );
  }
}

/// amsmath.sty:1344-1363 `\displ@y@`, which a document extends (`\g@addto@macro\displ@y@{\global\let\eq@name\relax}`:
/// 1406.6618 2301.11270 2304.01203 2401.16335 2402.12617); undefined, `\g@addto@macro` errs as in TeX (#843).
#[test]
fn amsmath_displ_y_takes_an_addition() {
  let (log, xml) = convert_with(
    repro!("ams-internals/amsmath_displ_y_addition.tex"),
    Some(PRELOAD),
  );
  assert_eq!(counts(&log), (0, 0), "{log}");
  assert_eq!(
    xml
      .matches("<equationgroup class=\"ltx_eqn_align\"")
      .count(),
    1,
    "{xml}"
  );
  assert_eq!(refnums(&xml), ["1"], "{xml}");
  assert!(xml.contains("text=\"a = b\""), "{xml}");
}

/// AASTeX 5.x and emulateapj have no `\fig` (aastex.cls 5.x, emulateapj.cls); the binding's AASTeX 6+ `\fig` family
/// gives way to the meaning before the class, a paper's own `\fig` (0910.5598; 1002.3626 astro-ph0511141, 1301.6182),
/// or none.
#[test]
fn aastex5_and_emulateapj_keep_a_preclass_fig() {
  for (tex, requires) in [
    (
      repro!("class-bindings/aastex5_keeps_a_preclass_fig.tex"),
      None,
    ),
    (
      repro!("class-bindings/emulateapj_keeps_a_preclass_fig.tex"),
      Some("emulateapj.cls"),
    ),
  ] {
    if let Some(file) = requires
      && !check_all_requirements(&[&[file]])
    {
      continue;
    }
    let (log, xml) = convert_with(tex, Some(PRELOAD));
    assert_eq!(counts(&log), (0, 0), "{log}");
    assert!(
      xml.contains("See Figure\u{a0}<ref labelref=\"LABEL:f\"/>.</p>"),
      "{xml}"
    );
  }
  let (log, _) = convert_with(
    "\\documentclass{aastex}\n\\begin{document}\n\\fig{x}\n\\end{document}\n",
    Some(PRELOAD),
  );
  assert!(
    log.contains("Error:undefined:\\fig"),
    "AASTeX 5.x defines no \\fig: {log}"
  );
}

/// thm-restate.sty passes `{restatable}`'s `#2` to `\begin`, which expands it: `{restatable}{\retheorem}{again}` with
/// `\newcommand\retheorem{theorem}` (1410.0855 1411.0871 1411.1619 1504.05476 1511.01379) restates a theorem; the
/// replay's `\begin` read the name as the characters `\retheorem`. `{restatable*}` alike.
#[test]
fn restatable_environment_named_by_a_macro() {
  let (log, xml) = convert_with(
    repro!("package-bindings/thm_restate_environment_named_by_a_macro.tex"),
    Some(PRELOAD),
  );
  assert_eq!(counts(&log), (0, 0), "{log}");
  assert_element(
    &xml,
    "theorem",
    &["xml:id=\"Thmtheoremx2\""],
    "<theorem class=\"ltx_theorem_theorem\" inlist=\"thm theorem:theorem\" xml:id=\"Thmtheoremx2\"><tags><tag>Theorem \
     1</tag><tag role=\"refnum\">1</tag><tag role=\"typerefnum\">Theorem 1</tag></tags><title class=\"ltx_runin\"><tag><text \
     font=\"bold\" xml:id=\"Thmtheoremx2.1\">Theorem 1</text></tag><text font=\"bold\" xml:id=\"Thmtheoremx2.2\"> \
     </text>(Named)<text font=\"bold\" xml:id=\"Thmtheoremx2.3\">.</text></title><para xml:id=\"Thmtheoremx2.p1\"><p \
     xml:id=\"Thmtheoremx2.p1.1\"><text font=\"italic\" xml:id=\"Thmtheoremx2.p1.1.1\">A \
     claim.</text></p></para></theorem>",
  );
  assert_eq!(
    xml.matches("class=\"ltx_theorem_theorem\"").count(),
    2,
    "{xml}"
  );
  assert_eq!(
    xml.matches("class=\"ltx_theorem_lemma\"").count(),
    2,
    "{xml}"
  );
  assert!(xml.contains("<tag>Lemma 2</tag>"), "{xml}");
  assert_eq!(xml.matches("A lemma.").count(), 2, "{xml}");
}

/// bmcart.cls reads an author's `addressref`/`noteref` and an address's or note's `id` only inside `\csname`
/// (:1137-1154, :1284-1289): `aalto_cs`, `mit^x`, `n_1` are names, not math scripts (2009.00252, 1403.3568), and each
/// author keeps the affiliations and the note its ids name.
#[test]
fn bmcart_ids_are_characters() {
  let (log, xml) = convert_files_with(
    repro!("sectioning-frontmatter/bmcart_copy/bmcart_ids_as_characters.tex"),
    &[(
      "bmcart.cls",
      repro!("sectioning-frontmatter/bmcart_copy/bmcart.cls"),
    )],
    Some(PRELOAD),
  );
  assert_eq!(counts(&log), (0, 0), "{log}");
  let creators: Vec<String> = xml
    .match_indices("<creator ")
    .filter_map(|(at, _)| xml_element(&xml[at..], "creator", &[]))
    .collect();
  let expected: Vec<String> = [
      "<creator role=\"author\"><personname>Daniel Monsivais</personname><contact name=\"Email:\u{a0}\" \
       role=\"email\">a@b.c</contact><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Aalto \
       University</contact><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">MIT</contact><contact \
       name=\"Note:\u{a0}\" role=\"note\">Equal contributor</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Ann Other</personname><contact \
       name=\"Affiliation:\u{a0}\" role=\"affiliation\">MIT</contact></creator>",
  ]
  .iter()
  .map(|creator| normalize_markup(creator))
  .collect();
  assert_eq!(creators, expected, "{xml}");
}
