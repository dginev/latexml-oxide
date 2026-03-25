use latexml_package::prelude::*;

LoadDefinitions!({
  InputDefinitions!("commath", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
