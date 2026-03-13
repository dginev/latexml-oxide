use crate::prelude::*;
LoadDefinitions!({
  // Predefine "comma below" accent, defined in latin10.def as ugly ooalign
  DefAccent!("\\textcommabelow", '\u{0326}', ",", below => true);
  // Now, read the rest from the REAL latin10
  InputDefinitions!("latin10", extension => Some("def".into()), noltxml => true);
});
