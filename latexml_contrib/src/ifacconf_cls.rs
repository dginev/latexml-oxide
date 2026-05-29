//! Stub for ifacconf.cls (IFAC conference proceedings).
//!
//! IFAC papers use Elsevier-style frontmatter (\author, \address, \ead,
//! \sep, \thanks, \fnref, \corref). Route to OmniBus and provide
//! content-preserving stubs identical to cas-* / ceurart patterns.
//!
//! Witness 2503.16455 (ifacconf paper).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amssymb");
  RequirePackage!("amsthm");
  // NOTE: do NOT eagerly `RequirePackage("xcolor")` here. Perl ships no
  // ifacconf binding (it falls to OmniBus) and never preloads xcolor for
  // this class. Preloading xcolor means a later document-level
  // `\usepackage[table]{xcolor}` (or any `\usepackage[<opts>]{xcolor}`)
  // sees xcolor as already-loaded and silently drops its options — so the
  // `table` option never fires `\RequirePackage{colortbl}`, `array` is
  // never pulled, and `m{<frac>\textwidth}` / `b{}` column types are
  // "Unrecognized tabular template" → cascading "Extra alignment tab"
  // errors. The document loads xcolor itself when it needs it (with the
  // right options). Witness arXiv:2004.03970 (`\usepackage[table]{xcolor}`
  // + `m{0.10\textwidth}` table → 8 errors in Rust, 0 in Perl).
  RequirePackage!("hyperref");
  RequirePackage!("graphicx");
  RequirePackage!("natbib");

  // Elsevier-style separator + frontmatter helpers — preserve content.
  DefMacro!("\\sep", ",");
  DefMacro!("\\address[]{}",
    "\\@add@frontmatter{ltx:note}[role=address]{#2}");
  DefMacro!("\\ead[]{}",
    "\\@add@frontmatter{ltx:note}[role=email]{#2}");
  DefMacro!("\\thanks{}",
    "\\@add@frontmatter{ltx:note}[role=thanks]{#1}");
  DefMacro!("\\fnref{}", "\\textsuperscript{#1}");
  DefMacro!("\\corref{}", "\\textsuperscript{*#1}");
  DefMacro!("\\fntext[]{}",
    "\\@add@frontmatter{ltx:note}[role=footnote]{#2}");
  DefMacro!("\\cortext[]{}",
    "\\@add@frontmatter{ltx:note}[role=corresp]{#2}");
});
