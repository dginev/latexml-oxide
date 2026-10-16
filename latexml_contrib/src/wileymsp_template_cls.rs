//! Stub for WileyMSP-template.cls (Wiley Mathematical Sciences Publishers).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  // Eager xcolor preload removed for Perl parity: it makes a later document
  // xcolor[table] load a no-op, so colortbl/array never load and array m{}/b{}
  // columns break (Unrecognized tabular template -> Extra alignment tab). The
  // document loads xcolor itself; color/definecolor stay via hyperref->color.
  // See ifacconf_cls.rs and SYNC_STATUS (eager-xcolor cluster).
  RequirePackage!("hyperref");
  RequirePackage!("fancyhdr");
  RequirePackage!("ragged2e");
  // WileyMSP-template.cls L: `\RequirePackage{framed}` — needed for
  // {snugshade} environment used by template's editorial callout boxes.
  // Witness 2208.03623.
  RequirePackage!("framed");
  RequirePackage!("authblk");
  RequirePackage!("caption");
  // WileyMSP-template page-sets with geometry; papers use `\geometry{…}` in the
  // preamble without an explicit `\usepackage{geometry}` (witness 2306.02129).
  // Load it here (defining the no-op `\geometry`) instead of an engine-wide
  // kernel stub, which would make `\ifcsname geometry\endcsname` falsely true
  // for every document (broke 2005.03740). See latex_constructs.rs geometry note.
  RequirePackage!("geometry");
  // WileyMSP-template.cls:5-10 `\RequirePackage{ragged2e}`, `{graphicx}`, `{caption}`, `[margin=0.5in]{geometry}`,
  // `{fancyhdr}`, `{gensymb}` (every one of the 41 sampled copies): papers write `\degree`, `\ohm`, `\micro`,
  // `\celsius` (41 of run 353 with them undefined; 2003.01904, 2007.06369, 2103.00309) and `\includegraphics`.
  RequirePackage!("graphicx");
  RequirePackage!("gensymb");

  DefMacro!(T_CS!("\\begin{affiliations}"), None, "");
  DefMacro!(T_CS!("\\end{affiliations}"), None, "");
  // Preserve author content as ltx:note frontmatter.
  DefMacro!(
    "\\correspondingauthor[]{}",
    "\\@add@frontmatter{ltx:note}[role=corresponding]{#2}"
  );
  DefMacro!(
    "\\corres{}",
    "\\@add@frontmatter{ltx:note}[role=corresponding]{#1}"
  );
  // WileyMSP-template.cls:34-37 `\newcommand{\dedication}[1]{\medskip\textit{#1}}` (2003.09099's copy, as 206 of
  // the 256 shipped): the dedication under the authors, the Perl bindings' pubnote (aa_support.sty.ltxml:117,
  // sv_support.sty.ltxml:76). Undefined in 158 papers of run 353, nearly all with the template's empty
  // `\dedication{}` (2003.09099, 2006.03434, 2007.06369, 2005.10895; text: "To all front line health workers.").
  DefMacro!("\\dedication{}", "\\lx@add@pubnote[role=dedication]{#1}");
  // WileyMSP-template.cls:49-52 `\newcommand{\threesubsection}[1]{\medskip\textit{#1}: }`: the run-in italic heads
  // of the Experimental Section (`\threesubsection{Sample preparation}\\ …`, 2009.06328, 2102.13559, 2111.07484; 87
  // papers of run 353).
  DefMacro!("\\threesubsection{}", "\\medskip\\textit{#1}: ");
});
