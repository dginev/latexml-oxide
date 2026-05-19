use crate::prelude::*;

/// DEP-20 helper for empty-body `DefPrimitive!("\\cs[opt-spec]", None);` stubs.
/// Mirrors `def_macro_noop` but routes through `def_primitive` so the CS
/// is registered as a digestion-time primitive rather than an expandable
/// macro. Body=None is treated as a no-op primitive (no Box emitted).
fn def_primitive_noop(proto: &str) -> Result<()> {
  let (cs_tok, params) = parse_prototype(proto, true)?;
  def_primitive(cs_tok, params, None, PrimitiveOptions::default())?;
  Ok(())
}


#[rustfmt::skip]
LoadDefinitions!({
  // Perl: amsbook.cls.ltxml
  // Ignorable options (Perl L22-30)
  for option in ["a4paper", "letterpaper", "landscape", "portrait",
    "8pt", "9pt", "10pt", "11pt", "12pt",
    "oneside", "twoside", "draft", "final", "e-only",
    "titlepage", "notitlepage", "onecolumn", "twocolumn",
    "centertags", "tbtags",
    "openright", "openany",
    "makeidx", "nomath", "noamsfonts", "psamsfonts"].iter()
  {
    DeclareOption!(*option, None);
  }
  // Perl L31-34: default ltx_leqno => 1 (left equation numbers), then
  // `leqno` re-asserts, `reqno` clears, `fleqn` sets ltx_fleqn=1. Rust
  // previously declared leqno/reqno/fleqn as no-ops, so amsbook docs
  // with [reqno] still rendered left-numbered equations.
  AssignMapping!("DOCUMENT_CLASSES", "ltx_leqno" => true);
  DeclareOption!("leqno", { AssignMapping!("DOCUMENT_CLASSES", "ltx_leqno" => true); });
  DeclareOption!("reqno", { assign_mapping("DOCUMENT_CLASSES", "ltx_leqno", None::<bool>); });
  DeclareOption!("fleqn", { AssignMapping!("DOCUMENT_CLASSES", "ltx_fleqn" => true); });
  DeclareOption!(None, {
    Digest!("\\PassOptionsToClass{\\CurrentOption}{book}")?;
  });
  ProcessOptions!();
  LoadClass!("book");
  RequirePackage!("ams_support");

  // Frontmatter/mainmatter/backmatter — Perl L46-56
  def_primitive_noop("\\frontmatter")?;
  def_primitive_noop("\\mainmatter")?;
  def_primitive_noop("\\backmatter")?;

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
