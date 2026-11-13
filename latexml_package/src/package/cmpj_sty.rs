//! cmpj.sty (Condensed Matter Physics, Lviv), with cmpj2.sty and cmpj3.sty — Rust-only bindings (Perl has none and
//! reads the package raw only under `--includestyles`, KNOWN_PERL_ERRORS #1233).
use crate::prelude::*;

LoadDefinitions!({
  load_cmpj_style("cmpj")?;
});

/// The real style `name` (the paper's copy first), raw, but `\refaddr`'s address marks print as TeX's second run does.
/// `\@@refadx` prints `\ref{key}` only once `r@key` is defined from the .aux, a bold "?" before (cmpj.sty:446-456,
/// cmpj2.sty:560-570, cmpj3.sty:659-669); LaTeXML resolves `\ref` itself and never defines `r@` (`\newlabel` is a no-op,
/// latex_constructs.pool.ltxml:5748), so the names showed "?" (1212.6367, 1212.6137, 1603.02437, 2112.09940).
pub fn load_cmpj_style(name: &str) -> Result<()> {
  // (read as the package itself, with the document's options, so its stores reach the frontmatter after it:
  // `\lx@package@loaded@raw`)
  let options: Vec<String> = lookup_vecdeque(&s!("opt@{name}.sty"))
    .map(|v| v.iter().map(|o| o.to_string()).collect())
    .unwrap_or_default();
  InputDefinitions!(name, noltxml => true, extension => Some(Cow::Borrowed("sty")), handleoptions => true,
    options => options);
  DefMacro!(
    r"\@@refadx{}",
    r"\let\@@refada\@empty\@@refad{\@for\@@refadb:=#1\do{\@@refada\def\@@refada{,\penalty\@m}\edef\@@refadb{\expandafter\@firstofone\@@refadb\@empty}\hbox{\ref{\@@refadb}}}}"
  );
  Ok(())
}
