//! Red/green guards for batch 67s: theorem/proof machinery under raw classes. amsthm's `{proof}` is locked against
//! the class and package files (smfart.cls:1194, smfbook.cls:1281, ijmart.cls:336 `\renewenvironment{proof}` over a
//! `\trivlist`), and a theorem's shared counter named by a macro expands, as latex.ltx's `\@othm` reads it through
//! `\csname` (smfthm.sty:52-56 `[\smf@thm]`, ccthm.sty `[\cc@thmcnter]`).
use super::perfect_kernel_batch46::{error_count, warning_count};

/// A class's `\renewenvironment{proof}` after `\RequirePackage{amsthm}` keeps the binding's `<proof>`, titled by the
/// class's `\proofname` (smfart 1711.11547, 1902.02049, 2108.12000).
#[test]
fn class_renewed_proof_keeps_the_binding() {
  let (log, xml) = latexml::util::test::convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/package-bindings/amsthm_proof_kept_against_a_class_renewenvironment.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(!xml.contains("trivlist"), "{xml}");
  assert_eq!(xml.matches("<proof ").count(), 1, "{xml}");
  latexml::util::test::assert_element(
    &xml,
    "proof",
    &[],
    r#"<proof xml:id="id1">
    <title class="ltx_runin" font="italic">Démonstration.</title>
    <para xml:id="p1">
      <p xml:id="p1.1"><text xml:id="p1.1.1">Because.∎</text></p>
    </para>
  </proof>"#,
  );
}

/// A package file's `\let\proof\relax` + `\newenvironment{proof}` is refused as a class's redefinition is: the
/// clearing `\let` keeps the binding aside (state.rs `let_i`, `"\proof:locked@cleared"`) and the refused copy puts
/// it back, and the optional title still reaches the binding's proof. (The fixture is not named `myproof.sty`: Perl's
/// FindFile_fallback strips a `my` prefix, Package.pm:2186-2189, and loads the proof.sty binding in its place;
/// `convert_in_process` now fails a guard whose fixture is shadowed that way.)
#[test]
fn package_redefined_proof_keeps_the_binding() {
  let (log, xml) = latexml::util::test::convert_files_with(
    r"\documentclass{article}
\usepackage{amsthm}
\usepackage{relaxproofpkg}
\begin{document}
\begin{proof}[Proof of the claim]Because.\end{proof}
\end{document}
",
    &[(
      "relaxproofpkg.sty",
      r"\ProvidesPackage{relaxproofpkg}
\let\proof\relax\let\endproof\relax
\newenvironment{proof}[1][\proofname]{\par\trivlist\item[\hskip\labelsep\bfseries #1:]}{\endtrivlist}
",
    )],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(!xml.contains("trivlist"), "{xml}");
  assert_eq!(xml.matches("<proof ").count(), 1, "{xml}");
  latexml::util::test::assert_element(
    &xml,
    "proof",
    &[],
    r#"<proof xml:id="id1"><title class="ltx_runin" font="italic">Proof of the claim.</title><para xml:id="p1"><p xml:id="p1.1"><text xml:id="p1.1.1">Because.∎</text></p></para></proof>"#,
  );
}

/// The document's own `\renewenvironment{proof}` stays its own, as in LaTeX and Perl (an author `{proof}` in the
/// paper.cls papers).
#[test]
fn document_renewed_proof_is_honoured() {
  let (log, xml) = latexml::util::test::convert_with(
    r"\documentclass{article}
\usepackage{amsthm}
\renewenvironment{proof}{\par\noindent\textbf{My proof.} }{\par}
\begin{document}
\begin{proof}Because.\end{proof}
\end{document}
",
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(!xml.contains("<proof"), "{xml}");
  latexml::util::test::assert_element(
    &xml,
    "p",
    &[],
    r#"<p xml:id="p1.1"><text font="bold" xml:id="p1.1.1">My proof.</text> Because.</p>"#,
  );
}

/// `\newtheorem{thm}[\thmcnter]{Theorem}` with `\def\thmcnter{equation}` shares the equation counter (ccthm.sty,
/// 1507.02955): the RED of 67p, now GREEN.
#[test]
fn newtheorem_shared_counter_named_by_a_macro() {
  let (log, xml) = latexml::util::test::convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/package-bindings/newtheorem_shared_counter_named_by_a_macro.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  latexml::util::test::assert_element(
    &xml,
    "theorem",
    &[],
    r#"<theorem class="ltx_theorem_thm" inlist="thm theorem:thm" xml:id="S0.E2">
        <tags>
          <tag>Theorem 2</tag>
          <tag role="refnum">2</tag>
          <tag role="typerefnum">Theorem 2</tag>
        </tags>
        <title class="ltx_runin"><tag><text font="bold" xml:id="S0.E2.1">Theorem 2</text></tag></title>
        <para xml:id="S0.E2.p1">
          <p xml:id="S0.E2.p1.1"><text font="italic" xml:id="S0.E2.p1.1.1">A claim.</text></p>
        </para>
      </theorem>"#,
  );
}

/// smfthm.sty:52-56's shape: `\gdef\smf@thm{smfthm}` names the counter every theorem shares, so the theorems number
/// 1, 2 and no `\the\smf@thm` is undefined (1711.11547, 1902.02049, 2108.12000). A `[within]` named by a macro
/// resets the counter by the section's, as latex.ltx's `\@addtoreset` reads it through `\csname`.
#[test]
fn newtheorem_counter_and_within_named_by_macros() {
  let (log, xml) = latexml::util::test::convert_with(
    r"\documentclass{article}
\makeatletter
\gdef\smf@thm{smfthm}
\newcounter{smfthm}
\newtheorem{theo}[\smf@thm]{Theorem}
\newtheorem{prop}[\smf@thm]{Proposition}
\def\thm@within{section}
\newtheorem{lem}{Lemma}[\thm@within]
\makeatother
\begin{document}
\begin{theo}One.\end{theo}
\begin{prop}Two.\end{prop}
\section{A}
\begin{lem}Three.\end{lem}
\end{document}
",
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let refnums: Vec<&str> = xml
    .split(r#"<tag role="refnum">"#)
    .skip(1)
    .map(|s| s.split('<').next().unwrap_or_default())
    .collect();
  assert_eq!(refnums, ["1", "2", "1", "1.1"], "{xml}");
  latexml::util::test::assert_element(
    &xml,
    "theorem",
    &["ltx_theorem_lem"],
    r#"<theorem class="ltx_theorem_lem" inlist="thm theorem:lem" xml:id="S1.Thmlem1"><tags><tag>Lemma 1.1</tag><tag role="refnum">1.1</tag><tag role="typerefnum">Lemma 1.1</tag></tags><title class="ltx_runin"><tag><text font="bold" xml:id="S1.Thmlem1.1">Lemma 1.1</text></tag></title><para xml:id="S1.Thmlem1.p1"><p xml:id="S1.Thmlem1.p1.1"><text font="italic" xml:id="S1.Thmlem1.p1.1.1">Three.</text></p></para></theorem>"#,
  );
}

/// apxproof wraps amsthm's proof (apxproof.sty:136-137 `\let\axp@oldproof\proof`, :393 `\def\proof`), whose
/// parameterless wrappers the lock lets through (it holds only against copies of amsthm's interface, 67ps): a
/// `theoremrep`'s proof goes to the appendix after its restated theorem, as pdflatex prints it, and every other proof
/// stays in place (2502.18805).
#[test]
fn apxproof_wraps_the_locked_proof() {
  let (log, xml) = latexml::util::test::convert_with(
    r"\documentclass{article}
\usepackage{amsthm}
\usepackage{apxproof}
\newtheoremrep{theorem}{Theorem}
\begin{document}
\section{Main}
\begin{theoremrep}A claim.\end{theoremrep}
\begin{proof}
Because.
\end{proof}
\begin{proof}
An inline proof.
\end{proof}
\end{document}
",
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(xml.matches("<proof ").count(), 2, "{xml}");
  let inline = xml.find("An inline proof.").expect("inline proof");
  let restated = xml
    .find("ltx_theorem_axptheoremrp")
    .expect("restated theorem");
  let moved = xml.find("Because.").expect("appendix proof");
  assert!(inline < restated && restated < moved, "{xml}");
  latexml::util::test::assert_element(
    &xml,
    "proof",
    &["xml:id=\"S1.2\""],
    r#"<proof xml:id="S1.2"><title class="ltx_runin" font="italic">Proof.</title><para xml:id="S1.p2"><p xml:id="S1.p2.1"><text xml:id="S1.p2.1.1">Because. ∎</text></p></para></proof>"#,
  );
}
