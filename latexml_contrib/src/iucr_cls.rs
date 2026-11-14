//! iucr.cls (the IUCr journals' class, shipped by the papers that use it; not in TeX Live) keeps a float's caption for
//! its own `figure`/`table` end code: `\hcaption{…}` stores it with `\gdef\captionbox` (iucr.cls:6645-6890) and the
//! environment's end prints it (:3828-3840). The kernel's float bindings own those environments, so that end code never
//! runs and every caption, with its `\label`, was lost (1707.09045: no caption in the document,
//! `Table~\ref{model:direct_model}` dangling). The class is read as it is; `\hcaption` is the caption. Guard
//! `perfect_kernel_batch80a::iucr_hcaption_is_the_caption`.
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  InputDefinitions!("iucr", noltxml => true, extension => Some(Cow::Borrowed("cls")));
  Let!("\\hcaption", "\\caption");
});
