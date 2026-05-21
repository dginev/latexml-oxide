//! media9.sty — embed interactive Flash, 3D and video into PDF
//! No Perl binding exists. The media9 package produces PDF-only
//! embedded media — not meaningful for HTML output.
//! Stub: define user-facing commands as no-ops.
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // media9 requires pdfbase.sty which uses expl3 and PDF primitives.
  // Neither is available in our engine. Stub the key user commands.
  def_macro_noop("\\includemedia[]{}{}")?;
  DefEnvironment!("{mediacommand}{}", "#body");
  def_macro_noop("\\mediabutton[]{}")?;
});
