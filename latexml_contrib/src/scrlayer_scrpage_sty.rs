//! scrlayer-scrpage.sty — KOMA-Script header/footer package, raw-interpreted
//! on top of the raw `scrlayer` (`scrlayer_sty.rs`) / scrkbase / scrbase chain.
//!
//! The former stub (git history: ~60 `def_macro_noop`s) defined no KOMA
//! option keys, so every `\KOMAoptions{headwidth=…,footsepline=…}` and
//! `\RedeclareLayer`/`\layerwidth`/`\DeclarePageStyleByLayers` a class issued
//! raised `unknown option` / `undefined:` errors (witness DEMO-TUDaPhD,
//! DEMO-TUDaThesis, neoschool, bfh-ci/DEMO-BFHLetter, zugferd, urcls,
//! makelabels, ijsra; original runaway witness arXiv 2110.09330). Header and
//! footer content is never shipped out, so the real package costs only its
//! load time.
use latexml_package::prelude::*;

LoadDefinitions!({
  InputDefinitions!("scrlayer-scrpage", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
