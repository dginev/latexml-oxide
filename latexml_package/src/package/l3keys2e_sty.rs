use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: l3keys2e.sty.ltxml
  // The raw load's expl3 syntax catcodes are restored to the values they had before it.
  let _ = expl3_sty::with_expl_syntax_restored(|| Ok(input_definitions("l3keys2e",
    NewDefault!(InputDefinitionOptions, noltxml => true, extension => Some(Cow::Borrowed("sty"))))));
});
