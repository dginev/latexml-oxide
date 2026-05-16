//! Stub binding for SIAM siamart-family classes (siamart, siamart220329, ...).
//!
//! Activated via prefix-match: any class name starting with "siamart" routes
//! here. Defines the high-level macros papers use (\newsiamthm, \newsiamremark,
//! \headers, \dedicatory) on top of OmniBus's article-like behaviour.
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  // siamart220329.cls L58: \RequirePackage[leqno]{amsmath}.
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  // Many siamart papers pre-define colors in their macros.tex before
  // their own `\usepackage{xcolor}`. Defensive xcolor load matches Perl
  // behaviour. Witness 2405.17955 (EPCID).
  RequirePackage!("xcolor");
  // siamart220329 L1371: \RequirePackage[capitalize,nameinlink]{cleveref}.
  RequirePackage!("cleveref");
  // siamart220329 L1361: \RequirePackage{algorithm}.
  RequirePackage!("algorithm");
  RequirePackage!("url");
  // siamonline220329 L1676: \RequirePackage[mathlines]{lineno}.
  RequirePackage!("lineno");
  // ifpdf is auto-loaded inside epstopdf; our binding triggers
  // \ifpdf usage during epstopdf raw-load, so preload it here.
  RequirePackage!("ifpdf");
  RawTeX!(
    r"\newtheorem{theorem}{Theorem}
\newtheorem{lemma}[theorem]{Lemma}
\newtheorem{corollary}[theorem]{Corollary}
\newtheorem{proposition}[theorem]{Proposition}
\newtheorem{definition}[theorem]{Definition}"
  );

  // \newsiamthm{name}{title}, \newsiamremark{name}{title}: siamart220329 L1452,
  // L1469. Both reduce to `\newtheorem{name}[theorem]{title}`; the theoremstyle
  // tweaks are visual and don't affect XML.
  DefMacro!("\\newsiamthm{}{}", r"\newtheorem{#1}[theorem]{#2}");
  DefMacro!("\\newsiamremark{}{}", r"\newtheorem{#1}[theorem]{#2}");

  // siamart frontmatter primitives — no-op to suppress undefined errors.
  DefMacro!("\\headers{}{}", "");
  DefMacro!("\\dedicatory{}", "");
  DefMacro!("\\fundingsource{}", "");
  // siamart papers often \externaldocument supplement/article before
  // loading xr — pre-stub.
  DefMacro!("\\externaldocument[]{}", "");
  DefMacro!("\\externalcitedocument[]{}", "");
  // siamart220329 L1130: \funding{...} writes a marked line in the
  // titlepage. Stub as gobble; the text appears in acknowledgements
  // section of the paper typically.
  DefMacro!("\\funding{}", "");
  // {MSCcodes} env — siamart220329 L743 wraps content in an "@abssec"
  // (frontmatter section). Mirror as keywords-like classification block.
  DefEnvironment!(
    "{MSCcodes}",
    "<ltx:classification scheme='MSC'>#body</ltx:classification>"
  );

});
