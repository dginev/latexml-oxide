//! scrbase.sty — part of the KOMA-Script support chain, raw-interpreted.
//! A registered binding so the raw load also happens under the default
//! (arXiv) configuration, where a bindingless `.sty` is only
//! dependency-scanned — the raw KOMA classes (`scrartcl_cls.rs`) and
//! `scrlayer-scrpage` need the real definitions in every configuration.
use latexml_package::prelude::*;

LoadDefinitions!({
  InputDefinitions!("scrbase", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
