use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: amsbook.cls.ltxml
  // Ignorable options
  for option in ["a4paper", "letterpaper", "landscape",
    "8pt", "9pt", "10pt", "11pt", "12pt",
    "oneside", "twoside", "draft", "final",
    "titlepage", "notitlepage", "onecolumn", "twocolumn",
    "leqno", "reqno", "centertags", "tbtags",
    "fleqn", "openright", "openany",
    "makeindex", "nomath", "noamsfonts"].iter()
  {
    DeclareOption!(*option, None);
  }
  DeclareOption!(None, {
    Digest!("\\PassOptionsToClass{\\CurrentOption}{book}")?;
  });
  ProcessOptions!();
  LoadClass!("book");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  RequirePackage!("amsfonts");
});
