//! backref.sty — back-references from bibliography to citations
//! (part of the Hyperref Bundle).
//!
//! backref adds a "Cited on pages X, Y" link at the end of each
//! bibliography entry, pointing back to the citation locations.
//! It does this by redefining `\@bibitem` / `\@lbibitem` to wrap
//! the bibitem body with a `\BR@bibitem` hook.
//!
//! Raw-loading backref.sty into our engine is fragile: backref's
//! redefinitions of `\@bibitem`/`\@lbibitem` chain with paper-
//! bundled bibliography helpers (e.g. myharvard's `\harvarditem`
//! which writes to `.aux` then calls `\item[]`) — the resulting
//! expansion loops indefinitely (witness 1107.0498: amsart +
//! `\usepackage[hyperpageref]{backref}` + paper-local myharvard.sty
//! → Convert TIMEOUT in `.bbl` processing; Perl handles the same
//! input with the same 4 undefined-macro errors but completes).
//!
//! Perl LaTeXML has no backref binding either; with default
//! `INCLUDE_STYLES=false`, raw backref.sty is not loaded — Perl
//! emits "missing binding" and continues without back-references.
//!
//! Match Perl: stub the user-facing API as no-ops. We lose the
//! back-reference links, but the bibliography itself renders
//! correctly.
use latexml_package::prelude::*;

LoadDefinitions!({
  // backref.sty:196 reads its options with `\ProcessKeyvalOptions`, which marks them processed: `\@curroptions` stays as it was.
  key_options_processed()?;
  // Loaded through hyperref's `backref`/`pagebackref` option (hyperref.sty:4587-4589, which the hyperref binding
  // flags), the missing back-reference lists are generated navigation the author did not write, and Perl ignores
  // the option silently: an Info. A document's own `\usepackage{backref}` keeps the Warning (OXIDIZED_DESIGN_DIVERGENCES
  // #547; witness 2203.07162).
  let message = "backref.sty is minimally stubbed — back-references from bibliography to citations are not rendered.";
  if lookup_bool("BACKREF_LOADED_BY_HYPERREF") {
    Info!("missing_file", "backref.sty", message);
  } else {
    Warn!("missing_file", "backref.sty", message);
  }
  // backref.sty:61-62 and :483 load kvoptions, kvsetkeys and rerunfilecheck.
  RequirePackage!("kvoptions");
  RequirePackage!("kvsetkeys");
  // rerunfilecheck has no binding: read raw only where raw loading is on (see mdframed_sty.rs).
  if lookup_bool("INCLUDE_STYLES") {
    RequirePackage!("rerunfilecheck");
  }
  // backref's user-facing API: setup, format, print. backref.sty:67 `\backrefsetup` = `\kvsetkeys{backref}`.
  DefMacro!("\\backrefsetup", "\\kvsetkeys{backref}");
  DefMacro!("\\backrefprint", "");
  // backref.sty:95-148 the language hooks, which set the words of the back-reference line (documents
  // `\renewcommand` them, `\backrefsep` and the like); :195 English is the default. :207-208 the
  // `\ifbackrefparscan` switch a bibliography sets (witness 1809.07574: `\backrefparscanfalse` in
  // `{thebibliography}`, with `[pagebackref]{hyperref}` loading backref). :78-92 `\backref` by the options
  // `pageref` (the default, :195), `ref`, `hyperref` and `hyperpageref` (the two hyperref's options pass), other
  // options and `\backrefsetup` keys accepted and dropped; :270-277 `\backrefalt`/`\backrefentrycount`, which
  // documents redefine to format the line; the line itself is not rendered, so they are never called here.
  RawTeX!(
    r"\def\backrefenglish{\def\backrefpagesname{pages}\def\backrefsectionsname{sections}\def\backrefsep{, }%
\def\backreftwosep{ and~}\def\backreflastsep{, and~}}
\def\backrefgerman{\def\backrefpagesname{Seiten}\def\backrefsectionsname{Abschnitte}\def\backrefsep{, }%
\def\backreftwosep{ und~}\def\backreflastsep{ und~}}
\def\backreffrench{\def\backrefpagesname{pages}\def\backrefsectionsname{sections}\def\backrefsep{, }%
\def\backreftwosep{ et~}\def\backreflastsep{ et~}}
\def\backrefspanish{\def\backrefpagesname{p\'aginas}\def\backrefsectionsname{secciones}\def\backrefsep{, }%
\def\backreftwosep{ y~}\def\backreflastsep{ y~}}
\def\backrefbrazil{\def\backrefpagesname{p\'aginas}\def\backrefsectionsname{se\c c\~oes}\def\backrefsep{, }%
\def\backreftwosep{ e~}\def\backreflastsep{ e~}}
\def\backrefafrikaans{\def\backrefpagesname{bladsye}\def\backrefsectionsname{afdelings}\def\backrefsep{, }%
\def\backreftwosep{ en~}\def\backreflastsep{ en~}}
\def\backrefitalian{\def\backrefpagesname{pagine}\def\backrefsectionsname{sezioni}\def\backrefsep{, }%
\def\backreftwosep{ e~}\def\backreflastsep{ e~}}
\backrefenglish
\SetupKeyvalOptions{family=backref,prefix=BR@}
\DeclareVoidOption{pageref}{\def\backref{\backrefpagesname\ }}
\DeclareVoidOption{ref}{\def\backref{\backrefsectionsname\ }}
\DeclareVoidOption{hyperref}{\def\backref{}}
\DeclareVoidOption{hyperpageref}{\def\backref{}}
\kv@set@family@handler{backref}{}
\DeclareDefaultOption{}
\backrefsetup{pageref}
\ProcessKeyvalOptions*\relax
\newif\ifbackrefparscan\backrefparscantrue
\@ifundefined{backrefalt}{\def\backrefalt#1#2#3#4{}}{}
\@ifundefined{backrefentrycount}{\def\backrefentrycount#1#2{#1}}{}"
  );
});
