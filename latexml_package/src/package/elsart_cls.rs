use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: elsart.cls.ltxml

  // Generally ignorable options
  for option in [
    "12pt", "11pt", "10pt", "oneside", "twoside",
    "doublespacing", "reviewcopy",
  ].iter() {
    DeclareOption!(option, None);
  }
  // elsart.cls:35, :50, :90 (v2.18, 2001/01/05, the copy 2010.00445 and cond-mat0406419 ship) — internals the classes
  // built on it use (the esl.dtx wrappers phbauth, phb-proc4-auth, LT23auth, PHYEAUTH, cpcauth… `\LoadClass{elsart}`
  // and test `\if@ussrhead`, `\if@TwoColumn`, `\@frontmatterwidth`; 367 of run 336's papers, read raw since 62k;
  // KNOWN_PERL_ERRORS #561), and its options that set them (:47-51).
  RawTeX!(r"\newif\if@TwoColumn\newif\if@ussrhead\@ussrheadfalse\newdimen\@frontmatterwidth");
  DeclareOption!("onecolumn", "\\@twocolumnfalse\\@TwoColumnfalse");
  DeclareOption!("twocolumn", "\\@twocolumntrue\\@TwoColumntrue");
  DeclareOption!("ussrhead", "\\@ussrheadtrue");
  // elsart.cls:36, :66-67, :625-626 (v2.20, 0712.3870's copy; 91 of the shipped copies alike): the reference-style
  // switches a journal style built on elsart tests in its own `{thebibliography}` (0907.1629, 0905.2086 adnd.sty:273-301
  // `\if@natbibloaded`; run-347: 4 papers), with the `nameyear` option that sets one.
  RawTeX!(r"\newif\if@nameyear\@nameyearfalse
\newif\if@natbibloaded\@natbibloadedfalse
\AtBeginDocument{\@ifpackageloaded{natbib}{\@natbibloadedtrue}{}}");
  DeclareOption!("nameyear", "\\@nameyeartrue");
  // elsart.cls:59-60 (v2.18): `symbold` keeps the kernel's `\d` (elsart_support_sty.rs).
  RawTeX!(r"\newif\if@symbold\@symboldfalse");
  DeclareOption!("symbold", "\\@symboldtrue");

  DeclareOption!("seceqn", {
    AssignValue!("@seceqn" => 1i64);
  });
  DeclareOption!("secthm", {
    AssignValue!("@secthm" => 1i64);
  });
  DeclareOption!("amsthm", {
    AssignValue!("@amsthm" => 1i64);
  });

  // Anything else is for article.
  DeclareOption!(None, {
    Digest!("\\PassOptionsToClass{\\CurrentOption}{article}")?;
  });

  ProcessOptions!();
  LoadClass!("article");
  RequirePackage!("elsart_support");
  // elsart.cls:28 (v2.18, 0708.4004's copy; v2.20:27 "Elsevier", 0712.3870's): `\@journal` is the publisher until
  // `\journal` sets it (elsart_support_core_sty.rs).
  RawTeX!(r"\def\@journal{Elsevier Science}");
  // elsart.cls:239-255 (0706.3521's copy; 71 copies papers ship alike) `\appendix`: the appendix sections "A", the
  // equations, figures and tables numbered within each appendix, "A.1" (`\app@number`: zeroed, reset by each appendix
  // section); `\appendix*` the same with the heading "Appendix" alone (`\lb@section{\appendixname}`). The kernel's
  // `\appendix` (article_cls.rs `\@appendix`) runs first for the `ltx:appendix` units. Perl's elsart.cls.ltxml leaves
  // the kernel's (equation 4, figure 2): KNOWN_PERL_ERRORS #1098. Guard
  // `perfect_kernel_batch78q::appendix_counters_interact_elsart`.
  RawTeX!(r"\def\app@number#1{\setcounter{#1}{0}\@addtoreset{#1}{section}\@namedef{the#1}{\thesection.\arabic{#1}}}
\def\appendix{\@appendix\@ifstar{\appendix@star}{\appendix@nostar}}
\def\appendix@nostar{\setcounter{section}{0}\def\thesection{\Alph{section}}%
  \setcounter{subsection}{0}\setcounter{subsubsection}{0}\setcounter{paragraph}{0}%
  \app@number{equation}\app@number{figure}\app@number{table}}
\def\appendix@star{\appendix@nostar\gdef\fnum@appendix{\appendixname}}");
});
