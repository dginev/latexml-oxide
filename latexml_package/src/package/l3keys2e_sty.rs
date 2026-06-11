use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: l3keys2e.sty.ltxml
  let _ = input_definitions("l3keys2e", NewDefault!(InputDefinitionOptions,
    noltxml => true, extension => Some(Cow::Borrowed("sty"))));
  // Restore catcodes after expl3 syntax changes from raw loading
  assign_catcode(' ', Catcode::SPACE, Some(Scope::Global));
  assign_catcode('\t', Catcode::SPACE, Some(Scope::Global));
  assign_catcode('~', Catcode::ACTIVE, Some(Scope::Global));
  assign_catcode(':', Catcode::OTHER, Some(Scope::Global));
  assign_catcode('_', Catcode::SUB, Some(Scope::Global));
  raw_tex(r"\endlinechar=13\relax")?;
});
