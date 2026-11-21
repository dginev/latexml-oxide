//! Red/green guards for batch REGFIXP, four v0.8.0 regressions: the frontmatter emitter's run after a parameterless
//! macro, SIunits' siunitx machinery and siunitx's free commands, thm-restate's frozen number in a wrapper theorem, and
//! amsppt's `\endroster` inside a group. Each guard converts its repro under `tools/perfect_kernel/repros/` with the
//! ar5iv preload and asserts the diagnostics by count and the whole element.
use latexml::util::test::{assert_element, convert_with, xml_element};

use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch84b::creators,
};

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!("../../../tools/perfect_kernel/repros/", $path))
  };
}

/// A name's run ending at a parameterless macro (`\todaysdate`) does not take the `\vspace` after it as that macro's
/// argument (`emit::past_what_it_set`, `content::reads_no_argument`): red (REGFIXJ3) 2 errors, 2 warnings — the `\vspace`
/// replayed without its `{-3ex}` read the wrapper's `\let` (Missing number) and left `\lx@saved@inst` undefined in each
/// name. `\vspace*` is the starred sibling. Witnesses 2401.17245, 2505.18108, 2205.07842, 1807.07558, 1605.01407.
/// Distinct from the emitter's `reads_past` guards (a call cut from its arguments by its own span): here the run grew.
#[test]
fn a_parameterless_macro_before_a_skip_keeps_the_name() {
  let (log, xml) = convert_with(
    repro!("sectioning-frontmatter/emitter_parameterless_macro_before_a_skip_keeps_the_name.tex"),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(
    creators(&xml),
    vec![
      vec!["Cristina Anghel".to_string()],
      vec!["1 May 2024".to_string()],
      vec!["Bob Baker 2 May 2024".to_string()],
    ],
    "{xml}"
  );
}

/// The SIunits binding loads siunitx for its machinery only: siunitx's `\begin{document}` no longer makes its free unit
/// commands `\ERROR` (`six_command_create`), which real SIunits.sty, loading no siunitx, never defines — a document's own
/// `\newcommand{\Pa}` there stands. Red (REGFIXJ3): 1 error (`undefined:\ERROR`), each `\newcommand` ignored. Witnesses
/// 1301.1500, 1705.09090. Distinct from `siunitx_unit_commands_stay_undefined_outside_units` (real siunitx: `\ERROR`
/// stays) and SIunits' own units (`stub-dependencies/siunits_unit_commands.tex`).
#[test]
fn siunits_leaves_siunitx_free_commands_free() {
  let (log, xml) = convert_with(
    repro!("stub-dependencies/siunits_document_command_named_as_a_siunitx_unit.tex"),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    !log.contains("Info:ignore:\\Pa") && !log.contains("Info:ignore:\\J"),
    "{log}"
  );
  for (id, tex) in [
    ("p1.m1", r"\hat{P}_{c}"),
    ("p1.m2", r"\mathbf{J}"),
    ("p1.m3", r"3\text{\,}\mathrm{Pa}"),
  ] {
    let attr = format!("xml:id=\"{id}\"");
    let found = xml_element(&xml, "Math", &[&attr]).unwrap_or_default();
    assert!(
      found.contains(&format!("tex=\"{tex}\"")),
      "{id}: {found}\n{xml}"
    );
  }
}

/// A `{restatable}` theorem freezes the number of the counter its head stepped (`\@currentlabel`), not
/// `\csname the<env>\endcsname`, which a wrapper environment lacks (ltexpprt.sty:216-217
/// `\newenvironment{theorem}{\begin{@theorem}}`). Red (REGFIXJ3): `undefined:\thetheorem` in the restated head.
/// The one Warning is the `{restatable}` group still open at `\end{document}` (its `\end{\@currenvir}` ends `@theorem`,
/// not the wrapper), reported apart. Witnesses 1911.08418, 2410.24104, 2503.21951. Distinct from
/// `perfect_kernel_batch78i::restated_theorem_prints_its_statement` (a `\newtheorem` environment, `\the<env>` defined).
#[test]
fn restatable_in_a_wrapper_theorem_is_numbered_by_its_counter() {
  let (log, xml) = convert_with(
    repro!("package-bindings/thm_restate_restatable_numbered_by_the_stepped_counter.tex"),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(log.contains("Environment restatable opened by"), "{log}");
  assert_element(
    &xml,
    "theorem",
    &["xml:id=\"S2.Thmtheoremx1\""],
    r#"<theorem class="ltx_theorem_theorem" inlist="thm theorem:@theorem" xml:id="S2.Thmtheoremx1"><tags><tag>Theorem 1.1</tag><tag role="refnum">1.1</tag><tag role="typerefnum">Theorem 1.1</tag></tags><title class="ltx_runin"><tag><text font="bold" xml:id="S2.Thmtheoremx1.1">Theorem 1.1</text></tag><text font="bold" xml:id="S2.Thmtheoremx1.2">.</text></title><para xml:id="S2.Thmtheoremx1.p1"><p xml:id="S2.Thmtheoremx1.p1.1"><text font="italic" xml:id="S2.Thmtheoremx1.p1.1.1">Every graph is fine.</text></p></para></theorem>"#,
  );
}

/// amsppt's `\endroster` (amsppt.sty:738-751, no group of its own) ends the roster at any group depth: met inside a
/// group the roster's body opened, it closes that group with a Warning (`until_terminal_inside_group`). Red (REGFIXJ3):
/// the roster ran to the document's end, "malformed: ltx:document … enumerate". Two shapes on the one path: the last
/// item's `{\it` never closed (alg-geom9511013, math0511546, math9901010) and `\roster {\item…` (math0309287). Distinct
/// from `perfect_kernel_batchregfixi::amsppt_input_roster_items_close` and the #868 roster guards: an unbalanced body.
#[test]
fn endroster_ends_the_roster_inside_a_group() {
  let (log, xml) = convert_with(
    repro!("amstex/amsppt_roster_endroster_inside_an_items_group.tex"),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(log.contains(r"Warning:unexpected:\endroster"), "{log}");
  assert_element(
    &xml,
    "para",
    &["xml:id=\"p1\""],
    r#"<para xml:id="p1"><enumerate xml:id="Ix1"><item xml:id="Ix1.ix1"><tags><tag>[S-D]</tag></tags><para xml:id="Ix1.ix1.p1"><p xml:id="Ix1.ix1.p1.1">B. St.-Donat, <text font="italic" xml:id="Ix1.ix1.p1.1.1">Sur les equations, C.R. Acad. Sci. Paris </text><text font="bold" xml:id="Ix1.ix1.p1.1.2">274</text><text font="italic" xml:id="Ix1.ix1.p1.1.3"> (1972).</text></p></para></item></enumerate><p xml:id="p1.1">After.</p></para>"#,
  );
  let (log, xml) = convert_with(
    repro!("amstex/amsppt_roster_open_brace_red.tex"),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert_element(
    &xml,
    "section",
    &["xml:id=\"C0.Sx1\""],
    r#"<section inlist="toc" xml:id="C0.Sx1"><title>1. Brace</title><para xml:id="C0.Sx1.p1"><enumerate xml:id="Ix1"><item xml:id="Ix1.ix1"><tags><tag>(i)</tag></tags><para xml:id="Ix1.ix1.p1"><p xml:id="Ix1.ix1.p1.1">One,</p></para></item><item xml:id="Ix1.ix2"><tags><tag>(ii)</tag></tags><para xml:id="Ix1.ix2.p1"><p xml:id="Ix1.ix2.p1.1">two.</p></para></item></enumerate><p xml:id="C0.Sx1.p1.1">After.</p><enumerate xml:id="Ix2"><item xml:id="Ix2.ix1"><tags><tag>(1)</tag></tags><para xml:id="Ix2.ix1.p1"><p xml:id="Ix2.ix1.p1.1">Next.</p></para></item></enumerate></para></section>"#,
  );
}
