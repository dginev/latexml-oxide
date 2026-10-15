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
  load_class("article", Vec::new(), Tokens!())?;
  RequirePackage!("elsart_support");
  // elsart.cls:28 (v2.18, 0708.4004's copy; v2.20:27 "Elsevier", 0712.3870's): `\@journal` is the publisher until
  // `\journal` sets it (elsart_support_core_sty.rs).
  RawTeX!(r"\def\@journal{Elsevier Science}");
});
