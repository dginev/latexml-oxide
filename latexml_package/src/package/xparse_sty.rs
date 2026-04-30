//! xparse.sty — document command parser interface
//! Perl: xparse.sty.ltxml — loads the raw sty with noltxml
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl xparse.sty.ltxml L20-21: mark versioned xparse variants as
  // acceptable INTERPRETABLE_SOURCES so they bypass the version-suffix
  // strip fallback and can be fed directly to the raw-TeX loader.
  AssignMapping!("INTERPRETABLE_SOURCES", "xparse-2018-04-12.sty" => 1);
  AssignMapping!("INTERPRETABLE_SOURCES", "xparse-2020-10-01.sty" => 1);

  // Load raw xparse.sty — may hit errors from partial expl3 kernel.
  let _ = input_definitions("xparse", NewDefault!(InputDefinitionOptions,
    noltxml => true, extension => Some(Cow::Borrowed("sty"))));
  // Restore catcodes: xparse loading calls \ExplSyntaxOn which changes catcodes.
  // If \ExplSyntaxOff doesn't fully restore (due to partial expl3 kernel),
  // spaces become IGNORE and paragraphs break.
  state::assign_catcode(' ', Catcode::SPACE, Some(Scope::Global));
  state::assign_catcode('\t', Catcode::SPACE, Some(Scope::Global));
  state::assign_catcode('~', Catcode::ACTIVE, Some(Scope::Global));
  state::assign_catcode(':', Catcode::OTHER, Some(Scope::Global));
  state::assign_catcode('_', Catcode::SUB, Some(Scope::Global));
  raw_tex(r"\endlinechar=13\relax")?;
});
