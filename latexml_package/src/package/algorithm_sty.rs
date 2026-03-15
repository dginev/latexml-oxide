use crate::prelude::*;

LoadDefinitions!({
  // Perl: InputDefinitions('algorithm', type => 'sty', noltxml => 1);
  // For now, just load algorithm; it leverages floats, and it works
  InputDefinitions!("algorithm", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
