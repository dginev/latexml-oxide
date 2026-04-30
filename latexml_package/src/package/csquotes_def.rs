use crate::prelude::*;
LoadDefinitions!({
  // Perl: csquotes.def.ltxml — a one-liner that loads the raw csquotes.def
  // encoding definitions.
  InputDefinitions!("csquotes", extension => Some("def".into()), noltxml => true);
});
