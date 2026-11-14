//! amsdtx.cls — the AMS documentation class for package docs.
//!
//! Same `\@nobslash` roundtrip hazard as amsldoc.cls (amsdtx.cls L79/L102) —
//! see amsldoc_cls.rs for the mechanism.
use latexml_package::prelude::*;

LoadDefinitions!({
  InputDefinitions!("amsdtx", noltxml => true, extension => Some(Cow::Borrowed("cls")));
  crate::amsldoc_cls::amsdoc_patch_nobslash()?;
});
