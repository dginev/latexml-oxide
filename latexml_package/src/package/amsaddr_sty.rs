use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: amsaddr.sty.ltxml
  RequirePackage!("ams_support");
  // amsaddr.sty:43 `\let\emails\@empty`, the e-mail list its `\email` fills and its `\@maketitle` prints (:44-50,
  // :102-113); here `\email` is the frontmatter's contact, and a document's own `\email` adds to the list
  // (2606.23853, 2606.25954 `\g@addto@macro\emails`), as defined in TeX (OD #1036).
  RawTeX!(r"\let\emails\@empty");
});
