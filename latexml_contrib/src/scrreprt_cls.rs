//! scrreprt.cls — the KOMA-Script report class, raw-interpreted through the
//! engine. Same shape and rationale as `scrartcl_cls.rs` (the former
//! OmniBus stub is at git history 3c9baade57^); the shared post-load
//! patches live in `koma_script.rs`.
use latexml_package::prelude::*;

LoadDefinitions!({
  crate::koma_script::koma_pre_load()?;
  InputDefinitions!("scrreprt", noltxml => true, extension => Some(Cow::Borrowed("cls")));
  crate::koma_script::koma_post_load(crate::koma_script::KomaClass::Chaptered)?;
  // (scrreprt.cls:3080-3083, :3206-3208, :3259: `\maketitle` marks its notes `\@fnsymbol` with or without a title page,
  // and resets the counter after it; scrreport.cls's `\LoadClassWithOptions{scrreprt}` too)
  latexml_package::engine::latex_constructs::declare_fnsymbol_title_block("scrreprt", None)?;
});
