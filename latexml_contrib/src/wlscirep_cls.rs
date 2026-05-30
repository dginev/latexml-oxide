//! Stub for wlscirep.cls (Wiley/Scientific Reports / Nature-related).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  // wlscirep.cls L11: `\RequirePackage[english]{babel}`. This binding mirrors
  // the class's `\RequirePackage` list but had omitted babel, so author
  // preamble that customizes captions via `\addto\captionsenglish{…}` (a babel
  // core macro / english caption hook) hit `undefined:\addto` /
  // `undefined:\captionsenglish` where Perl — which raw-loads the .cls and
  // gets babel that way (its dependency-scan loads babel) — is clean. Load it.
  // Witness 1603.09243 (`\addto\captionsenglish{\renewcommand\figurename{…}}`).
  RequirePackage!("babel", options => vec!["english".to_string()]);
  // wlscirep.cls L5: `\RequirePackage{multicol}` — the class loads multicol so
  // papers can wrap the body in `\begin{multicols}{2}…\end{multicols}` for the
  // two-column Scientific-Reports layout without an explicit `\usepackage`.
  // Perl raw-loads the .cls and gets multicol that way; our binding mirrors the
  // RequirePackage list but had omitted it, so `\begin{multicols}` errored
  // `{multicols} is not defined`. Witness 1601.07750 (`\begin{multicols}{2}`).
  RequirePackage!("multicol");
  RequirePackage!("amsmath");
  RequirePackage!("amssymb");
  RequirePackage!("amsthm");
  // wlscirep.cls L28: `\RequirePackage{wasysym}` (right after amsmath/
  // amssymb). Nature Scientific Reports papers use wasysym shape symbols
  // (`\hexagon`/`\square`/`\triangle`) as lattice-type labels in math.
  // Perl ships no wlscirep binding → raw-loads wlscirep.cls → wasysym
  // loaded. Our binding intercepts the cls and mirrored most of its
  // RequirePackages but omitted wasysym, so `\hexagon` was undefined where
  // Perl is clean. Witness 1610.05398 (`V^\delta_{\hexagon}`). RUST 1 → 0.
  RequirePackage!("wasysym");
  // wlscirep.cls L17: `\RequirePackage{calc}` (provides `\widthof`,
  // `\settototalheight`, infix `\setlength` arithmetic). Perl raw-loads
  // wlscirep.cls → calc loaded; our stub omitted it, so `\widthof` was
  // undefined where Perl is clean. Witness 1710.08155 (`\widthof`).
  RequirePackage!("calc");
  // Eager xcolor preload removed for Perl parity: it makes a later document
  // xcolor[table] load a no-op, so colortbl/array never load and array m{}/b{}
  // columns break (Unrecognized tabular template -> Extra alignment tab). The
  // document loads xcolor itself; color/definecolor stay via hyperref->color.
  // See ifacconf_cls.rs and SYNC_STATUS (eager-xcolor cluster).
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
  // wlscirep.cls L60: \RequirePackage{fancyhdr} for custom headers/footers.
  // Without it, \fancyhf / \fancyfoot used in the cls body (and in user
  // documents that override the page style) stay undefined. Witness
  // 2310.16477.
  RequirePackage!("fancyhdr");

  // wlscirep frontmatter / bibliography helpers — preserve author content.
  DefMacro!("\\JournalTitle{}", "\\emph{#1}");
  DefMacro!("\\affiliation{}",
    "\\@add@frontmatter{ltx:note}[role=affiliation]{#1}");
  DefMacro!("\\corres{}",
    "\\@add@frontmatter{ltx:note}[role=corresponding]{#1}");
  DefMacro!("\\presentadd[]{}",
    "\\@add@frontmatter{ltx:note}[role=present-address]{#2}");
});
