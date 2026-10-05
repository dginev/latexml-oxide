//! tocloft.sty — the lists of contents, figures and tables, laid out. Interpreted raw, for its `\cft…` parameters and
//! commands that classes and documents set, with the kernel's own lists put back where tocloft puts its
//! `\tableofcontents`, `\listoffigures` and `\listoftables`: those run `\@starttoc`, which reads a `.toc` LaTeXML never
//! writes, so only their heading was left and the `<TOC>` was lost (Perl has no binding and the same loss under
//! `rawstyles`; SciPost.cls loads it: 1811.09408, 2105.01655, 2203.11601).
use latexml_package::prelude::*;

LoadDefinitions!({
  InputDefinitions!("tocloft", noltxml => true, extension => Some(Cow::Borrowed("sty")));
  // Under tocloft's own condition (`titles` keeps the class's lists) and at its own time: tocloft replaces all three in
  // begin-document hooks (tocloft.sty:118-140, 536-538, 638-640), and this one, registered after them, runs right after
  // them and before any hook a class or document adds later, as in TeX.
  // Only when tocloft went that far: it stops early (`\@cftquit`, tocloft.sty:43-52) under a class with neither
  // `\chapter` nor `\section`, registering none of its hooks; `\cftparskip` comes after that point.
  Digest!(
    r"\@ifundefined{cftparskip}{}{\if@cftnctoc\else\AtBeginDocument{\let\tableofcontents\lx@tableofcontents
    \let\listoffigures\lx@listoffigures\let\listoftables\lx@listoftables}\fi}"
  )?;
});
