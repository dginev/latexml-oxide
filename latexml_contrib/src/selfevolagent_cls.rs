//! Binding for selfevolagent.cls (Self-Evolving-Agents survey class).
//!
//! A near-identical sibling of `fairmeta.cls`: the same `\addtolist[5]`-based
//! frontmatter interface (\author/\affiliation/\contribution/\metadata/
//! \correspondence/\abstract, \beginappendix) defined in the class BODY, which
//! an unknown `.cls` does not raw-load → all `Error:undefined`. Route them
//! through `\@add@frontmatter`/`\lx@add@author`/`\lx@add@abstract`. See
//! `fairmeta_cls.rs` for the shared rationale.
//!
//! Witness: 2508.07407 (ar5iv #556).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amssymb");
  RequirePackage!("graphicx");
  RequirePackage!("subcaption");
  RequirePackage!("xcolor");
  RequirePackage!("colortbl");
  RequirePackage!("booktabs");
  RequirePackage!("multirow");
  RequirePackage!("bm");
  RequirePackage!("etoolbox");
  RequirePackage!("caption");
  RequirePackage!("hyperref");
  RequirePackage!("natbib");
  RequirePackage!("nicematrix");
  // \RequirePackage[most]{tcolorbox} (selfevolagent.cls L32) — PassOptions before
  // the require so tcolorbox loads its `most` libraries (see fairmeta_cls.rs).
  pass_options("tcolorbox", "sty", vec![s!("most")])?;
  RequirePackage!("tcolorbox");

  // \geometry{...} — visual-only page-geometry hint.
  def_macro_noop("\\geometry{}")?;

  // Class palette (\color{selfevolagentfg} is used by the abstract box).
  Digest!("\\definecolor{selfevolagentpink}{HTML}{4b0082}")?;
  Digest!("\\definecolor{selfevolagentfg}{HTML}{1C2B33}")?;
  Digest!("\\definecolor{selfevolagentbg}{HTML}{fffafa}")?;
  Digest!("\\definecolor{commentcolor}{rgb}{0.294, 0, 0.51}")?;
  Digest!("\\definecolor{selfevolagent_dark}{HTML}{37D2A6}")?;
  Digest!("\\definecolor{selfevolagent_light}{HTML}{9BE9D3}")?;
  Digest!("\\definecolor{selfevolagent_lighter}{HTML}{CDF4E9}")?;
  Digest!("\\definecolor{selfevolagent_blue}{HTML}{0064E0}")?;

  // Frontmatter (see fairmeta_cls.rs). The accumulator lists become no-ops.
  def_macro_noop("\\authorlist")?;
  def_macro_noop("\\affiliationlist")?;
  def_macro_noop("\\contributionlist")?;
  def_macro_noop("\\metadatalist")?;

  DefMacro!("\\author[]{}", "\\lx@add@author{#2}");
  DefMacro!(
    "\\affiliation[]{}",
    "\\@add@frontmatter{ltx:note}[role=affiliation]{#2}"
  );
  DefMacro!(
    "\\contribution[]{}",
    "\\@add@frontmatter{ltx:note}[role=contribution]{#2}"
  );
  // \metadata[label]{value}: the label can be arbitrary markup (e.g. an
  // \includegraphics-bearing "Github" chip), so it cannot go in a `role`
  // attribute — render it as note CONTENT "label: value".
  DefMacro!("\\metadata[]{}", "\\@add@frontmatter{ltx:note}{#1: #2}");
  DefMacro!(
    "\\correspondence{}",
    "\\@add@frontmatter{ltx:note}[role=correspondence]{#1}"
  );
  DefMacro!("\\date{}", "\\@add@frontmatter{ltx:note}[role=date]{#1}");
  DefMacro!("\\abstract{}", "\\lx@add@abstract{#1}");
  DefMacro!("\\email{}", "\\href{mailto:#1}{\\texttt{#1}}");
  DefMacro!("\\beginappendix", "\\appendix");
});
