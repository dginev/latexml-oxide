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
  // table-notes block below the tabular, inside the table float. Perl
  // (aa_support.sty.ltxml L229) renders `\tablefoot` as a plain `\footnote`
  // → `ltx:note` (Meta.class, valid in a table float, and — crucially —
  // able to hold the *multi-paragraph* note bodies A&A papers write, where a
  // blank line inside `\tablefoot{...}` becomes a `\par`).
  //
  // A bare `\par\textbf{Notes.} #1` (the old form, witnesses 2406.05044 /
  // 2406.14661) works only for single-paragraph notes: a `\par` *inside*
  // `#1` builds an `ltx:para`, which `table_model` forbids
  // (`ltx:para isn't allowed in ltx:table`; witness 1701.02312, whose
  // `\tablefoot` has a blank-line `\par`). Route through `\footnote` like
  // Perl — but keep the visible "Notes."/"References." label (real aa.cls
  // shows it) by prefixing it *inside* the note, where block content is legal.
  DefMacro!("\\tablebib{}", "\\footnote{\\textbf{References.} #1}");
  DefMacro!("\\tablefoot{}", "\\footnote{\\textbf{Notes.} #1}");
  DefMacro!("\\tablebibname", "References");
  DefMacro!("\\tablefootname", "Notes");
  // A&A authors use \orcid for ORCID identifier. Preserve as ltx:note.
  DefMacro!("\\orcid{}",
    "\\@add@frontmatter{ltx:note}[role=orcid]{#1}");
});
