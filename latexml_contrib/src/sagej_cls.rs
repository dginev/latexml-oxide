//! Stub for sagej.cls (SAGE journals).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  // Do NOT eager-load xcolor (Perl ships no sagej binding → OmniBus, no
  // preload). A preloaded xcolor makes a later `\usepackage[table]{xcolor}`
  // a no-op → colortbl/array never load → array `m{}`/`b{}` columns are
  // "Unrecognized tabular template" → "Extra alignment tab". The document
  // loads xcolor with its own options; `\color`/`\definecolor` stay
  // available via hyperref→color. See ifacconf_cls.rs / SYNC_STATUS.
  RequirePackage!("hyperref");
  // sagej templates use \toprule / \midrule / \bottomrule from booktabs.
  // The raw cls relies on the user `\usepackage{booktabs}` but many
  // papers don't load it explicitly. Eager-load. Witness 2309.01813.
  RequirePackage!("booktabs");
  // sagej.cls L108: `\RequirePackage{latexsym,ifthen,rotating,calc,textcase,
  // booktabs,color,endnotes}`. Perl ships no sagej binding → raw-loads the
  // bundled sagej.cls → endnotes loaded → `\endnote`/`\theendnotes` defined.
  // Our binding intercepts sagej.cls (so that `\RequirePackage` never runs),
  // leaving `\endnote` undefined where Perl is clean. Load endnotes to match
  // the real .cls (SAGE papers use `\endnote{…}` for footnote-style notes).
  // Witness 1901.10968 (`\endnote`, `\theendnotes`).
  RequirePackage!("endnotes");

  // sagej frontmatter — preserve author content.
  DefMacro!("\\corrauth{}",
    "\\@add@frontmatter{ltx:note}[role=corresponding]{#1}");
  DefMacro!("\\affiliation{}",
    "\\@add@frontmatter{ltx:note}[role=affiliation]{#1}");
  DefMacro!("\\runninghead{}",
    "\\@add@frontmatter{ltx:note}[role=runninghead]{#1}");

  // {acks}, {funding}, {dci} envs (sagej L456-470). Use
  // internal_vertical mode so block-level body (paragraphs, lists,
  // funding-statement prose) is accepted — restricted_horizontal
  // default tripped `Attempt to end mode restricted_horizontal in
  // internal_vertical` on multi-paragraph bodies.
  DefEnvironment!("{acks}", "<ltx:acknowledgements>#body</ltx:acknowledgements>",
    mode => "internal_vertical");
  DefEnvironment!("{funding}", "<ltx:acknowledgements name='funding'>#body</ltx:acknowledgements>",
    mode => "internal_vertical");
  DefEnvironment!("{dci}", "<ltx:acknowledgements name='dci'>#body</ltx:acknowledgements>",
    mode => "internal_vertical");
});
