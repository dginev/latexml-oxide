use latexml_package::prelude::*;

LoadDefinitions!({
  // load raw for now.
  InputDefinitions!("tipa", noltxml => true, extension => Some(Cow::Borrowed("sty")));
  // tipa.sty internally calls `\RequirePackage[T3,\f@encoding]{fontenc}` to
  // pull in the T3 encoding definitions (`t3enc.def`), which is what
  // defines `\textrhookrevepsilon` / `\textbaru` / etc IPA symbols. Our
  // fontenc binding's `LoadDefinitions!` body only runs once per package,
  // so the already-loaded fontenc (with [T1] options) doesn't re-process
  // options when tipa's `\RequirePackage` re-arrives. Compensate by
  // directly reading `t3enc.def` here. Driver: arXiv:1802.05444.
  InputDefinitions!("t3enc", extension => Some(Cow::Borrowed("def")));
  // Native fallback for a trimmed host (feedback_ci_trimmed_texlive): when
  // neither tipa.sty nor t3enc.def is on disk the raw loads above define
  // nothing and `\textipa{...}` errors undefined (~12 sandbox papers). These
  // `\providecommand`s are idempotent — a no-op when the raw file loaded,
  // the sole definer when it did not — mirroring tipa.sty:75-76: `\textipa`
  // passes its argument through under a (here empty) `\tipaencoding` switch,
  // so the ASCII IPA source renders instead of aborting. SHARED with Perl
  // (no tipa.sty.ltxml; Perl relies on the same raw file).
  RawTeX!(r"\providecommand\tipaencoding{}\providecommand\textipa[1]{{\tipaencoding #1}}");
});
