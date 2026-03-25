use latexml_package::prelude::*;

LoadDefinitions!({
  InputDefinitions!("fp", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
