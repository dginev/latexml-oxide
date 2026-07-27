//! PRIMEarxiv.sty — a paper-bundled fork of `arxiv.sty` ("based on the
//! Arxiv style of George Kour", its own first line). It differs from the
//! upstream file only cosmetically: no `\headeright`/`\undertitle`/
//! `\shorttitle`, a rule-less header, and `\keywordname` written
//! `{\bfseries \emph Keywords}` — an unbraced `\emph` that italicises the
//! single letter `K` (PRIMEarxiv.sty L39). That quirk is reproduced here
//! rather than corrected: it is what the raw file renders under
//! `--includestyles` (`<em>K</em><span class="ltx_font_bold">eywords</span>`).
//!
//! Same rationale, same configuration gate and same Perl-parity position
//! as [`crate::arxiv_sty`] — see that module's header. Witness 2605.10111
//! (`\keywords{...}` at templateArxiv.tex L54); same-host Perl reports the
//! identical single `undefined:\keywords`.
use latexml_package::prelude::*;

LoadDefinitions!({
  if lookup_bool("INCLUDE_STYLES") {
    // Raw style loading is on: the paper's own PRIMEarxiv.sty is authoritative.
    InputDefinitions!("PRIMEarxiv", noltxml => true, extension => Some(Cow::Borrowed("sty")));
  } else {
    // PRIMEarxiv.sty drops arxiv.sty's `\headeright` / `\undertitle` /
    // `\shorttitle` (L27-32 of the upstream file), so only the keyword
    // pair — plus the two `\RequirePackage` dependencies — is ported.
    RequirePackage!("geometry");
    RequirePackage!("fancyhdr");
    crate::arxiv_sty::define_keywords(r"{\bfseries \emph Keywords}")?;
  }
});
