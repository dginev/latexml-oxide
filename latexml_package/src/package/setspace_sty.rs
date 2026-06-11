//! setspace.sty — line spacing (no-op in LaTeXML)
//! Perl: setspace.sty.ltxml
use crate::prelude::*;

LoadDefinitions!({
  def_macro_noop("\\singlespacing")?;
  def_macro_noop("\\onehalfspacing")?;
  def_macro_noop("\\doublespacing")?;
  def_macro_noop("\\setstretch{}")?;
  def_macro_noop("\\SetSinglespace{}")?;
  def_macro_noop("\\setdisplayskipstretch{}")?;
  def_macro_noop("\\restore@spacing")?;

  // {spacing}{}: the body-wrapping variant. Keep BOUND_MODE vertical so `$$`
  // inside it still enters display math. Witness 2305.08368:
  // `\begin{spacing}{1.25}` wrapping the whole body made `$$x_1=...$$` fall
  // through the `$` handler's vertical-only check (tex_math.rs:447) → 199
  // `Error:unexpected:_` cascades. Perl-faithful binding is `#body` only, but
  // the default Package.pm mode is `restricted_horizontal`; in Rust we have
  // to make `internal_vertical` explicit so paragraphs and display math
  // survive the wrap.
  DefEnvironment!("{spacing}{}", "#body", mode => "internal_vertical");
  DefEnvironment!("{singlespace}", "#body");
  DefEnvironment!("{onehalfspace}", "#body");
  DefEnvironment!("{doublespace}", "#body");
  // Standalone-switch overrides: some papers (witness 2310.08233 IEEEtran)
  // use `\singlespace` as a SWITCH inside an arg-grabbing context such as
  // `\title{\singlespace ...}`. DefEnvironment binds `\singlespace` to the
  // env-begin CS, which pushes a (restricted_)horizontal mode-switch group.
  // That group cannot be cleanly popped at arg-close, leaking into the
  // following `\@add@frontmatter@now`. Override the env-begin CS (and its
  // sibling `\endsinglespace`) with plain noops so standalone use is inert.
  // For `\begin{singlespace}...\end{singlespace}` blocks: \begin/\end track
  // env scope themselves; the begin/end CSes are just hooks. Without the
  // hooks the body still renders (it is just literal #body), so the env
  // form remains correct.
  def_macro_noop("\\singlespace")?;
  def_macro_noop("\\endsinglespace")?;
  def_macro_noop("\\onehalfspace")?;
  def_macro_noop("\\endonehalfspace")?;
  def_macro_noop("\\doublespace")?;
  def_macro_noop("\\enddoublespace")?;
});
