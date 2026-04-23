use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: amsart.cls.ltxml — AMS journal article class

  //======================================================================
  // Document structure.

  // LoadClass('ams_core', withoptions => 1);
  load_class_with_options("ams_core", Tokens!())?;
  RequireResource!("ltx-amsart.css");

  // amsart.cls.ltxml : journal article
  // amsproc.cls.ltxml : article in book proceedings
  // amsbook.cls.ltxml : monograph
  // gen-j-l.cls.ltxml "generic journal article" => amsart
  // gen-p-l.cls.ltxml "generic proceedings article" => amsproc
  // gen-m-l.cls.ltxml "monograph" => amsbook

  //======================================================================
  // Sec. 5. Document Body
  Let!("\\specialsection", "\\chapter"); // Close enough?
});
