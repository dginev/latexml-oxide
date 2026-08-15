use crate::prelude::*;

// pdfcol.sty — pdfTeX colour-stack manager (Heiko Oberdiek / LaTeX team). It is
// pulled in transitively by tcolorbox's `breakable` library to keep box colours
// consistent across a page break, and defines a handful of `\pdfcol…` commands
// that ultimately drive pdfTeX's `\pdfcolorstack` primitive (issue #531).
//
// A PDF colour stack is a page-model concept with NO rendering in LaTeXML's
// HTML/XML output, and pdfTeX's primitives are unavailable here — exactly the
// situation the real package guards with its `\ifpdfcolAvailable … \else …`
// switch. Perl LaTeXML ships no pdfcol.sty.ltxml either, so both engines raised
// `undefined:\pdfcolInitStack` on breakable tcolorboxes (SHARED-FAILURE). This
// binding ports pdfcol.sty's OWN "disabled" fallback branch (pdfcol.sty L275-291,
// TeX Live 2025): every command becomes a no-op and `\pdfcolIfStackExists` always
// takes its false branch (no stack is ever registered). OXIDIZED_DESIGN #112.
#[rustfmt::skip]
LoadDefinitions!({
  // \def\pdfcolInitStack#1{\PDFCOL@Disabled}  — a stack is never created.
  DefMacro!("\\pdfcolInitStack{}", "");
  // \long\def\pdfcolIfStackExists#1#2#3{#3} — the stack never exists → false arg.
  DefMacro!("\\pdfcolIfStackExists{}{}{}", "#3");
  // \def\pdfcolSwitchStack#1{\PDFCOL@Disabled}
  DefMacro!("\\pdfcolSwitchStack{}", "");
  // \def\pdfcolSetCurrentColor{\PDFCOL@Disabled} — no argument.
  DefMacro!("\\pdfcolSetCurrentColor", "");
  // \def\pdfcolSetCurrent#1{\PDFCOL@Disabled}
  DefMacro!("\\pdfcolSetCurrent{}", "");
  // \pdfcolErrorNoStacks raises a package error in the real disabled branch; in
  // HTML the whole feature is legitimately unavailable, so silence it (\relax)
  // rather than surface an error for a PDF-only capability that has no output.
  Let!("\\pdfcolErrorNoStacks", "\\relax");
});
