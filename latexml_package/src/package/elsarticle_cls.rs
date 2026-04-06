use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: elsarticle.cls.ltxml
  // Generally ignorable options
  for option in ["preprint", "final", "review", "5p", "3p", "1p",
    "12pt", "11pt", "10pt", "endfloat", "endfloats", "numafflabel",
    "doubleblind", "oneside", "twoside", "onecolumn", "twocolumn",
    "longtitle", "lefttitle", "centertitle", "reversenotenum",
    "nopreprintline", "symbold", "ussrhead", "nameyear",
    "doublespacing", "reviewcopy"].iter()
  {
    DeclareOption!(*option, None);
  }
  // Options that load packages or set values
  DeclareOption!("times", None);
  DeclareOption!("seceqn", None);
  DeclareOption!("secthm", None);
  DeclareOption!("amsthm", None);
  DeclareOption!("authoryear", None);
  DeclareOption!("number", None);
  DeclareOption!("numbers", None);
  // Pass other options to article
  DeclareOption!(None, {
    Digest!("\\PassOptionsToClass{\\CurrentOption}{article}")?;
  });
  ProcessOptions!();
  LoadClass!("article");
  RequirePackage!("elsart_support_core");
  RequirePackage!("fleqn");
  RequirePackage!("graphicx");
  RequirePackage!("pifont");
  RequirePackage!("natbib");
  RequirePackage!("hyperref");
  DefMacro!("\\biboptions{}", "\\setcitestyle{#1}");
});
