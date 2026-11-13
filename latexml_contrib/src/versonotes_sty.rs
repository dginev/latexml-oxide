//! versonotes.sty — notes set on the verso page facing the text.
//!
//! `\versonote{text}` writes its note to the `.aux` (versonotes.sty:42-45,
//! `\verso@note{page}{ypos}{text}`), and the NEXT run sets it on the facing
//! verso page from the shipout hook (:217-254). A single pass never reads it
//! back, so every note was lost, in Perl too (no binding; versonotes/sample
//! recall 61.5 %, sweep #121). Here the note is a margin note where it is
//! written, as `\marginpar` is (OXIDIZED_DESIGN_DIVERGENCES #290). Guard:
//! `class_census::versonote_is_a_margin_note`.
use latexml_package::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  InputDefinitions!("versonotes", noltxml => true, extension => Some(Cow::Borrowed("sty")));
  DefMacro!("\\versonote{}", "\\marginpar{#1}");
});
