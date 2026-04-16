use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: aa.cls.ltxml — Astronomy & Astrophysics
  // Ignorable options
  for option in ["10pt", "11pt", "12pt", "twoside", "onecolumn", "twocolumn",
    "draft", "final", "referee", "longauth", "rnote", "oldversion",
    "runningheads", "envcountreset", "envcountsect",
    "structabstract", "traditabstract", "letter"].iter()
  {
    DeclareOption!(*option, None);
  }
  DeclareOption!(None, {
    Digest!("\\PassOptionsToClass{\\CurrentOption}{article}")?;
  });
  ProcessOptions!();
  LoadClass!("article");
  RequirePackage!("aa_support");
  // Raw aa.cls unconditionally loads natbib (not just for bibnumber/bibauthoryear options).
  // Many aa papers use \citep/\citet without explicit \usepackage{natbib}.
  RequirePackage!("natbib");
  // Override \pmatrix to use plain TeX version (not amsmath)
  DefMacro!("\\pmatrix{}",
    "\\lx@gen@plain@matrix{name=pmatrix,datameaning=matrix,left=\\lx@left(,right=\\lx@right)}{#1}");
  // Override \cases to use plain TeX version
  DefMacro!("\\cases{}",
    "\\lx@gen@plain@cases{meaning=cases,left=\\lx@left\\{,conditionmode=text,style=\\textstyle}{#1}");
});
