use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: subfiles.cls.ltxml
  // LaTeXML does not implement the subfiles class;
  // Please process the full main document.
  // We will punt by using the OmniBus generic class.
  Error!("unexpected", "subfiles", "LaTeXML does not implement the subfiles class; Please process the full main document. We will punt by using the OmniBus generic class");
  load_class("OmniBus", Vec::new(), Tokens!())?;
});
