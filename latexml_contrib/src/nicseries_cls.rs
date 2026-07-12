//! nic-series.cls — the "NIC Series" (John von Neumann Institute for Computing)
//! proceedings class (author-bundled; not raw-loaded). OmniBus renders the
//! `\author … \inst{…} \and …` / `\institute{…}` frontmatter fine; the only
//! stray is `\authortoc`, a table-of-contents-only short author list that leaks
//! as literal text (witness 2410.14397). Gobble it (it duplicates `\author`).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  def_macro_noop("\\authortoc{}")?;
});
