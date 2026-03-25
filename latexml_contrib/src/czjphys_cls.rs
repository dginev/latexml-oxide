use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  InputDefinitions!("czjphys", noltxml => true, extension => Some(Cow::Borrowed("cls")));
});
