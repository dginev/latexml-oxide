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
  RequirePackage!("ams_support");

  // Frontmatter/mainmatter/backmatter — Perl L46-56
  DefPrimitive!("\\frontmatter", None);
  DefPrimitive!("\\mainmatter", None);
  DefPrimitive!("\\backmatter", None);

  // List formatting — Perl L58-72
  DefMacro!("\\@listI", "\\leftmargin\\leftmargini\\parsep 4.5\\p@ plus2\\p@ minus\\p@\\topsep 8.5\\p@ plus3\\p@ minus4\\p@\\itemsep4.5\\p@ plus2\\p@ minus\\p@");
  Let!("\\@listi", "\\@listI");
  DefMacro!("\\@listii", "\\leftmargin\\leftmarginii\\labelwidth\\leftmarginii\\advance\\labelwidth-\\labelsep\\topsep 4\\p@ plus2\\p@ minus\\p@\\parsep 2\\p@ plus\\p@ minus\\p@\\itemsep\\parsep");
  DefMacro!("\\@listiii", "\\leftmargin\\leftmarginiii\\labelwidth\\leftmarginiii\\advance\\labelwidth-\\labelsep\\topsep 2\\p@ plus\\p@ minus\\p@\\parsep\\z@\\partopsep\\p@ plus\\z@ minus\\p@\\itemsep\\topsep");
  DefMacro!("\\@listiv", "\\leftmargin\\leftmarginiv\\labelwidth\\leftmarginiv\\advance\\labelwidth-\\labelsep");
  DefMacro!("\\@listv", "\\leftmargin\\leftmarginv\\labelwidth\\leftmarginv\\advance\\labelwidth-\\labelsep");
  DefMacro!("\\@listvi", "\\leftmargin\\leftmarginvi\\labelwidth\\leftmarginvi\\advance\\labelwidth-\\labelsep");

  // Perl L64-66: description end alias and \upn = \textup
  Let!("\\enddescription", "\\endlist");
  Let!("\\upn", "\\textup");
});
