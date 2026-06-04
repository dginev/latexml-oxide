//! Stub for Elsevier's autart.cls (Automatica journal).
//!
//! autart.cls is an article-derivative. The native binding fallback to
//! OmniBus misses class-defined helpers like {ack} environment and
//! several elsart-style frontmatter macros.
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  // Do NOT eager-load amsthm here. Perl ships no autart binding (→ OmniBus,
  // no amsthm preload), and OmniBus already installs LAZY theorem-env
  // autoload stubs (`\begin{theorem}`/`\begin{proof}`/… each `require`
  // amsthm on first use). Preloading amsthm eagerly breaks the common
  // pattern of a document that clears a class-defined `\proof` and then
  // (re)loads amsthm to get amsthm's version:
  //     \let\proof\relax        % drop autart/class \proof
  //     \usepackage{amsthm}     % expect amsthm to (re)define \proof
  // With amsthm pre-loaded, the `\usepackage{amsthm}` is a no-op (already
  // loaded), so amsthm's `\let\proof\@proof` never re-runs and `\proof`
  // stays `\relax` → `\begin{proof}` → "{proof} environment not defined".
  // Witness arXiv:2009.00150 (autart + `\let\proof\relax` + amsthm). The
  // lazy OmniBus stub still covers papers that DON'T load amsthm themselves.

  // autart.cls L317-323: \def\ack{\section*{Acknowledgements}}
  // with \let\endack\par. Bind {ack}/{ack*} as structural
  // ltx:acknowledgements (post-processors map to canonical
  // role/styling).
  DefEnvironment!("{ack}",  "<ltx:acknowledgements>#body</ltx:acknowledgements>",
    mode => "internal_vertical");
  DefEnvironment!("{ack*}", "<ltx:acknowledgements>#body</ltx:acknowledgements>",
    mode => "internal_vertical");

  // Common elsart frontmatter macros (autart inherits elsart style) —
  // preserve author-supplied content as ltx:note frontmatter.
  DefMacro!("\\address[]{}",
    "\\@add@frontmatter{ltx:note}[role=address]{#2}");
  DefMacro!("\\thanksref{}", "\\textsuperscript{#1}");
  DefMacro!("\\corauthref{}", "\\textsuperscript{*#1}");
  DefMacro!("\\corauth{}",
    "\\@add@frontmatter{ltx:note}[role=corresponding]{#1}");
  DefMacro!("\\thanks{}",
    "\\@add@frontmatter{ltx:note}[role=thanks]{#1}");

  // autart.cls L516 defines `\qed` directly at class level (a `\Box` at the
  // end of a proof). Perl — which OmniBus-fallbacks autart and dep-scans
  // autart.cls's `\if@amsthm \RequirePackage{amsthm}` (the regex dep-scan
  // ignores the `\if@amsthm` guard) — ends up loading amsthm and so produces
  // amsthm's `\qed` (∎). We do NOT eager-load amsthm here (see the amsthm note
  // above re: 2009.00150), so mirror amsthm's `\qed`/`\ltx@qed` (∎) directly to
  // match Perl's ground-truth output. A paper that later `\usepackage{amsthm}`
  // simply re-installs the identical definitions. Used via the common
  // `\def\epf{\hfill\mbox{\qed}}` idiom OUTSIDE any proof env, so OmniBus's
  // lazy theorem-env autoload never fires. Witness 1703.03101.
  DefMacro!("\\qed", "\\ltx@qed");
  DefConstructor!("\\ltx@qed",
    "?#isMath(<ltx:XMTok role='PUNCT'>\u{220E}</ltx:XMTok>)(\u{220E})",
    enter_horizontal => true,
    reversion => "\\qed"
  );
  Let!("\\mathqed",    "\\qed");
  Let!("\\textsquare", "\\qed");
  Let!("\\qedsymbol",  "\\qed");
  Let!("\\openbox",    "\\qed");
});
