use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  InputDefinitions!("tlp", noltxml => true, extension => Some(Cow::Borrowed("cls")));
  DefMacro!("\\citeauthoryear{}{}{}", "#2 #3");
});
