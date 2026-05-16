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
  // Perl aa.cls.ltxml L57 resets the amsmath loaded-flag so documents
  // that need amsmath's {cases} can `\usepackage{amsmath}` again after
  // aa_support pulled amsmath in implicitly. Example:
  // arXiv:astro-ph/0203101. Without this reset the re-RequirePackage
  // is a no-op and {cases} stays as the plain-TeX version above.
  AssignValue!("amsmath.sty_loaded" => Stored::None, Some(Scope::Global));

  // aa.cls L1651-1664: \tablebib{...} / \tablefoot{...} emit a labeled
  // tablefootnote block. Render as paragraphs prefixed by their names.
  // Witnesses 2406.05044, 2406.14661.
  DefMacro!("\\tablebib{}", "\\par\\textbf{References.} #1");
  DefMacro!("\\tablefoot{}", "\\par\\textbf{Notes.} #1");
  DefMacro!("\\tablebibname", "References");
  DefMacro!("\\tablefootname", "Notes");
});
