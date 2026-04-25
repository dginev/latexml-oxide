use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl pgfplots.sty.ltxml — port handles the same InputDefinitions
  // flow that pulls in the raw pgfplots.sty on top of our pgf-latexml
  // shim. Perl L24 marks `\pgfplots@iffileexists` `locked => 1` so the
  // raw-TeX load can't clobber our \IfFileExists alias; Rust now mirrors.
  //
  // Still unported (Perl L27-33): compat-mode detection + autoset to
  // `mostrecent`. That requires Expand of `\pgfk@/pgfplots/compat/*` CSes
  // from the raw-sty body — safe to omit since pgfplots defaults to a
  // usable compat level without the autoset.
  DefMacro!("\\pgfplots@iffileexists", "\\IfFileExists", locked => true);
  InputDefinitions!("pgfplots", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});

// Open issue (sandbox 2026-04-24, 11 papers — 1305.3934 et al):
// `\pgfplots@curplotlist` and `\pgfplots@curlegend` undefined errors
// from raw pgfplots.code.tex L5790/5795 — those `\let`s are local to
// the `\pgfplots@drawplots@@@iter` body, but some axis/legend hooks
// reach them earlier. Naive defensive init at load (either via `\let`
// to `\pgfutil@empty` or `\def` to empty) routes downstream
// `\ifx ... \pgfutil@empty` matches into a token-limit infinite loop,
// regressing the cluster from conversion_error → fatal. Resolution
// path: identify the concrete macro that calls these CSes outside the
// iterator body and add a localised guard there, OR fix the iterator
// init order. Deferred — preserving the conversion_error baseline.
