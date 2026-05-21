//! Stub for mcom-l.cls / proc-l.cls / tran-l.cls (AMS journal classes).
use latexml_package::prelude::*;

LoadDefinitions!({
  // mcom-l L30: \LoadClass{amsart}
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  RequirePackage!("amssymb");
  RequirePackage!("xcolor");
  RequirePackage!("hyperref");
  RequirePackage!("url");

  // \copyrightinfo{year}{holder} — AMS \copyrightinfo from ams_support.
  // Some mcom-l/proc-l papers call it directly without `\usepackage
  // {ams_support}`. Preserve as ltx:note. Witness 2503.09526.
  DefMacro!("\\copyrightinfo{}{}",
    "\\@add@frontmatter{ltx:note}[role=copyright]{\\copyright #1: #2}");
  // \commby{person} — amsart.cls L565: "(Communicated by ...)". AMS
  // journal frontmatter often calls it; preserve as ltx:note.
  // Witness 2409.14512 (proc-l).
  DefMacro!("\\commby{}",
    "\\@add@frontmatter{ltx:note}[role=communicated-by]{#1}");

  // AMS journal frontmatter — preserve author content as ltx:note
  // frontmatter entries with role markers (per content-preservation
  // directive).
  DefMacro!("\\subjclass[]{}",
    "\\@add@frontmatter{ltx:classification}[scheme=AMS]{#2}");
  DefMacro!("\\keywords{}",
    "\\@add@frontmatter{ltx:classification}[scheme=keywords]{#1}");
  DefMacro!("\\thanks{}",
    "\\@add@frontmatter{ltx:note}[role=thanks]{#1}");
  DefMacro!("\\address{}",
    "\\@add@frontmatter{ltx:note}[role=address]{#1}");
  DefMacro!("\\curraddr{}",
    "\\@add@frontmatter{ltx:note}[role=current-address]{#1}");
  DefMacro!("\\email{}",
    "\\@add@frontmatter{ltx:note}[role=email]{#1}");
  DefMacro!("\\urladdr{}",
    "\\@add@frontmatter{ltx:note}[role=url]{#1}");
  DefMacro!("\\dedicatory{}",
    "\\@add@frontmatter{ltx:note}[role=dedication]{#1}");
});
