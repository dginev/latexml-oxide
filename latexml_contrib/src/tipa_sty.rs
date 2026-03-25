use latexml_package::prelude::*;

LoadDefinitions!({
  // load raw for now.
  InputDefinitions!("tipa", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
