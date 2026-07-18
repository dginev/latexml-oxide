//! Binding for openmoss.cls (OpenMOSS survey/report class).
//!
//! A third sibling of `fairmeta.cls` / `selfevolagent.cls`: the same
//! `\addtolist`-based class-body frontmatter (\author/\affiliation/\contribution/
//! \checkdata/\correspondence/\abstract, \beginappendix), plus an `openmossblue`
//! colour set and `\openmossblue`/`\nm` helpers — all `Error:undefined` because
//! an unknown `.cls` body is not raw-loaded. Route the frontmatter through
//! `\@add@frontmatter`/`\lx@add@author`/`\lx@add@abstract`. See `fairmeta_cls.rs`.
//!
//! Witness: 2605.12090 (ar5iv #605).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amssymb");
  RequirePackage!("amsthm");
  RequirePackage!("graphicx");
  RequirePackage!("subcaption");
  RequirePackage!("xcolor");
  RequirePackage!("booktabs");
  RequirePackage!("multirow");
  RequirePackage!("bm");
  RequirePackage!("etoolbox");
  // \RequirePackage[latin, english]{babel} (openmoss.cls L22) — the class does
  // `\addto\extrasenglish{…}`, so babel's `\extrasenglish`/`\addto` must exist.
  RequirePackage!("babel", options => vec![s!("latin"), s!("english")]);
  RequirePackage!("ulem");
  RequirePackage!("caption");
  RequirePackage!("hyperref");
  RequirePackage!("cleveref");
  RequirePackage!("natbib");
  RequirePackage!("nicematrix");
  // \RequirePackage[most]{tcolorbox} (openmoss.cls L36) — PassOptions before the
  // require so tcolorbox loads its `most` libraries (see fairmeta_cls.rs).
  pass_options("tcolorbox", "sty", vec![s!("most")])?;
  RequirePackage!("tcolorbox");

  def_macro_noop("\\geometry{}")?;

  // Class palette (used by \openmossblue and \textcolor{openmossblue}).
  Digest!("\\definecolor{OpenMossCyan}{HTML}{82D9FF}")?;
  Digest!("\\definecolor{OpenMossBlue}{HTML}{82B1FF}")?;
  Digest!("\\definecolor{openmossbg}{HTML}{387BD9}")?;
  Digest!("\\definecolor{openmossblue}{HTML}{387BD9}")?;
  DefMacro!("\\openmossblue{}", "{\\bfseries\\color{openmossblue}#1}");
  DefMacro!("\\nm{}", "#1");

  // Frontmatter (see fairmeta_cls.rs). Accumulator lists become no-ops.
  def_macro_noop("\\authorlist")?;
  def_macro_noop("\\affiliationlist")?;
  def_macro_noop("\\contributionlist")?;
  def_macro_noop("\\checkdatalist")?;

  DefMacro!("\\author[]{}", "\\lx@add@author{#2}");
  DefMacro!(
    "\\affiliation[]{}",
    "\\@add@frontmatter{ltx:note}[role=affiliation]{#2}"
  );
  DefMacro!(
    "\\contribution[]{}",
    "\\@add@frontmatter{ltx:note}[role=contribution]{#2}"
  );
  // \checkdata[label]{value} — arbitrary label (e.g. "Github Repo", a \url),
  // rendered as note CONTENT "label: value" (a label with a space can't be a
  // role attribute).
  DefMacro!("\\checkdata[]{}", "\\@add@frontmatter{ltx:note}{#1: #2}");
  DefMacro!(
    "\\correspondence{}",
    "\\@add@frontmatter{ltx:note}[role=correspondence]{#1}"
  );
  DefMacro!("\\abstract{}", "\\lx@add@abstract{#1}");
  DefMacro!("\\email{}", "\\href{mailto:#1}{\\texttt{#1}}");
  DefMacro!("\\beginappendix", "\\appendix");
});
