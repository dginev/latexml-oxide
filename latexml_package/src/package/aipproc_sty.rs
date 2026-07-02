use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  RequirePackage!("revtex3_support");
  assign_value("\\text:locked", Stored::None, Some(Scope::Global));
  RequirePackage!("longtable");
  RequirePackage!("psfig");
  def_macro_noop("\\lefthead{}")?;
  def_macro_noop("\\righthead{}")?;

  // Perl aipproc.sty.ltxml does NOT define \references — Perl behaves
  // lossy-silent when papers use `\begin{references}…\bibitem` under
  // aipproc (it drops the whole bibliography, reporting "No obvious
  // problems"). Rust's stricter validator surfaces the malformation
  // as Error:malformed:ltx:bibitem "…isn't allowed in <ltx:section>".
  //
  // Rust-over-Perl improvement: alias `\references` / `\endreferences`
  // to `\thebibliography` / `\endthebibliography`. This routes the
  // `\bibitem`s through the established bibliography machinery
  // (beginBibliography, item autoclose, etc.) and preserves the
  // bibliography content in the output — better than Perl's silent
  // drop. Fixes 4 papers in the 10k sandbox Class D bibitem-aipproc
  // cluster: astro-ph9711070, cond-mat0109365, nucl-ex9706010,
  // nucl-th0010030. See docs/archive/SANDBOX_TRIAGE_2026-05-21.md Class D sub-pattern.
  // `\reference` is `\let` to `\bibitem` ONLY WITHIN the `references` env
  // (in `\references`'s body), not globally — a global alias makes a paper's
  // own `\newcommand{\reference}{…}` (math shorthand) silently fail, leaving
  // `\reference`=`\bibitem` to fire inside `$…$` math → bibitem-in-XMArg leak.
  // See aipproc_cls.rs for the full rationale (witness 1701.08966).
  DefMacro!("\\references", "\\let\\reference\\bibitem\\thebibliography{}");
  Let!("\\endreferences", "\\endthebibliography");
});
