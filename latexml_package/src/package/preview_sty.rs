use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: preview.sty.ltxml — stub to avoid errors
  // preview is used for extracting previews of specific environments;
  // not practically useful with LaTeXML

  DefConditional!("\\ifPreview");

  for option in [
    "noconfig", "delayed", "psfixbb", "dvips", "pdftex", "xetex", "auctex", "lyx",
    "showlabels", "tightpage", "counters", "tracingall", "showbox",
  ] {
    DeclareOption!(option, None);
  }
  for option in ["displaymath", "textmath", "graphics", "floats", "sections", "footnotes"] {
    DeclareOption!(option, None);
  }
  DeclareOption!("active", None);

  DefMacro!("\\PreviewMacro OptionalMatch:* []{}", None);
  DefMacro!("\\PreviewEnvironment OptionalMatch:* []{}", None);
  DefMacro!("\\PreviewSnarfEnvironment OptionalMatch:* []{}", None);
  DefMacro!("\\PreviewOpen OptionalMatch:* []{}", None);
  DefMacro!("\\PreviewClose OptionalMatch:* []{}", None);

  DefEnvironment!("{preview}", "#body");
  DefEnvironment!("{nopreview}", "#body");

  DefRegister!("\\PreviewBorder", Dimension(0));
});
