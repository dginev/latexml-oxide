//! Stub for elife.cls (eLife journal class).
//!
//! elife.cls extends extarticle with fontspec/xetex/luatex font setup
//! and defines eLife-branded colors. The raw cls's preamble does
//! `\RequirePackage{...}` for stix/opensans/XITSMath which our system
//! handles as "missing fonts" warnings, then proceeds past the
//! `\definecolor{eLifeMediumGrey}` block — but a downstream macro
//! (\NAT@nmfmt redefinition with `\color{eLifeMediumGrey}`) somehow
//! mis-handles the color name in some contexts, surfacing as 69+
//! `unexpected:eLifeMediumGrey` errors.
//!
//! Pre-define the 4 eLife color names so they're guaranteed available.
//! Witness 2307.12956.
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("extarticle");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  RequirePackage!("amssymb");
  RequirePackage!("graphicx");
  // Pre-load xcolor with [dvipsnames, table] so user xcolor calls
  // don't silently option-clash.
  RequirePackage!("xcolor", options => vec!["dvipsnames".to_string(), "table".to_string()]);
  RequirePackage!("booktabs");
  RequirePackage!("hyperref");
  RequirePackage!("authblk");
  RequirePackage!("microtype");

  // Eagerly define eLife-branded colors so user `\color{eLifeMediumGrey}`
  // calls work regardless of where the raw cls bails.
  RawTeX!("\\definecolor{eLifeDarkBlue}{HTML}{273B81}");
  RawTeX!("\\definecolor{eLifeLightBlue}{HTML}{0A9DD9}");
  RawTeX!("\\definecolor{eLifeMediumGrey}{HTML}{6D6E70}");
  RawTeX!("\\definecolor{eLifeLightGrey}{HTML}{929497}");

  // Conditional flags used by raw cls preamble.
  DefConditional!("\\if@onehalfspacing");
  DefConditional!("\\if@doublespacing");
});
