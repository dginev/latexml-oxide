use latexml_package::prelude::*;

LoadDefinitions!({
  InputDefinitions!("bussproofs", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
