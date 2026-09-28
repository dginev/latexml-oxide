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
  // amsart.cls:159-162: `leqno` and `reqno` also go to amsmath, whose own
  // switch sets the tags (amsmath.sty:53-55; the binding resets `ltx_leqno`
  // as it loads, DIVERGENCES #336). `leqno` is the default (amsart.cls:350).
  DeclareOption!("leqno", sub {
    AssignMapping!("DOCUMENT_CLASSES", "ltx_leqno" => true);
    Digest!("\\PassOptionsToPackage{leqno}{amsmath}")?;
  });
  DeclareOption!("reqno", sub {
    assign_mapping("DOCUMENT_CLASSES", "ltx_leqno", None::<bool>);
    Digest!("\\PassOptionsToPackage{reqno}{amsmath}")?;
  });
  DeclareOption!("fleqn", sub { AssignMapping!("DOCUMENT_CLASSES", "ltx_fleqn" => true); });
  execute_options(&["leqno"])?; // Default is left!

  ProcessOptions!();

  // I think all options are (non)handled above, so don't need to pass any.
  load_class("article", Vec::new(), Tokens!())?;
  RequirePackage!("ams_support");
  ams_support_sty::amsart_author_storage()?;
});
