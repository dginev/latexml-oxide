//! Red/green guards for perfect-kernel batch 67n: D1 ("a locked binding owns its calls", `state::owns_calls`) extends to
//! robust bindings (user ruling 2026-10-15). A robust binding's own meaning is the parameterless `\protect \cs␣`; it is
//! judged by its inner `\cs␣`, whose signature also gives the tokens that open a call (`state::call_openers`). The
//! kernel `\cite` (sect11.rs, `robust => true, locked => true`, OXIDIZED_DESIGN #88) is the one such binding: eplain's
//! btxmac scratch `\edef\cite{\the\catcode`@}` (eplain.tex:346-348, :666) was refused, so `\catcode`@=\@oldatcatcode`
//! read no number and `@` became an escape character (Fatal `ParamSpec` at eplain.tex:721; math0308041, math0612670,
//! 0705.2148, 0802.3843, 1207.1752, 1401.4197). Repros `tools/perfect_kernel/repros/macro-state/m5_robust_cite_*.tex`.
use latexml::util::test::assert_element;

use super::perfect_kernel_batch46::{convert_with, convert_with_then, error_count, warning_count};

/// The log and XML of `tex` converted under ar5iv, which must give no error and no warning.
fn convert_clean_log(tex: &str) -> (String, String) {
  let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  (log, xml)
}

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/macro-state/",
      $name,
      ".tex"
    ))
  };
}

/// The binding's citation of `knuth`, as the kernel `\cite` builds it.
const KNUTH_CITE: &str = r#"<cite class="ltx_citemacro_cite">[<bibref bibrefs="knuth" separator="," yyseparator=","/>]</cite>"#;

/// eplain's btxmac scratch value of `\cite` is the meaning: `\let\x=\cite` copies it, `\catcode`@=\x` reads 11 (base:
/// `Missing number`, `@` an escape character, then Fatal `ParamSpec` at `\def\@Fornoop#1\@@#2#3{}`), and the braced
/// call still runs the binding (pdflatex: `[11] and 11knuth.`).
#[test]
fn robust_cite_scratch_value_is_the_meaning() {
  let (log, xml) = convert_clean_log(repro!("m5_robust_cite_edef_scratch_eplain"));
  assert_eq!(log.matches("Info:lock:\\cite").count(), 1, "{log}");
  assert!(!log.contains("Ignoring redefinition of \\cite"), "{log}");
  assert_element(
    &xml,
    "p",
    &[],
    &format!(r#"<p xml:id="p1.1">[11] and {KNUTH_CITE}.</p>"#),
  );
}

/// A group-local value of the robust `\cite` is what `[\cite]` prints (base: the binding read `]` as its key); the
/// call after the group is the binding's.
#[test]
fn robust_cite_value_in_group() {
  let (_log, xml) = convert_clean_log(repro!("m5_robust_cite_value_in_group"));
  assert_element(
    &xml,
    "p",
    &[],
    &format!(r#"<p xml:id="p1.1">[CITEV] and {KNUTH_CITE}.</p>"#),
  );
}

/// After a parameterless redefinition, a use without an argument takes the value, and both calls — the `[` the inner
/// `\cite␣`'s signature opens with, and an explicit `{` — keep the binding's citation (eplain.tex:445
/// `\def\cite{\@getoptionalarg\@cite}` then the document's `\cite[…]{…}`/`\cite{…}`).
#[test]
fn robust_cite_calls_keep_the_binding_after_a_value() {
  let (_log, xml) = convert_clean_log(repro!("m5_robust_cite_calls_after_value"));
  assert_element(
    &xml,
    "p",
    &[],
    &format!(
      r#"<p xml:id="p1.1">Value: CITEV. See <cite class="ltx_citemacro_cite">[<bibref bibrefs="knuth" separator="," yyseparator=","/>, p. 3]</cite> and {KNUTH_CITE}.</p>"#
    ),
  );
}

/// A redefinition with parameters defines a command, not a value: it stays refused, as in Perl, and every use is the
/// binding's.
#[test]
fn robust_cite_command_redefinition_stays_refused() {
  let (log, xml) = convert_clean_log(
    "\\documentclass{article}\n\\begin{document}\n\\renewcommand\\cite[1]{X#1}\nSee \\cite{knuth}.\n\\begin{thebibliography}{9}\\bibitem{knuth} D. Knuth, TeXbook.\\end{thebibliography}\n\\end{document}",
  );
  assert_eq!(
    log.matches("Ignoring redefinition of \\cite").count(),
    1,
    "{log}"
  );
  assert!(!log.contains("Info:lock:\\cite"), "{log}");
  assert_element(
    &xml,
    "p",
    &[],
    &format!(r#"<p xml:id="p1.1">See {KNUTH_CITE}.</p>"#),
  );
}

/// The robust kernel `\cite` (no natbib) owns its calls, judged by its inner `\cite␣`; its call openers are the `[` of
/// that signature. The owners under the bare article class follow; `\cite` is the only one whose binding is robust (sect11.rs
/// declares it `robust => true, locked => true`, the one binding that is both — the 67n review's census).
#[test]
fn robust_kernel_cite_owns_its_calls() {
  let (log, _xml, (owns, openers, owners)) = convert_with_then(
    "\\documentclass{article}\\begin{document}x\\end{document}",
    Some("ar5iv.sty"),
    |_| {
      use latexml_core::state::{
        call_openers, locked_binding_token, locked_control_sequences, lookup_meaning, owns_calls,
      };
      let cite = latexml_core::T_CS!("\\cite");
      let owns = lookup_meaning(&cite).is_some_and(|b| owns_calls(&cite, &b));
      // `call_openers` reads the kept binding: put the kernel `\cite` there, as an override would.
      let owner = locked_binding_token(&cite);
      latexml_core::state::assign_meaning(
        &owner,
        lookup_meaning(&cite).unwrap(),
        Some(latexml_core::state::Scope::Global),
      );
      let openers: Vec<String> = call_openers(&owner)
        .iter()
        .map(|t| t.with_str(ToString::to_string))
        .collect();
      let owners: Vec<String> = locked_control_sequences()
        .into_iter()
        .filter(|name| {
          let cs = latexml_core::T_CS!(name.as_str());
          lookup_meaning(&cs).is_some_and(|b| owns_calls(&cs, &b))
        })
        .collect();
      (owns, openers, owners)
    },
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  assert!(owns);
  assert_eq!(openers, ["["]);
  assert_eq!(owners, [
    r"\DeclareLowercaseMapping",
    r"\DeclareTextAccentDefault",
    r"\DeclareTitlecaseMapping",
    r"\DeclareUppercaseMapping",
    r"\author",
    r"\bibliography",
    r"\cite",
    r"\mathbf",
    r"\mathcal",
    r"\mathit",
    r"\mathnormal",
    r"\mathrm",
    r"\mathscr",
    r"\mathsf",
    r"\mathtt",
    r"\multicolumn",
    r"\newcounter",
    r"\overline",
    r"\sqrt",
    r"\title",
    r"\underline",
  ]);
}
