//! scrreprt.cls — the KOMA-Script report class, raw-interpreted through the
//! engine. Same shape and rationale as `scrartcl_cls.rs` (the former
//! OmniBus stub is at git history 3c9baade57^); the shared post-load
//! patches live in `koma_script.rs`.
use latexml_package::prelude::*;

LoadDefinitions!({
  InputDefinitions!("scrreprt", noltxml => true, extension => Some(Cow::Borrowed("cls")));
  crate::koma_script::koma_post_load()?;
});
