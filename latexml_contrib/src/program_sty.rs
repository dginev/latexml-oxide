use latexml_package::prelude::*;

LoadDefinitions!({
  InputDefinitions!("program", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
