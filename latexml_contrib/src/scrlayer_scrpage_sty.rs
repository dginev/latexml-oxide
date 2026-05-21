//! Stub for scrlayer-scrpage.sty (KOMA-Script page-layer / header-footer).
//!
//! scrlayer-scrpage and its base scrlayer/scrbase/scrkbase chain through
//! several thousand lines of KOMA macros, much of it page-layout for
//! print output. For XML/HTML conversion the layout is invisible; raw-
//! loading the chain triggers a token-limit runaway (100M tokens, 16GB
//! RAM in 40s on 2110.09330). Stub the user-facing API instead.
use latexml_package::prelude::*;

LoadDefinitions!({
  // Public scrlayer-scrpage commands (visual-only, gobble silently)
  def_macro_noop("\\pagestyle{}")?;
  def_macro_noop("\\thispagestyle{}")?;
  def_macro_noop("\\automark[]{}")?;
  def_macro_noop("\\automark*[]{}")?;
  def_macro_noop("\\clearpairofpagestyles")?;
  def_macro_noop("\\clearscrheadfoot")?;
  def_macro_noop("\\clearscrheadings")?;
  def_macro_noop("\\clearscrplain")?;
  def_macro_noop("\\setkomafont{}{}")?;
  def_macro_noop("\\addtokomafont{}{}")?;
  def_macro_noop("\\KOMAoptions{}")?;
  def_macro_noop("\\setuphead{}{}")?;
  def_macro_noop("\\setupfoot{}{}")?;
  def_macro_noop("\\lehead[]{}")?;
  def_macro_noop("\\cehead[]{}")?;
  def_macro_noop("\\rehead[]{}")?;
  def_macro_noop("\\lohead[]{}")?;
  def_macro_noop("\\cohead[]{}")?;
  def_macro_noop("\\rohead[]{}")?;
  def_macro_noop("\\lefoot[]{}")?;
  def_macro_noop("\\cefoot[]{}")?;
  def_macro_noop("\\refoot[]{}")?;
  def_macro_noop("\\lofoot[]{}")?;
  def_macro_noop("\\cofoot[]{}")?;
  def_macro_noop("\\rofoot[]{}")?;
  def_macro_noop("\\ihead[]{}")?;
  def_macro_noop("\\chead[]{}")?;
  def_macro_noop("\\ohead[]{}")?;
  def_macro_noop("\\ifoot[]{}")?;
  def_macro_noop("\\cfoot[]{}")?;
  def_macro_noop("\\ofoot[]{}")?;
  def_macro_noop("\\rohead*{}")?;
  // Options
  DeclareOption!(None, {});
  ProcessOptions!();
});
