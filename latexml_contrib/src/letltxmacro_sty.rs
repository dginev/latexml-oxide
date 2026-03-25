use latexml_package::prelude::*;

LoadDefinitions!({
  InputDefinitions!("letltxmacro", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
