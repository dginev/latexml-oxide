//! arxiv.sty — the widely-bundled "arXiv preprint" style
//! (George Kour, <https://github.com/kourgeorge/arxiv-style>), shipped
//! *inside* the paper's own source rather than installed in texmf.
//!
//! No Perl LaTeXML binding exists (the file is not a CTAN package at all),
//! so same-host Perl reports the same `Error:undefined:\keywords` whenever
//! the raw .sty is not read. Witness 2605.02338: Perl and Rust both emit
//! exactly one error, `undefined:\keywords`.
//!
//! **Why a binding when the file is right there on disk.** `\keywords` is
//! only undefined in configurations that do NOT raw-load style files
//! (`INCLUDE_STYLES` off — the bare default). Under `--includestyles` /
//! the ar5iv profile the paper's own arxiv.sty loads and everything works;
//! a binding that *replaced* that load would throw away the file's
//! `\@maketitle`, its `abstract`/`table` environment redefinitions and its
//! section formatting for no gain. So this binding is deliberately
//! **configuration-aware**: when raw style loading is available it does
//! nothing but hand control back to the real file; otherwise it supplies
//! the small set of definitions a document actually calls by name. The
//! gate is the same `lookup_bool("INCLUDE_STYLES")` the raw-load path
//! itself uses (`latexml_core/src/binding/content.rs` L776).
//!
//! The bare-mode definitions are a verbatim port of arxiv.sty L10, L29-31,
//! L33 and L44-47 — including `\and`'s local rebinding to `$\cdot$`, which
//! is what separates the keyword list in the rendered output. Getting the
//! ARITY right is the point: an undefined `\keywords` is recovered as a
//! zero-argument `<ltx:ERROR/>` and its braced argument then leaks into
//! the body as an unlabelled paragraph.
//!
//! Witnesses: 2605.02338 (`arxiv.sty` + `\keywords{a \and b \and c}`),
//! 2605.10111 (the `PRIMEarxiv.sty` fork — see `primearxiv_sty.rs`).
use latexml_package::prelude::*;

/// arxiv.sty L44-47, verbatim. `\keywordname` differs between the upstream
/// file and its PRIMEarxiv fork, so it is passed in. Kept as raw TeX (not
/// a `DefMacro!` body) so that the space arxiv.sty's line break leaves at
/// the end of `\and` survives exactly as the real file has it.
pub(crate) fn define_keywords(keywordname: &str) -> Result<()> {
  RawTeX!(&s!(r"\def\keywordname{{{}}}", keywordname));
  RawTeX!(
    r"\def\keywords#1{\par\addvspace\medskipamount{\rightskip=0pt plus1cm
\def\and{\ifhmode\unskip\nobreak\fi\ $\cdot$
}\noindent\keywordname\enspace\ignorespaces#1\par}}"
  );
  Ok(())
}

/// The shared bare-mode fallback for the arxiv.sty family: everything a
/// document calls by name, and nothing that only affects page layout.
pub(crate) fn define_bare_fallbacks(keywordname: &str) -> Result<()> {
  // arxiv.sty L10 / L33. On the miss-handler path these two are picked up
  // by the `\RequirePackage` dependency scan of the raw file; a registered
  // binding bypasses that scan, so request them explicitly.
  RequirePackage!("geometry");
  RequirePackage!("fancyhdr");
  // arxiv.sty L29-31.
  RawTeX!(r"\newcommand{\headeright}{A Preprint}");
  RawTeX!(r"\newcommand{\undertitle}{A Preprint}");
  RawTeX!(r"\newcommand{\shorttitle}{\@title}");
  define_keywords(keywordname)
}

LoadDefinitions!({
  if lookup_bool("INCLUDE_STYLES") {
    // Raw style loading is on: the paper's own arxiv.sty is authoritative.
    InputDefinitions!("arxiv", noltxml => true, extension => Some(Cow::Borrowed("sty")));
  } else {
    define_bare_fallbacks(r"{\bfseries \emph{Keywords}}")?;
  }
});
