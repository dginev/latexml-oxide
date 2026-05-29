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
  // Eager xcolor preload removed for Perl parity: it makes a later document
  // xcolor[table] load a no-op, so colortbl/array never load and array m{}/b{}
  // columns break (Unrecognized tabular template -> Extra alignment tab). The
  // document loads xcolor itself; color/definecolor stay via hyperref->color.
  // See ifacconf_cls.rs and SYNC_STATUS (eager-xcolor cluster).
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
  // loading xr — pre-stub. siamart loads xr-hyper (not xr), which
  // supports `\externaldocument[prefix][nocite]{file}` (two optional
  // args). Semiverbatim on the file arg neutralizes `_` so paper-
  // bundled filenames like `ex_supplement` don't trip text-mode `_`
  // errors. Witness 2402.12241.
  def_macro_noop("\\externaldocument[][] Semiverbatim")?;
  def_macro_noop("\\externalcitedocument[][] Semiverbatim")?;
  // siamart220329 L1130: \funding{...} writes a marked line in the
  // titlepage. Preserve as a frontmatter ltx:acknowledgements via
  // \@add@frontmatter so the element lands at top-level no matter
  // where the macro is invoked. Papers commonly nest \funding{...}
  // inside \thanks{...} (which is rendered as ltx:note); an inline
  // DefConstructor of ltx:acknowledgements there fires
  // Error:malformed:ltx:acknowledgements isn't allowed in <ltx:note>.
  // Witness 2311.08549.
  DefMacro!("\\funding{}",
    "\\@add@frontmatter{ltx:acknowledgements}[name=Funding]{#1}");
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

  // {@abssec}{title} — siamonline190516.cls L810 (`\newenvironment{@abssec}[1]`):
  // a titled frontmatter paragraph (`#1.` bold heading, then body). The
  // public abstract/keywords/AMS/keyword envs wrap it, but papers also
  // use `\begin{@abssec}{Author Information…}…\end{@abssec}` directly.
  // `\begin{@abssec}` resolves via `\csname @abssec\endcsname`, so it
  // works regardless of `@` catcode. siamonline190516.cls is reported
  // missing-file in Perl (24 undefined-macro errors there); our siamart
  // binding covers everything else, so defining this env makes the
  // conversion clean. `{@doisec}` (L820) is the same shape with a
  // trailing rule. Witness 2005.11911 (`\documentclass{siamonline190516}`).
  // Use `inline-logical-block` (Misc.class) rather than `ltx:para`:
  // papers place `\begin{@abssec}` inside frontmatter note contexts
  // (e.g. an author/acknowledgements block already wrapped in
  // `ltx:note`), where a block `ltx:para` is rejected
  // (`malformed:ltx:para isn't allowed in <ltx:note>`). Misc.class is
  // accepted in both inline and block positions.
  DefEnvironment!(
    "{@abssec}{}",
    "<ltx:inline-logical-block class='ltx_abssec'><ltx:text font='bold'>#1. </ltx:text>#body</ltx:inline-logical-block>"
  );
  DefEnvironment!(
    "{@doisec}{}",
    "<ltx:inline-logical-block class='ltx_doisec'><ltx:text font='bold'>#1. </ltx:text>#body</ltx:inline-logical-block>"
  );
});
