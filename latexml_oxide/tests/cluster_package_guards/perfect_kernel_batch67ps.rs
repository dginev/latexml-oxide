//! Red/green guards for batch 67ps (the integrated 67p + 67s round): amsthm's proof under theorem.sty's borrow and
//! under a real amsthm.
//! - What a binding borrows (`require_package_borrowed`: theorem.sty's and ntheorem.sty's bindings borrow amsthm)
//!   carries `DefinitionOrigin::Borrowed` and yields to any later definition, the document's or any file's; the
//!   borrow sets no proof lock and leaves what it loaded (amsgen) unloaded.
//! - Under a real amsthm the proof lock holds only against a copy of amsthm's interface (an optional argument with a
//!   default first, `[1][\proofname]`), titles the kept proof from the copy's own default, and lapses once the
//!   document has replaced or undefined the proof; an empty default (`[1][]`, 0705.0545's nccthm) is not a copy.
//! - amsthm's reload after the borrow keeps a proof a file put in place and theorem.sty's font commands.
//!
//! - A borrowing binding claims the borrowed names its real package defines too (ntheorem's `[amsthm]` proof), and a
//!   robust copy is judged by its inner `\cs␣` (round 4).
//!
//! Repros `tools/perfect_kernel/repros/package-bindings/{theorem_borrowed_proof_yields_to_a_class_def,
//! theorem_borrowed_proof_yields_to_a_package_newenvironment,amsthm_proof_lock_lets_a_hardcoded_head_through,
//! amsthm_proof_copy_titled_by_its_default,amsthm_proof_lock_lapses_after_the_document_undefines_it,
//! amsthm_proof_lock_passes_an_empty_default,ntheorem_amsthm_option_keeps_its_proof,
//! amsthm_proof_lock_holds_against_a_robust_copy,theorem_borrow_leaves_amsgen_unloaded,
//! amsthm_reload_keeps_theorem_font_commands}.tex`.
use latexml::util::test::assert_element;

use super::perfect_kernel_batch46::{convert_files_with, convert_with, error_count, warning_count};

/// The XML of `tex` converted under ar5iv (with `files` beside it), which must give no error and no warning.
fn convert_clean(tex: &str, files: &[(&str, &str)]) -> String {
  let (log, xml) = if files.is_empty() {
    convert_with(tex, Some("ar5iv.sty"))
  } else {
    convert_files_with(tex, files, Some("ar5iv.sty"))
  };
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  xml
}

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/package-bindings/",
      $name
    ))
  };
}

/// A class built on theorem.sty `\def`s its own proof: the borrow set no lock, so the class's head and end mark stay
/// (pdflatex "Beweis. Body ten. □"; a03442572a: the binding's `<proof>` "Proof." with ∎).
#[test]
fn borrowed_proof_yields_to_a_class_def() {
  let xml = convert_clean(
    repro!("theorem_borrowed_proof_yields_to_a_class_def.tex"),
    &[],
  );
  assert!(!xml.contains("<proof"), "{xml}");
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    r#"<p xml:id="p1.1"><text font="bold" xml:id="p1.1.1">Beweis.</text> Body ten. <Math mode="inline" tex="\Box" text="Box" xml:id="p1.m1"><XMath xml:id="p1.m1.1"><XMTok name="Box" role="UNKNOWN">□</XMTok></XMath></Math></p>"#,
  );
}

/// A package's `\newenvironment{proof}` after theorem.sty defines it, as in LaTeX, where amsthm's proof does not
/// exist (cc.cls's ccproof.sty:56, 1507.02955; pdflatex "MyProof. Body one.").
#[test]
fn borrowed_proof_yields_to_a_package_newenvironment() {
  let xml = convert_clean(
    repro!("theorem_borrowed_proof_yields_to_a_package_newenvironment.tex"),
    &[],
  );
  assert!(!xml.contains("<proof"), "{xml}");
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    r#"<p xml:id="p1.1"><text font="bold" xml:id="p1.1.1">MyProof.</text> Body one.</p>"#,
  );
}

/// A package's `\renewenvironment{proof}` after theorem.sty is its own (pdflatex "Beweis. Body four.": the package's
/// trivlist head). pdflatex also reports `! LaTeX Error: Environment proof undefined` there, theorem.sty having no
/// proof; we give no diagnostic, the borrowed proof standing in for the undefined one (KNOWN_PERL_ERRORS #686).
#[test]
fn borrowed_proof_yields_to_a_package_renewenvironment() {
  let xml = convert_clean(
    r"\documentclass{article}
\usepackage{theorem}
\usepackage{renproofpkg}
\begin{document}
\begin{proof}Body four.\end{proof}
\end{document}
",
    &[(
      "renproofpkg.sty",
      r"\renewenvironment{proof}[1][\proofname]{\par\trivlist\item[\hskip\labelsep\itshape #1.]}{\endtrivlist}
\def\proofname{Beweis}
",
    )],
  );
  assert!(!xml.contains("<proof"), "{xml}");
  assert_element(
    &xml,
    "item",
    &[],
    r#"<item xml:id="S0.I1.ix1"><tags><tag><text font="italic" xml:id="S0.I1.ix1.1">Beweis.</text></tag></tags><para xml:id="S0.I1.ix1.p1"><p xml:id="S0.I1.ix1.p1.1">Body four.</p></para></item>"#,
  );
}

/// A document's own `{proof}` in the `\@ifpackageloaded{amsthm}` false branch after theorem.sty (pdflatex "DocProof.
/// Body two."; before: the borrowed proof, "Proof." with ∎).
#[test]
fn document_proof_after_theorem_is_its_own() {
  let xml = convert_clean(
    r"\documentclass{article}
\usepackage{theorem}
\makeatletter
\@ifpackageloaded{amsthm}{}{\newenvironment{proof}{\par\textbf{DocProof.} }{\par}}
\makeatother
\begin{document}
\begin{proof}Body two.\end{proof}
\end{document}
",
    &[],
  );
  assert!(!xml.contains("<proof"), "{xml}");
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    r#"<p xml:id="p1.1"><text font="bold" xml:id="p1.1.1">DocProof.</text> Body two.</p>"#,
  );
}

/// theorem.sty, then a real amsthm, then a package's amsthm copy (`[1][\proofname]`): the lock holds, titled by the
/// package's `\proofname` (pdflatex "Beweis. Body five."; the binding's ∎ is the end mark amsthm's QED would print).
#[test]
fn theorem_then_amsthm_locks_a_copy() {
  let xml = convert_clean(
    r"\documentclass{article}
\usepackage{theorem}
\let\theoremstyle\undefined\let\newtheoremstyle\undefined
\usepackage{amsthm}
\usepackage{renproofpkg}
\newtheorem{thm}{Theorem}
\begin{document}
\begin{thm}T.\end{thm}
\begin{proof}Body five.\end{proof}
\end{document}
",
    &[(
      "renproofpkg.sty",
      r"\renewenvironment{proof}[1][\proofname]{\par\trivlist\item[\hskip\labelsep\itshape #1.]}{\endtrivlist}
\def\proofname{Beweis}
",
    )],
  );
  assert!(!xml.contains("trivlist"), "{xml}");
  assert_eq!(xml.matches("<proof ").count(), 1, "{xml}");
  assert_element(
    &xml,
    "proof",
    &[],
    r#"<proof xml:id="id1"><title class="ltx_runin" font="italic">Beweis.</title><para xml:id="p1"><p xml:id="p1.1"><text xml:id="p1.1.1">Body five.∎</text></p></para></proof>"#,
  );
}

/// theorem.sty, then apxproof (whose raw file loads amsthm for real and wraps its proof): the theoremrep's proof
/// follows the restated theorem in the appendix, the inline proof stays (pdflatex t3: "Proof. Inline proof body." in
/// the body, "Proof. Appendix proof body." after the restated theorem).
#[test]
fn theorem_then_apxproof_wraps_the_proof() {
  let xml = convert_clean(
    r"\documentclass{article}
\usepackage{theorem}
\let\theoremstyle\undefined
\usepackage[appendix=append]{apxproof}
\newtheoremrep{theorem}{Theorem}
\begin{document}
\section{Main}
\begin{theoremrep}
Statement A.
\end{theoremrep}
\begin{proof}
Appendix proof body.
\end{proof}
\begin{proof}
Inline proof body.
\end{proof}
\end{document}
",
    &[],
  );
  assert_eq!(xml.matches("<proof ").count(), 2, "{xml}");
  let inline = xml.find("Inline proof body.").expect("inline proof");
  let restated = xml
    .find("ltx_theorem_axptheoremrp")
    .expect("restated theorem");
  let moved = xml.find("Appendix proof body.").expect("appendix proof");
  assert!(inline < restated && restated < moved, "{xml}");
  assert_element(
    &xml,
    "proof",
    &["xml:id=\"S1.2\""],
    r#"<proof xml:id="S1.2"><title class="ltx_runin" font="italic">Proof.</title><para xml:id="S1.p2"><p xml:id="S1.p2.1"><text xml:id="S1.p2.1.1">Appendix proof body. ∎</text></p></para></proof>"#,
  );
}

/// A redefinition with a head of its own goes through under a real amsthm: its head text and end mark stay
/// (pdflatex "Beweisskizze: Body nine. ■"; a03442572a: "Proof." with ∎).
#[test]
fn proof_with_a_head_of_its_own_goes_through() {
  let xml = convert_clean(
    repro!("amsthm_proof_lock_lets_a_hardcoded_head_through.tex"),
    &[],
  );
  assert!(!xml.contains("<proof"), "{xml}");
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    r#"<p xml:id="p1.1"><text font="smallcaps" xml:id="p1.1.1">Beweisskizze:</text> Body nine. <Math mode="inline" tex="\blacksquare" text="blacksquare" xml:id="p1.m1"><XMath xml:id="p1.m1.1"><XMTok name="blacksquare" role="UNKNOWN">■</XMTok></XMath></Math></p>"#,
  );
}

/// An amsthm copy whose default is not `\proofname` (thuthesis.cls:4052 `[1][\thu@proof@name]`) is kept as the
/// binding's proof, titled by that default (pdflatex "Nachweis: Body twelve." / "Sketch: Body twelve b.").
#[test]
fn proof_copy_titled_by_its_own_default() {
  let xml = convert_clean(repro!("amsthm_proof_copy_titled_by_its_default.tex"), &[]);
  assert_eq!(xml.matches("<proof ").count(), 2, "{xml}");
  assert_element(
    &xml,
    "proof",
    &["xml:id=\"id1\""],
    r#"<proof xml:id="id1"><title class="ltx_runin" font="italic">Nachweis.</title><para xml:id="p1"><p xml:id="p1.1"><text xml:id="p1.1.1">Body twelve.∎</text></p></para></proof>"#,
  );
  assert_element(
    &xml,
    "proof",
    &["xml:id=\"id2\""],
    r#"<proof xml:id="id2"><title class="ltx_runin" font="italic">Sketch.</title><para xml:id="p2"><p xml:id="p2.1"><text xml:id="p2.1.1">Body twelve b.∎</text></p></para></proof>"#,
  );
}

/// The document's `\let\proof\undefined\let\endproof\undefined` after amsart lapses the lock, so a package's
/// copy-shaped `\newenvironment{proof}[1][Nachweis]` defines it, as in LaTeX (pdflatex "Nachweis. Body seven.";
/// a03442572a: Error:undefined:\proof). Only the lapse lets it through: its default is not empty.
#[test]
fn proof_lock_lapses_once_the_document_undefines_it() {
  let xml = convert_clean(
    repro!("amsthm_proof_lock_lapses_after_the_document_undefines_it.tex"),
    &[],
  );
  assert!(!xml.contains("<proof"), "{xml}");
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    r#"<p xml:id="p1.1"><text font="bold" xml:id="p1.1.1">Nachweis.</text> Body seven.</p>"#,
  );
}

/// A redefinition whose optional argument defaults to nothing (nccthm.sty:173 `\newcommand*{\proof}[1][]`) is not a
/// copy of amsthm's proof, so the lock lets it through (0705.0545, after the document's `\let\proof\undefined`;
/// pdflatex "NccProof. Body eleven."; a03442572a: Error:undefined:\proof).
#[test]
fn proof_lock_passes_an_empty_default() {
  let xml = convert_clean(repro!("amsthm_proof_lock_passes_an_empty_default.tex"), &[]);
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    r#"<p xml:id="p1.1"><text font="bold" xml:id="p1.1.1">NccProof.</text> Body eleven.</p>"#,
  );
}

/// ntheorem's `amsthm` option defines `{proof}` (ntheorem.sty:668-711), which the binding takes from the borrowed
/// amsthm: it is ntheorem's own, so a later package's `\newenvironment{proof}` is refused (pdflatex "! LaTeX Error:
/// Command \proof already defined." and "Proof. Body nineteen."; we report the refusal as Perl does, `Info:ignore`;
/// e6b39e586b: "LaterHead. Body nineteen.").
#[test]
fn ntheorem_amsthm_option_keeps_its_proof() {
  let (log, xml) = convert_with(
    repro!("ntheorem_amsthm_option_keeps_its_proof.tex"),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(log.matches("Info:ignore:proof ").count(), 1, "{log}");
  assert!(!xml.contains("LaterHead"), "{xml}");
  assert_eq!(xml.matches("<proof ").count(), 1, "{xml}");
  assert_element(
    &xml,
    "proof",
    &[],
    r#"<proof xml:id="id1"><title class="ltx_runin" font="italic">Proof.</title><para xml:id="p1"><p xml:id="p1.1"><text xml:id="p1.1.1">Body nineteen.<text class="ltx_align_floatright" xml:id="p1.1.1.1"/></text></p></para></proof>"#,
  );
}

/// A robust copy of amsthm's proof (`\DeclareRobustCommand\proof[1][\proofname]`) is judged by its inner `\proof␣`,
/// as `owns_calls` judges a robust binding: the lock holds, and the binding's proof stays (pdflatex "Proof. Body
/// fourteen."; e6b39e586b: the copy's itemize, its parameterless wrapper passing the copy rule).
#[test]
fn proof_lock_holds_against_a_robust_copy() {
  let xml = convert_clean(
    repro!("amsthm_proof_lock_holds_against_a_robust_copy.tex"),
    &[],
  );
  assert!(!xml.contains("trivlist"), "{xml}");
  assert_eq!(xml.matches("<proof ").count(), 1, "{xml}");
  assert_element(
    &xml,
    "proof",
    &[],
    r#"<proof xml:id="id1"><title class="ltx_runin" font="italic">Proof.</title><para xml:id="p1"><p xml:id="p1.1"><text xml:id="p1.1.1">Body fourteen.∎</text></p></para></proof>"#,
  );
}

/// The borrow leaves amsthm and amsgen, which amsthm loaded, unloaded (pdflatex "State: amsgen not loaded, amsthm
/// not loaded.").
#[test]
fn borrow_leaves_what_it_loaded_unloaded() {
  let xml = convert_clean(repro!("theorem_borrow_leaves_amsgen_unloaded.tex"), &[]);
  assert_element(
    &xml,
    "p",
    &[],
    r#"<p xml:id="p1.1">State: amsgen not loaded, amsthm not loaded.</p>"#,
  );
}

/// amsthm's reload after the borrow keeps theorem.sty's `\theorembodyfont`/`\theoremheaderfont` (amsthm.sty defines
/// neither), which do not reach amsthm's theorems (pdflatex: CMBX10 head, CMTI10 body).
#[test]
fn amsthm_reload_keeps_theorem_font_commands() {
  let xml = convert_clean(repro!("amsthm_reload_keeps_theorem_font_commands.tex"), &[]);
  assert_element(
    &xml,
    "theorem",
    &[],
    r#"<theorem class="ltx_theorem_thm" inlist="thm theorem:thm" xml:id="Thmthm1"><tags><tag>Theorem 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Theorem 1</tag></tags><title class="ltx_runin"><tag><text font="bold" xml:id="Thmthm1.1">Theorem 1</text></tag><text font="bold" xml:id="Thmthm1.2">.</text></title><para xml:id="Thmthm1.p1"><p xml:id="Thmthm1.p1.1"><text font="italic" xml:id="Thmthm1.p1.1.1">Body eight.</text></p></para></theorem>"#,
  );
}

/// amsthm's real load after the borrow keeps a proof a file has put in place of the borrowed one, as
/// amsthm.sty:431's `\newenvironment{proof}` fails over it (pdflatex prints the file's "Hard proof." head; it reports
/// "Command \proof already defined" there, and its `\newenvironment` still sets `\endproof`, amsthm's).
#[test]
fn amsthm_reload_keeps_a_files_proof() {
  let xml = convert_clean(
    r"\documentclass{article}
\usepackage{theorem}
\usepackage{hardproofpkg}
\let\theoremstyle\undefined\let\newtheoremstyle\undefined
\usepackage{amsthm}
\begin{document}
\begin{proof}Body fifteen.\end{proof}
\end{document}
",
    &[(
      "hardproofpkg.sty",
      r"\def\proof{\par\noindent\textit{Hard proof.}\ }
\def\endproof{\par}
",
    )],
  );
  assert!(!xml.contains("<proof"), "{xml}");
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    r#"<p xml:id="p1.1"><text font="italic" xml:id="p1.1.1">Hard proof.</text> Body fifteen.</p>"#,
  );
}

/// llncs's own proof, which apxproof.sty:39-46 keeps aside before loading amsthm and puts back after it, is not the
/// meaning the lock protects (amsthm's `\@proof`): its `\endproof` comes back with it, and the appendix proof follows
/// the restated theorem (2311.01302; pdflatex "Proof. An inline proof." in the body, "Proof. Because." in the
/// appendix; an end half that followed any binding's proof kept amsthm's `\endproof`: 24 `\endgroup` errors).
#[test]
fn llncs_then_apxproof_keeps_its_proof() {
  let xml = convert_clean(
    r"\documentclass{llncs}
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
    &[],
  );
  let inline = xml.find("An inline proof.").expect("inline proof");
  let restated = xml
    .find("ltx_theorem_axptheoremrp")
    .expect("restated theorem");
  let moved = xml.find("Because.").expect("appendix proof");
  assert!(inline < restated && restated < moved, "{xml}");
  assert_element(
    &xml,
    "theorem",
    &["xml:id=\"Thmproofx2\""],
    r#"<theorem class="ltx_theorem_proof" inlist="thm theorem:proof" xml:id="Thmproofx2"><tags><tag>Proof</tag><tag role="typerefnum">Proof</tag></tags><title class="ltx_runin"><tag><text font="bold" xml:id="Thmproofx2.1">Proof</text></tag></title><para xml:id="Thmproofx2.p1"><p xml:id="Thmproofx2.p1.1">Because.</p></para></theorem>"#,
  );
}
