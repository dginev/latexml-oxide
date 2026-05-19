//! Stub for wlscirep.cls (Wiley/Scientific Reports / Nature-related).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amssymb");
  RequirePackage!("amsthm");
  RequirePackage!("xcolor");
  RequirePackage!("hyperref");
  // wlscirep.cls L53: `\RequirePackage[superscript,biblabel,nomove]{cite}`.
  // Our cite binding Lets `\citeonline` -> `\cite`, so authors writing
  // `\citeonline{key}` (witness 2306.04599) need cite loaded.
  RequirePackage!("cite");
  // wlscirep.cls L52 has a commented-out `\RequirePackage[numbers]{natbib}`
  // — Perl's dep-scanner picks it up anyway (it scans .cls source-text
  // not the LaTeX execution path). Authors writing
  // `\bibliographystyle{agsm}` get a .bbl with `\harvarditem` /
  // `\harvardand` / `\harvardyearleft`, which natbib provides as
  // backwards-compat aliases for cite. Witness 2308.08350.
  RequirePackage!("natbib");
  // wlscirep.cls L29: \RequirePackage{booktabs} unconditionally.
  // Witness 2408.07161 (\toprule/\midrule/\bottomrule used without
  // explicit \usepackage{booktabs}).
  RequirePackage!("booktabs");
  // wlscirep also configures caption layout — pull caption.sty so
  // \captionsetup is available. Witness 2411.06447, 2411.10607.
  RequirePackage!("caption");

  // wlscirep frontmatter / bibliography helpers — preserve author content.
  DefMacro!("\\JournalTitle{}", "\\emph{#1}");
  DefMacro!("\\affiliation{}",
    "\\@add@frontmatter{ltx:note}[role=affiliation]{#1}");
  DefMacro!("\\corres{}",
    "\\@add@frontmatter{ltx:note}[role=corresponding]{#1}");
  DefMacro!("\\presentadd[]{}",
    "\\@add@frontmatter{ltx:note}[role=present-address]{#2}");
});
