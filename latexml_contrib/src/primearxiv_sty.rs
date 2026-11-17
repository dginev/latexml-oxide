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
use latexml_package::{
  prelude::*,
  shipped_class::{FNSYMBOL_TITLE_MARKS, read_copy_title_marks},
};

LoadDefinitions!({
  if lookup_bool("INCLUDE_STYLES") {
    // Raw style loading is on: the paper's own PRIMEarxiv.sty is authoritative; the marks its `\maketitle` sets
    // declared (PRIMEarxiv.sty:176-179; 2509.03066, 2204.09716).
    read_copy_title_marks(&[FNSYMBOL_TITLE_MARKS], || {
      InputDefinitions!("PRIMEarxiv", noltxml => true, extension => Some(Cow::Borrowed("sty")));
      Ok(true)
    })?;
  } else {
    // PRIMEarxiv.sty drops arxiv.sty's `\headeright` / `\undertitle` /
    // `\shorttitle` (L27-32 of the upstream file), so only the keyword
    // pair — plus the two `\RequirePackage` dependencies — is ported.
    RequirePackage!("geometry");
    RequirePackage!("fancyhdr");
    crate::arxiv_sty::define_keywords(r"{\bfseries \emph Keywords}")?;
  }
});
