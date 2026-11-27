//! scrbook.cls — the KOMA-Script book class, raw-interpreted through the
//! engine. Same shape and rationale as `scrartcl_cls.rs` (the former
//! OmniBus stub is at git history 3c9baade57^); the shared post-load
//! patches live in `koma_script.rs`.
use latexml_package::prelude::*;

LoadDefinitions!({
  crate::koma_script::koma_pre_load()?;
  InputDefinitions!("scrbook", noltxml => true, extension => Some(Cow::Borrowed("cls")));
  crate::koma_script::koma_post_load(crate::koma_script::KomaClass::Chaptered)?;
  // (scrbook.cls:3083-3086, :3209-3211, :3262: `\maketitle` marks its notes `\@fnsymbol` with or without a title page,
  // and resets the counter after it)
  latexml_package::engine::latex_constructs::declare_fnsymbol_title_block("scrbook", None)?;
  crate::koma_script::koma_book_matter()?;
});
