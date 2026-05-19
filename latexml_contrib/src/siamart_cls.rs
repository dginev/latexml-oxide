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
  // siamart220329 L1361: \RequirePackage{algorithm}.
  RequirePackage!("algorithm");
  RequirePackage!("url");
  // siamart220329 L1285: \RequirePackage{hyperref}[6.83] (unconditional).
  // Mirror so papers using \hidelinks/\href/\hypersetup don't error.
  // hyperref MUST come before cleveref (cleveref errors out otherwise).
  // Witness 2407.00765 (siamart220329 with `[hidelinks,…]` class option).
  RequirePackage!("hyperref");
  // siamart220329 L1371: \RequirePackage[capitalize,nameinlink]{cleveref}.
  // Loaded AFTER hyperref to satisfy cleveref's ordering check.
  // Witness 2501.11060 (Error:latex:cleveref must be loaded after hyperref!).
  RequirePackage!("cleveref");
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

  // siamart frontmatter primitives. Round-34 surpass-Perl: preserve
  // \dedicatory, \fundingsource, and \funding as author-typed
  // frontmatter notes (the funding text is often a real funding
  // statement worth keeping). \headers{left}{right} → running-head
  // text, preserve as ltx:note.
  DefMacro!("\\headers{}{}",
    "\\@add@frontmatter{ltx:note}[role=runningheads]{#1 / #2}");
  DefMacro!("\\dedicatory{}",
    "\\@add@frontmatter{ltx:note}[role=dedicatory]{#1}");
  DefMacro!("\\fundingsource{}",
    "\\@add@frontmatter{ltx:note}[role=funding-source]{#1}");
  // siamart papers often \externaldocument supplement/article before
  // loading xr — pre-stub.
  def_macro_noop("\\externaldocument[]{}")?;
  def_macro_noop("\\externalcitedocument[]{}")?;
  // siamart220329 L1130: \funding{...} writes a marked line in the
  // titlepage. Preserve as ltx:acknowledgements (matching the user's
  // memory: prefer ltx:acknowledgements over a section).
  DefConstructor!("\\funding{}",
    "<ltx:acknowledgements name='Funding'>#1</ltx:acknowledgements>");
  // {MSCcodes} env — siamart220329 L743 wraps content in an "@abssec"
  // (frontmatter section). Mirror as keywords-like classification block.
  DefEnvironment!(
    "{MSCcodes}",
    "<ltx:classification scheme='MSC'>#body</ltx:classification>"
  );

  // {AMS} env — siamart190516 L742 same pattern (AMS subject
  // classification). Some older SIAM templates use {AMS} instead of
  // {MSCcodes}. Witness 2306.11286, 2306.13351 (siamart190516).
  DefEnvironment!(
    "{AMS}",
    "<ltx:classification scheme='AMS'>#body</ltx:classification>"
  );
});
