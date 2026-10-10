use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: elsart.cls.ltxml

  // Generally ignorable options
  for option in [
    "12pt", "11pt", "10pt", "oneside", "twoside",
    "symbold", "nameyear", "doublespacing", "reviewcopy",
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
});
