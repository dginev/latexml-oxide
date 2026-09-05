//! dhucs.sty — kotex's Unicode Hangul support.
//!
//! The real package is loaded. dhucs.sty:44 `\ifx 가가\else…\RequirePackage
//! {kotexutf}\expandafter\endinput\fi` is a byte-vs-native-Unicode engine probe:
//! pdfTeX sees two different UTF-8 bytes (false → the kotexutf byte path,
//! kotexutf-core.tex:177 `\def\dhucs@hu{\z@}`), while a Unicode-native reader
//! (this engine, Perl alike) sees one character twice (true → the native
//! branch). In that branch `\dhucs@hu`, `\setInterHangulSkip`, `\jong`/
//! `\jung`/`\rieul` and the `\disablehangul*` switches are defined only under
//! the LuaTeX (dhucs.sty:58-75, `\csname directlua\endcsname`) or XeTeX
//! (:78-93, `\csname XeTeXrevision\endcsname`) probes — neither of which may be
//! satisfied here — so memhangul-ucs.sty:90 `\memh@hu=\dhucs@hu`,
//! oblivoir-utf.cls:271, kosections-utf.sty:53 `\hskip\dhucs@hu` fail
//! (10 kotex/oblivoir manuals; SHARED with Perl, pdflatex clean). Supply the
//! branch's engine-neutral subset (dhucs.sty:69-75 minus the luatexko
//! attribute calls) when the raw load left them undefined. Guard:
//! `perfect_kernel_batch56::dhucs_native_branch_defines_the_hangul_skip`.
use latexml_package::prelude::*;

LoadDefinitions!({
  InputDefinitions!("dhucs", noltxml => true, extension => Some(Cow::Borrowed("sty")));
  RawTeX!(
    r"\@ifundefined{dhucs@hu}{\let\dhucs@hu\z@}{}
\@ifundefined{setInterHangulSkip}{\let\setInterHangulSkip\@gobble}{}
\@ifundefined{jong}{\let\jong\relax}{}
\@ifundefined{jung}{\let\jung\relax}{}
\@ifundefined{rieul}{\let\rieul\relax}{}
\@ifundefined{disablehangulfontspec}{\def\disablehangulfontspec{}}{}
\@ifundefined{disablehangullinebreak}{\def\disablehangullinebreak{}}{}
\@ifundefined{pdfstringdefPreHook}{\let\pdfstringdefPreHook\@empty}{}
\@ifundefined{dhucs@emph@raise}{\newdimen\dhucs@emph@raise}{}"
  );
  // The last two lines are dhucs.sty:116-117's `\if@hangul` block (and
  // kotexutf.sty:515-516, the byte-engine route pdflatex takes after the
  // `\ifx가가` probe at dhucs.sty:44 endinputs dhucs): a Unicode-native reader
  // stays in dhucs.sty with `@hangul` false, so memhangul-ucs.sty:509
  // `\g@addto@macro\pdfstringdefPreHook` and :451 `\raise\dhucs@emph@raise`
  // read undefined names (istgame-doc, obchapterstyles-doc, obsideparas,
  // oblivoir-simpledoc, kotex-doc, kotex-utf-doc). Guard:
  // `perfect_kernel_batch56::dhucs_native_branch_defines_pdfstringdefprehook`.
});
