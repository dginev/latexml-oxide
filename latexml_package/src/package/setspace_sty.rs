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

  // Paragraph-container envs: keep BOUND_MODE vertical so `$$` inside them
  // still enters display math. Witness 2305.08368: `\begin{spacing}{1.25}`
  // wrapping the whole body made `$$x_1=...$$` fall through the `$` handler's
  // vertical-only check (tex_math.rs:447) → 199 `Error:unexpected:_` cascades.
  // Perl-faithful binding is `#body` only, but the default Package.pm mode
  // is `restricted_horizontal`; in Rust we have to make `internal_vertical`
  // explicit so paragraphs and display math survive the wrap.
  DefEnvironment!("{singlespace}",  "#body", mode => "internal_vertical");
  DefEnvironment!("{onehalfspace}", "#body", mode => "internal_vertical");
  DefEnvironment!("{doublespace}",  "#body", mode => "internal_vertical");
  DefEnvironment!("{spacing}{}",    "#body", mode => "internal_vertical");
});
