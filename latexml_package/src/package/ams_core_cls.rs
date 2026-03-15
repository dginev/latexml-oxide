use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: ams_core.cls.ltxml — common support for amsart, amsproc, amsbook

  //======================================================================
  // Document structure.

  // None of the options are vital, I think; deferred.
  // [though loading an unwanted amsfonts (noamsfonts) could be an issue]
  for option in [
    "a4paper", "letterpaper", "landscape", "portrait",
    "oneside", "twoside", "draft", "final", "e-only",
    "titlepage", "notitlepage",
    "openright", "openany", "onecolumn", "twocolumn",
    "nomath", "noamsfonts", "psamsfonts",
    "centertags", "tbtags",
    "8pt", "9pt", "10pt", "11pt", "12pt",
    "makeidx",
  ].iter() {
    DeclareOption!(*option, None);
  }
  AssignMapping!("DOCUMENT_CLASSES", "ltx_leqno" => true); // Default is left!
  DeclareOption!("leqno", sub { AssignMapping!("DOCUMENT_CLASSES", "ltx_leqno" => true); });
  DeclareOption!("reqno", sub { assign_mapping("DOCUMENT_CLASSES", "ltx_leqno", None::<bool>); });
  DeclareOption!("fleqn", sub { AssignMapping!("DOCUMENT_CLASSES", "ltx_fleqn" => true); });

  ProcessOptions!();

  // I think all options are (non)handled above, so don't need to pass any.
  load_class("article", Vec::new(), Tokens!())?;
  RequirePackage!("ams_support");
});
