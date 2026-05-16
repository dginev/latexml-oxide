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

  // siamart220329 L1361 \RequirePackages{algorithm}. Our package set
  // doesn't ship algorithm binding; provide a minimal env stub so
  // siamart-using papers don't crash on \begin{algorithm}.
  DefEnvironment!(
    "{algorithm}",
    "<ltx:float class='ltx_algorithm'>#body</ltx:float>"
  );
});
