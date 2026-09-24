//! Stub for bytedance_seed.cls (ByteDance Seed paper template).
//!
//! User-bundled template with author/affiliation/contribution-list
//! helpers built on \addtolist. Our raw-load fails on the
//! `\addtolist[5]` (5-arg, optional-arg-first) signature; provide
//! content-preserving stubs so the metadata reaches XML output.
//!
//! Witness: 2503.04598 (Seed-1.5 thinking paper).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amssymb");
  // bytedance_seed.cls:11-101 (the class ships with each paper): this binding
  // replaces the class, so it loads what the class loads, in its order and with
  // its options. Missing, the papers flooded on `\citet` (natbib, 2605.05558:
  // 501 errors), `\cref` (cleveref, 2605.02134), `{subfigure}` and
  // `\newtcblisting` (subcaption, tcolorbox `most`, 2605.06548).
  RequirePackage!("geometry");
  RequirePackage!("microtype");
  RequirePackage!("placeins");
  RequirePackage!("setspace");
  RequirePackage!("parskip");
  RequirePackage!("babel", options => vec!["latin".to_string(), "english".to_string()]);
  RequirePackage!("hyphenat");
  RequirePackage!("lipsum");
  RequirePackage!("etoolbox");
  RequirePackage!("fancyhdr");
  RequirePackage!("ulem");
  RequirePackage!("graphicx");
  RequirePackage!("subcaption");
  RequirePackage!("booktabs");
  RequirePackage!("nicematrix");
  RequirePackage!("multirow");
  RequirePackage!("bm");
  RequirePackage!("tcolorbox", options => vec!["most".to_string()]);
  RequirePackage!("xcolor");
  // bytedance_seed.cls:35-37: the template's colours (and `\\uiucblue{…}`), which papers use in tikz
  // (`\draw[uiucblue]`, 2605.05558: "I do not know the key '/tikz/uiucblue'").
  RawTeX!(
    r"\definecolor{uiucbg}{HTML}{2E5AA8}\definecolor{uiucblue}{HTML}{2E5AA8}
\newcommand{\uiucblue}[1]{{\bfseries\color[HTML]{2E5AA8} #1}}"
  );
  RequirePackage!("hyperref");
  RequirePackage!("cleveref", options => vec!["noabbrev".to_string(), "nameinlink".to_string()]);
  RequirePackage!("natbib", options => vec!["numbers".to_string(), "sort&compress".to_string()]);
  RequirePackage!("titlesec");
  RequirePackage!("caption");

  // Author/affiliation/contribution lists — preserve as ltx:note.
  def_macro_noop("\\authorlist")?;
  def_macro_noop("\\affiliationlist")?;
  def_macro_noop("\\contributionlist")?;
  def_macro_noop("\\checkdatalist")?;
  // \author[mark]{name} — the frontmatter author API, the mark as its annotation
  // (as jheppub/quantumarticle). It was defined as `\author{#2}`, ITSELF: an
  // endless expansion, caught by the loop guard for a plain name but a wall-clock
  // hang for the `Name\textsuperscript{1*}` every paper writes (arXiv 2605.24117,
  // 2605.05558, 2605.02134, 2605.15735, 2605.06548; Perl and pdflatex complete).
  // Witness 2503.04598. Guard: `perfect_kernel_batch56::bytedance_author_is_a_creator`.
  DefMacro!(
    "\\author[]{}",
    "\\lx@add@creator[role=author,annotations={#1}]{#2}"
  );
  // \affiliation[mark]{text} — emit affiliation note.
  DefMacro!(
    "\\affiliation[]{}",
    "\\@add@frontmatter{ltx:note}[role=affiliation]{#2}"
  );
  // \contribution[mark]{text} — emit contribution note.
  DefMacro!(
    "\\contribution[]{}",
    "\\@add@frontmatter{ltx:note}[role=contribution]{#2}"
  );
  // \checkdata[label]{value} — emit as keyed note (label: value).
  DefMacro!(
    "\\checkdata[]{}",
    "\\@add@frontmatter{ltx:note}[role=#1]{#2}"
  );
  // \correspondence{text} — emit corresponding-author note.
  DefMacro!(
    "\\correspondence{}",
    "\\@add@frontmatter{ltx:note}[role=correspondence]{#1}"
  );
  // \beginappendix — bytedance uses this in place of \appendix.
  DefMacro!("\\beginappendix", "\\appendix");
  DefMacro!("\\seedblue{}", "{\\color[HTML]{2E5AA8}\\textbf{#1}}");
  DefMacro!("\\nm{}", "#1");
  DefMacro!("\\citeas{}", "\\cite{#1}");
});
