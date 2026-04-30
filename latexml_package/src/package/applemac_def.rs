use crate::prelude::*;
LoadDefinitions!({
  // Perl: applemac.def.ltxml — predefine \textapplelogo, then load real
  // applemac.def encoding definitions.
  DefPrimitive!("\\textapplelogo", "[applelogo]");
  InputDefinitions!("applemac", extension => Some("def".into()), noltxml => true);
});
