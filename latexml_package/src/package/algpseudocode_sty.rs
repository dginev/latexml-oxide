use crate::prelude::*;

LoadDefinitions!({
  // Perl: InputDefinitions('algpseudocode', type => 'sty', noltxml => 1);
  // This will load our hacked algorithmicx
  InputDefinitions!("algpseudocode", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
