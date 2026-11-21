//! iucr.cls (the IUCr journals' class, shipped by the papers that use it; not in TeX Live) keeps a float's caption for
//! its own `figure`/`table` end code: `\hcaption{…}` stores it with `\gdef\captionbox` (iucr.cls:6645-6890) and the
//! environment's end prints it (:3828-3840). The kernel's float bindings own those environments, so that end code never
//! runs and every caption, with its `\label`, was lost (1707.09045: no caption in the document,
//! `Table~\ref{model:direct_model}` dangling). The class is read as it is; `\hcaption` is the caption. Guard
//! `perfect_kernel_batch80a::iucr_hcaption_is_the_caption`.
//!
//! The class read is the paper's copy as the document names it: the prefix fallback answers `iucrjournals` with this
//! binding, and its shipped `iucrjournals.cls` is the one TeX reads (`\IUCrCemaillink`: 2608.11466, 2609.21618,
//! 2607.09916; guard `perfect_kernel_batchregfixi::iucrjournals_reads_its_own_class`). A paper whose source lacks the
//! class falls to OmniBus, with the loader's missing-file Warning.
use latexml_package::prelude::*;

LoadDefinitions!({
  let Some(class) = latexml_package::shipped_class::requested_shipped_class("iucr")? else {
    return latexml_package::shipped_class::omnibus_without_copy("iucr");
  };
  LoadClass!("OmniBus");
  InputDefinitions!(&class, noltxml => true, extension => Some(Cow::Borrowed("cls")));
  Let!("\\hcaption", "\\caption");
  // iucrjournals.cls:201-206: `\IUCrCemaillink`/`\IUCrEmaillink` in `\author` print only a mail logo linking to the
  // address, which is lost from the author block (a tikz picture): the address is the author's email (2503.09183).
  for cs in ["\\IUCr@cemaillinkX", "\\IUCr@emaillinkX"] {
    if IsDefined!(&T_CS!(cs)) {
      RawTeX!(&s!("\\def{cs}#1{{\\lx@add@email{{#1}}}}"));
    }
  }
});
