//! examdesign.cls — exams with question banks, answer keys and randomised
//! versions. Loaded RAW, with one seam opened first.
//!
//! examdesign.cls:323-344 repurposes `\section`: `\def\section{\stepcounter
//! {section}\setcounter{question}{1}}` / `\def\endsection{\make@qlist}`, and
//! its question environments are `\begin{section}…\end{section}` wrappers
//! (examdesign.cls:802-812 `matching`, `fillin`, `truefalse`, `shortanswer`,
//! `multiplechoice`). The kernel `\section` is `locked` (latex_constructs.rs
//! `DefMacro!("\\section", …, locked=>true)`, Perl latex_constructs.pool:559),
//! so both engines refused the class's definition and ran `\@startsection`
//! on an environment body — `Expected opening '{'`, `\lx@tag Attempt to end
//! mode restricted_horizontal`, then `\endgroup` errors at every `\end` (Perl
//! 67 errors, Rust 100+ → Fatal on examplea/exampleb/examplec). pdflatex is
//! clean because the class owns `\section`. Same precedent as the `\chapter`
//! unlock for source3body (latex_constructs.rs, KNOWN_PERL_ERRORS #141).
//! Guard: `perfect_kernel_batch54::examdesign_owns_section_as_an_environment`.

use latexml_package::prelude::*;

LoadDefinitions!({
  assign_value("\\section:locked", false, Some(Scope::Global));
  InputDefinitions!("examdesign", noltxml => true, extension => Some(Cow::Borrowed("cls")));
});
