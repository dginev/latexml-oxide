//! feynmf.sty — Feynman diagrams with MetaFont
//! Perl: feynmf.sty.ltxml — loads the raw sty with noltxml
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  InputDefinitions!("feynmf", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
